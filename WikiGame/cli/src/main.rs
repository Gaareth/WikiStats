use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::exit;

use anyhow::anyhow;
use clap::{ArgAction, Args, Parser, Subcommand};
use colored::Colorize;
use dirs::home_dir;
use indicatif::MultiProgress;
use log::{error, warn, LevelFilter};
use parse_mediawiki_sql::field_types::PageTitle;
use parse_mediawiki_sql::utils::memory_map;
use reqwest::StatusCode;
use rusqlite::Connection;
use schemars::schema_for;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

use wiki_stats::process::{process_threaded, process_wikis_seq, test_bench_threaded};
use wiki_stats::sqlite::load::load_linktarget_map;
use wiki_stats::sqlite::{get_all_database_files, join_db_wiki_path, DATABASE_SUFFIX};
use wiki_stats::stats::{Stats, WikiIdent};
use wiki_stats::web::find_smallest_wikis;
use wiki_stats::{download, stats, validate, web};

use crate::testdata::gentest;

mod testdata;

#[derive(Parser, Debug)]
#[command(name = "WikiStats CLI")]
#[command(
    about = "Download and process wikipedia dumps",
    author = "Gaareth",
    version = clap::crate_version!()
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Logging verbosity -v to -vvvv (trace). Default is -vv (info)
    #[arg(short, long, action = ArgAction::Count, default_value_t = 2)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download, unpack, create sql files
    ProcessDatabases {
        #[arg(
            short,
            long,
            value_name = "PATH",
            help = "Path containing dump dates sub dirs with the download and sqlite sub directories"
        )]
        path: PathBuf,

        #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ', required = true)]
        wikis: Vec<String>,

        #[arg(short, long, default_value_t = false)]
        skip_download: bool,

        #[arg(long)]
        overwrite_sql: bool,

        #[arg(short, long)]
        dump_date: Option<String>,
    },

    /// Generate stats
    Stats {
        #[command(flatten)]
        args: StatsArgs,

        /// Add sample stats
        #[arg(long)]
        add_sample: bool,
    },

    /// Generate BFS sample stats (quite expensive). Make sure the output json file was already used for the normal stats
    SampleStats(StatsArgs),

    // Get all DumpDates
    DumpDates {
        #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ', required = true)]
        wikis: Vec<String>,

        /// Specify which tables to download
        #[arg(short, long, num_args = 1.., default_values = & ["pagelinks", "page"])]
        tables: Vec<String>,
    },

    GetTasks {
        #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ', required = true)]
        wikis: Vec<String>,

        /// Specify which tables to download
        #[arg(short, long, num_args = 1.., default_values = & ["pagelinks", "page"])]
        tables: Vec<String>,

        /// Path containing the json statistics files
        #[arg(short, long, value_name = "PATH")]
        stats_path: PathBuf,
    },

    Debug {
        #[command(subcommand)]
        subcommands: DebugCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DebugCommands {
    GenTestData,
    GenStatsJSONSchema,
    FindSmallestWiki {
        #[arg(short, long, num_args = 1.., default_values = & ["pagelinks", "page"])]
        tables: Vec<String>,
    },
    ValidatePageLinks {
        #[arg(short, long, value_name = "PATH", help = "Path to db file")]
        path: PathBuf,
    },
    ValidateWikis {
        /// Path containing the json statistics files
        #[arg(short, long, value_name = "PATH")]
        json_path: Option<PathBuf>,

        #[arg(
            short,
            long,
            value_name = "PATH",
            help = "Path containing dump dates sub dirs with the download and sqlite sub directories"
        )]
        db_path: PathBuf,
    },
}

/// Arguments for the `stats` subcommand
#[derive(Args, Debug)]
struct StatsArgs {
    /// Output of the statistic json file
    #[arg(short, long, value_name = "PATH")]
    output_path: PathBuf,

    /// Path containing the sqlite db files
    #[arg(short, long, value_name = "PATH")]
    db_path: PathBuf,

    #[arg(short, long, num_args = 1.., value_delimiter = ' ', required = true)]
    wikis: Vec<String>,

    /// Generate stats for all wikis found in --db-path (conflicts with --wikis / -w)
    #[arg(long, conflicts_with = "wikis")]
    all_wikis: bool,

    #[command(flatten)]
    sample_options: SampleOptions,
}

/// Common arguments for sample stats
#[derive(Args, Debug)]
struct SampleOptions {
    /// Specify sample size for stats
    #[arg(short, long, default_value_t = 500)]
    sample_size: usize,

    /// Specify the number of threads for sampling
    #[arg(short, long, default_value_t = 200)]
    threads: usize,
}

/// Arguments for each wiki
#[derive(Args, Debug, Clone)]
struct WikiArgs {
    /// Name of the wiki
    #[arg(short, long)]
    wiki: String,

    /// Download all tables for this wiki
    #[arg(short, long, conflicts_with = "tables")]
    all_tables: bool,

    /// Specify which tables to download (conflicts with --all-tables)
    #[arg(short, long, requires = "wiki")]
    tables: Vec<String>,
}

//TODO: check first in env

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed reading env variables");
    let cli = Cli::parse();

    let logfile_path = home_dir()
        .expect("Failed retrieving home dir of OS")
        .join("wikiStats-cli.log");

    let logfile = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(&logfile_path)
        .unwrap_or_else(|_| panic!("Failed creating? logfile at {logfile_path:?}"));

    let term_loglevel = match cli.verbose {
        0 => LevelFilter::Error, // No verbosity -> Error level
        1 => LevelFilter::Warn,  // -v           -> Warn level
        2 => LevelFilter::Info,  // -vv          -> Info level
        3 => LevelFilter::Debug, // -vvv         -> Debug level
        _ => LevelFilter::Trace, // -vvvv or more -> Trace level
    };
    println!("LogLevel: {term_loglevel}");

    CombinedLogger::init(vec![
        TermLogger::new(
            term_loglevel,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, Config::default(), logfile),
    ])
    .unwrap();

    match &cli.command {
        Commands::ProcessDatabases {
            path,
            wikis,
            skip_download,
            dump_date,
            overwrite_sql,
        } => {
            validate_wiki_names(wikis)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki names".red()));
            let basepath = path;

            if !basepath.exists() {
                eprintln!(
                    "{}: The specified path does not exist: {}",
                    "Error".red(),
                    basepath.display().to_string().underline()
                );
                exit(-1);
            }
            // let names: Vec<String> = wikis.into_iter().map(|arg| wiki.clone()).collect();
            process_wikis_seq(wikis, basepath, dump_date.clone(), false).await; // 6m 4s no download???

            // process_threaded(wikis, basepath, dump_date.clone(), *overwrite_sql).await; // 169.16s
        }
        Commands::Stats { args, add_sample } => {
            let StatsArgs {
                output_path,
                db_path,
                wikis,
                all_wikis,
                sample_options,
            } = args;

            let wikis = if *all_wikis {
                &get_all_database_files(db_path).unwrap_or_else(|e| {
                    eprintln!("{}: {e}", "Failed fetching all wikis from db path".red());
                    exit(-1)
                })
            } else {
                wikis
            };
            println!("Wikis: {wikis:?}");

            validate_wiki_names(wikis)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki names".red()));
            validate_sqlite_files(db_path, wikis)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki sqlite files".red()));

            // dbg!(&validate_wiki_names(wikis));
            // exit(-1);

            let dump_date = db_path
                .parent()
                .and_then(Path::file_name)
                .and_then(|s| s.to_str())
                .expect("Failed extracting dumpdate from path. Please provide using --dump-date");
            println!("Assuming dump_date (from path): {dump_date}");

            println!(
                "Creating stats at {:?} using db files from: {:?}",
                &output_path, &db_path
            );
            wiki_stats::stats::create_stats(&output_path, wikis.clone(), db_path, dump_date).await;

            if *add_sample {
                let SampleOptions {
                    sample_size,
                    threads,
                } = sample_options;
                println!("Creating sample bfs stats..");
                wiki_stats::stats::add_sample_bfs_stats(
                    &output_path,
                    db_path,
                    wikis.clone(),
                    *sample_size,
                    *threads,
                    Some(0),
                    false,
                )
                .await;
            }
        }

        Commands::SampleStats(args) => {
            let StatsArgs {
                output_path,
                db_path,
                wikis,
                all_wikis,
                sample_options,
            } = args;

            let wikis = if *all_wikis {
                &get_all_database_files(db_path).unwrap_or_else(|e| {
                    eprintln!("{}: {e}", "Failed fetching all wikis from db path".red());
                    exit(-1)
                })
            } else {
                wikis
            };
            println!("Wikis: {wikis:?}");

            validate_wiki_names(wikis)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki names".red()));

            let SampleOptions {
                sample_size,
                threads,
            } = sample_options;
            println!("Creating sample bfs stats..");

            // wiki_stats::stats::find_wcc(WikiIdent::new(
            //     "dewiki",
            //     db_path.join("dewiki_database.sqlite"),
            // ))

            wiki_stats::stats::add_sample_bfs_stats(
                &output_path,
                db_path,
                wikis.clone(),
                *sample_size,
                *threads,
                None,
                true,
            )
            .await;

            
        }

        Commands::DumpDates { wikis, tables } => {
            validate_wiki_names(wikis)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki names".red()));
            println!("> Searching all dump dates for {wikis:?} where the tables {tables:?} are available");

            for wiki in wikis {
                let dump_dates = download::get_all_available_dump_dates(&wiki, tables).await;
                println!("Available '{wiki}' dump dates: {dump_dates:?}");
            }
        }

        Commands::GetTasks {
            wikis,
            tables,
            stats_path,
        } => {
            validate_wiki_names(wikis)
                .await
                .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki names".red()));
            if !stats_path.exists() {
                if let Err(e) = fs::create_dir_all(stats_path) {
                    eprintln!(
                        "{}: Failed to create stats_path directory: {}",
                        "Error".red(),
                        e
                    );
                    exit(-1);
                }
            }

            let mut finished_dump_dates: Vec<String> = fs::read_dir(stats_path)
                .unwrap()
                .flatten()
                .filter_map(|entry| {
                    let file_name = entry.file_name();
                    let file_name = file_name.to_str()?;

                    if file_name.ends_with(".json") {
                        let mut stats: Stats =
                            serde_json::from_str(&fs::read_to_string(entry.path()).unwrap_or_else(
                                |_| panic!("Failed loading stats file: {file_name:?}"),
                            ))
                            .unwrap_or_else(|_| {
                                panic!("Failed deserializing stats file: {file_name:?}")
                            });

                        // println!()
                        // if one requested wiki is not done here, it is not finished and will need to be redone
                        if wikis.iter().any(|w| !stats.wikis.contains(w)) {
                            return None;
                        }

                        let dump_date = stats.dump_date;
                        Some(dump_date)
                    } else {
                        None
                    }
                })
                .collect();

            finished_dump_dates.sort();
            println!("Finished dumpdates: {finished_dump_dates:?}");

            let available_dump_dates =
                download::get_all_available_dump_dates_for_all_wikis(wikis, tables).await;

            // remove dups
            let mut available_dump_dates: Vec<_> = available_dump_dates
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            available_dump_dates.sort();
            println!("Available dumpdates: {available_dump_dates:?}");

            let mut dump_dates_todo: Vec<String> = available_dump_dates
                .into_iter()
                .filter(|x| !finished_dump_dates.contains(x))
                .collect();
            dump_dates_todo.sort();

            println!("Todo dumpdates: {dump_dates_todo:?}")
        }

        Commands::Debug { subcommands } => {
            match subcommands {
                DebugCommands::GenTestData => {
                    let mmap = unsafe {
                        memory_map("/home/gareth/dev/Rust/WikiGame/lib/tests/data/dewiki-20240901-linktarget.sql").unwrap()
                    };
                    let map = load_linktarget_map(mmap);
                    assert!(!map.is_empty())
                    // gentest("/home/gareth/dev/Rust/WikiGame/lib/tests/data/dewiki-20240901-linktarget.sql",
                    //         "/home/gareth/dev/Rust/WikiGame/lib/tests/data/dewiki-20240901-linktarget-small.sql").unwrap()
                }
                DebugCommands::ValidatePageLinks { path } => {
                    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                    let prefix = &filename.clone()[..2];
                    println!("Assuming wikiprefix: {prefix}");

                    let dump_date = path.parent()
                        .and_then(Path::parent)
                        .and_then(Path::file_name)
                        .and_then(|s| s.to_str())
                        .expect("Failed extracting dumpdate from path. Please provide using --dump-date");

                    println!("Assuming dumpdate: {dump_date}");

                    let random_pages: Vec<PageTitle> = web::get_random_wikipedia_pages(2, prefix)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|p| PageTitle(p.title))
                        .collect();
                    let valid =
                        validate::post_validation(path, dump_date, prefix, &random_pages).await;
                    dbg!(&valid);

                    // dbg!(find_smallest_wikis(&["pagelinks", "page", "linktarget"]).await.unwrap());
                }
                DebugCommands::FindSmallestWiki { tables } => {
                    // println!("{:?}", find_smallest_wikis(tables).await.unwrap());
                    println!("> Sorting all wiki for which the sum of  tables {tables:?} is the smallest");
                    for wiki in find_smallest_wikis(tables).await.unwrap() {
                        println!("{wiki:?}");
                    }
                }
                DebugCommands::GenStatsJSONSchema => {
                    let file = File::create("stats-schema.json").expect("Failed creating file");
                    let schema = schema_for!(stats::Stats);
                    serde_json::to_writer_pretty(file, &schema)
                        .expect("Failed writing schema to file");
                }

                DebugCommands::ValidateWikis { db_path, json_path } => {
                    for entry in fs::read_dir(db_path).unwrap().flatten() {
                        let filename = entry.file_name();
                        let filename = filename.to_str().unwrap();
                        if entry.path().is_dir() && filename.chars().next().unwrap().is_numeric() {
                            println!("> Checking dumpdate directory '{filename}'");
                            let sqlite_dir = entry.path().join("sqlite");
                            let download_dir = entry.path().join("downloads");

                            let mut sql_wiki_names = Vec::new();
                            let mut download_wiki_names = Vec::new();

                            if download_dir.is_dir() {
                                for file in fs::read_dir(&download_dir).unwrap().flatten() {
                                    let filename = file.file_name();
                                    let filename = filename.to_str().unwrap().to_string();
                                    let wiki_name = filename.split_once("-").unwrap().0.to_string();
                                    download_wiki_names.push(wiki_name)
                                }
                            }
                            // remove dups
                            let download_wiki_names: Vec<_> = download_wiki_names
                                .into_iter()
                                .collect::<HashSet<_>>()
                                .into_iter()
                                .collect();

                            if sqlite_dir.is_dir() {
                                for file in fs::read_dir(&sqlite_dir).unwrap().flatten() {
                                    let filename = file.file_name();
                                    let filename = filename.to_str().unwrap().to_string();
                                    let wiki_name = filename.split_once("_").unwrap().0.to_string();
                                    if file.metadata().unwrap().size() > 0 {
                                        sql_wiki_names.push(wiki_name)
                                    } else {
                                        error!("{:?} is 0 bytes large!", file.path())
                                    }
                                }
                            }

                            // are there any db files that are not sql files?
                            let unprocessed_wikis: Vec<String> = download_wiki_names
                                .iter()
                                .filter(|x| !sql_wiki_names.contains(x))
                                .cloned()
                                .collect();

                            if !sql_wiki_names.is_empty() {
                                if unprocessed_wikis.is_empty() {
                                    println!("All downloaded files were processed");

                                    if download_dir.exists() {
                                        println!(
                                            "You may delete the download dir {download_dir:?}"
                                        );
                                    }
                                } else {
                                    println!("The following files were not processed correctly: {unprocessed_wikis:?}")
                                }
                            } else {
                                println!("{sqlite_dir:?} is empty!")
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod cli_test {
    use crate::{validate_wiki_names, Cli};

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[tokio::test]
    async fn test_validate_wikis() {
        assert!(validate_wiki_names(&["enwiki"]).await.is_ok());
        assert!(validate_wiki_names(&["enwiki", "jawiki"]).await.is_ok());

        assert!(validate_wiki_names(&["DOESNOTEXIST"]).await.is_err());
        assert!(validate_wiki_names(&["enwiki", "DOESNOTEXIST"])
            .await
            .is_err());
        assert!(validate_wiki_names(Vec::<String>::new().as_slice())
            .await
            .is_err());
    }

    // #[tokio::test]
    // async fn test_validate_sqlite_files() {
    //     assert!(validate_sqlite_files(&["jawiki"]).await.is_ok());
    //     assert!(validate_wiki_names(&["jawiki"]).await.is_err());
    // }
}

/// Checks if wikimedia has sql dumps for the wiki
/// Returns `Ok(())` if valid, or an `Err` with an error message if invalid.
async fn validate_wiki_name(wiki: &str) -> anyhow::Result<()> {
    if wiki.is_empty() {
        return Err(anyhow!("Wiki name cannot be empty."));
    }

    let resp = reqwest::get(format!("https://dumps.wikimedia.org/{wiki}/")).await?;

    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else if status == StatusCode::NOT_FOUND {
        Err(anyhow!("There are no dumps for '{wiki}' on https://dumps.wikimedia.org/. Check https://dumps.wikimedia.org/backup-index.html for available wikis"))
    } else {
        Err(anyhow!(
            "Error checking '{wiki}' on https://dumps.wikimedia.org/{wiki}/. StatusCode: {status}"
        ))
    }
}

async fn validate_wiki_names(wikis: &[impl AsRef<str>]) -> Result<(), String> {
    if wikis.is_empty() {
        return Err("Please provide at least one name".to_string());
    }

    for wiki in wikis {
        if let Err(e) = validate_wiki_name(wiki.as_ref()).await {
            return Err(e.to_string());
        }
    }
    Ok(())
}

/// Validate the wiki name
/// Returns `Ok(())` if valid, or an `Err` with an error message if invalid.
async fn validate_sqlite_file(db_path: impl AsRef<Path>, wiki: &str) -> anyhow::Result<()> {
    if wiki.is_empty() {
        return Err(anyhow!("Wiki name cannot be empty."));
    }
    let db_path = db_path.as_ref().to_path_buf();
    let path = join_db_wiki_path(db_path, wiki);
    Connection::open(path)?;
    Ok(())
}

async fn validate_sqlite_files(
    db_path: impl AsRef<Path>,
    wikis: &[impl AsRef<str>],
) -> Result<(), String> {
    if wikis.is_empty() {
        return Err("Please provide at least one name".to_string());
    }

    for wiki in wikis {
        if let Err(e) = validate_sqlite_file(&db_path, wiki.as_ref()).await {
            return Err(e.to_string());
        }
    }
    Ok(())
}

// let wiki_name = "dewiki";
//
//     let dump_date = download::download_wikis(vec!["dewiki"],
//                                              &["categorylinks", "page"],
//                                              "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/").await;
//
//     dbg!(&dump_date);
//     // sqlite::to_sqlite::create_category_links_db_quick(wiki_name, &dump_date,
//     //                                                   "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/");
//
//     sqlite::to_sqlite::create_title_id_conv_db_quick(wiki_name, &dump_date,
//                                                       "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/")

// wiki_stats::stats::create_stats().await

// precalc_interlinks_most_popular_threaded_quick("ruwiki");
//
// let old = unsafe { memory_map("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/20240301/downloads/enwiki-20240301-pagelinks.sql").unwrap() };
// let new = unsafe { memory_map("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/20240401/downloads/enwiki-20240401-pagelinks.sql").unwrap() };
//
// // let apr20 = unsafe { memory_map("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/20240420/downloads/enwiki-20240420-pagelinks.sql").unwrap() };
//
// let id_title_map = load_id_to_title("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/20240401/sqlite/enwiki_database_tobig.sqlite");
//
// let fmt: Box<dyn Fn(PageLink) -> String> = Box::new(move |pl: PageLink|
//     { format!("{}, {}",
//               id_title_map.get(&PageId(pl.from.0))
//                   .map(|p| p.clone().0).unwrap_or(pl.from.0.to_string()), pl.title.0) });
//
// sqlite::diff::diff_sqldump::<PageLink>(
//     &old, &new,
//     |pl| pl.from_namespace.0 != 0 || pl.namespace.0 != 0,
//     fmt);
// //
// sqlite::page_links::count_redirects(&db_wiki_path("dewiki"))

// bfs_bidirectional(&PageId(10455827), &PageId(1588), db_wiki_path("dewiki"));
// 0301:   785_164_001
// 0401: 1_591_804_203
// 0420: 1_180_857_228
// count_progress_bar::<PageLink>(&apr20);

// let b = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/";
// let mb = MultiProgress::new();
// let tosqlite = ToSqlite::new_bar("eswiki", "20240501", &mb, b);
// tosqlite.create_title_id_conv_db_default();
// tosqlite.create_pagelinks_db_custom(
//     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/eswiki-20240501-pagelinks.sql",
// "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/sqlite/eswiki_page_database.sqlite",
// "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/sqlite/eswiki_pagelinks_database.sqlite");

// process_threaded(wikis(vec!["es"]),b).await;
// process_wikis_seq(&wikis(&["de"]), "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/").await;

// let mmap = unsafe { memory_map(
//     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/20240801/downloads/jawiki-20240801-pagelinks.sql").unwrap() };
// let rows = iterate_sql_insertions::<PageLink>(&mmap).count();
// dbg!(&rows);

// let dt_str = chrono::Utc::now().format("%Y-%m-01").to_string();
// let path = format!("stats-{}.json", dt_str);
// dbg!(&path);

// dbg!(&wiki_stats::stats::sample_bidirectional_bfs_stats("dewiki".to_string(), 10_000, 50).await);
// wiki_stats::stats::add_sample_bfs_stats(&path).await;
// wiki_stats::stats::create_stats(&path).await;
// process_wiki("enwiki".to_string()).await;
// process_wiki("dewiki".to_string()).await;
//  let dump_date = download::download_wikis_all(wikis(vec!["en", "fr", "es", "ja", "ru"]),
//                                      "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/").await;
// process_wikis(wikis(vec!["de", "en", "fr", "es", "ja", "ru"])).await;
