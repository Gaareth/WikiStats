use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    os::unix::fs::MetadataExt,
    path::Path,
    process::exit,
};

use colored::Colorize;
use indicatif::ProgressStyle;
use log::{error, info};
use parse_mediawiki_sql::{
    field_types::{PageId, PageTitle},
    iterate_sql_insertions,
    schemas::{LinkTarget, PageLink},
    utils::{Mmap, memory_map},
};
use schemars::schema_for;
use wiki_stats::{
    calc::MAX_SIZE,
    parse_dump_date,
    sqlite::load::{load_links_map, load_linktarget_map, load_map},
    stats,
    validate::{self, validate_post_validation},
    web::{self, find_smallest_wikis},
};

use crate::{args::DebugCommands, print_error_and_exit};

pub async fn handle_debug_commands(subcommands: DebugCommands) {
    match subcommands {
        DebugCommands::GenTestData => {
            let mmap = unsafe {
                memory_map(
                    "/home/gareth/dev/Rust/WikiGame/lib/tests/data/dewiki-20240901-linktarget.sql",
                )
                .unwrap()
            };
            let map = load_linktarget_map(mmap);
            assert!(!map.is_empty())
            // gentest("/home/gareth/dev/Rust/WikiGame/lib/tests/data/dewiki-20240901-linktarget.sql",
            //         "/home/gareth/dev/Rust/WikiGame/lib/tests/data/dewiki-20240901-linktarget-small.sql").unwrap()
        }

        DebugCommands::PreValidate {
            downloads_path,
            wiki,
            page_titles,
            dump_date,
        } => {
            let dump_date = dump_date.unwrap_or(
                downloads_path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|s| s.to_str())
                    .expect(
                        "Failed extracting dumpdate from path. Please provide using --dump-date",
                    )
                    .to_string(),
            );

            let pl_sql_file_path =
                downloads_path.join(format!("{}-{}-pagelinks.sql", wiki, dump_date));
            let lt_sql_file_path =
                downloads_path.join(format!("{}-{}-linktarget.sql", wiki, dump_date));

            let page_titles_to_check: Vec<PageTitle> =
                page_titles.into_iter().map(|p| p.into()).collect();

            let (valid, _) = validate::pre_validation(
                pl_sql_file_path,
                lt_sql_file_path,
                &page_titles_to_check,
                dump_date,
                true,
            )
            .await;

            if !valid {
                print_error_and_exit!("Validation failed!")
            } else {
                println!("{}", "Validation successful".green());
            }
        }
        DebugCommands::ValidatePageLinks {
            path,
            num_pages,
            page_titles,
            dump_date,
        } => {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            let wiki_name = filename.split("_").next().unwrap().to_string();
            let prefix: &str = &filename.clone()[..2];
            println!("Assuming wikiprefix: {prefix}");

            let dump_date = dump_date.unwrap_or(
                path.parent()
                    .and_then(Path::parent)
                    .and_then(Path::file_name)
                    .and_then(|s| s.to_str())
                    .expect(
                        "Failed extracting dumpdate from path. Please provide using --dump-date",
                    )
                    .to_string(),
            );

            println!("Assuming dumpdate: {dump_date}");

            let random_pages: Vec<PageTitle> = if let Some(num_pages) = num_pages {
                web::get_random_wikipedia_pages(num_pages, prefix)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|p| PageTitle(p.title))
                    .collect()
            } else {
                page_titles.into_iter().map(|s| PageTitle(s)).collect()
            };

            // let random_pages = vec![PageTitle("Karlo Butić".to_string())];

            let (valid, post_diffs) =
                validate::post_validation(&path, &dump_date, prefix, &random_pages).await;

            if !valid {
                print!("Validation failed!");
                let dumpdate_path = path.parent().and_then(|p| p.parent()).unwrap().to_owned();
                if post_diffs.len() > 0 {
                    let msg = format!("[{wiki_name}] Failed post validation for {wiki_name:?}");
                    error!("{}", &msg);
                    info!(
                        "> Checking if differences are also inside the downloaded sql dump files"
                    );
                    if !validate_post_validation(&dump_date, &wiki_name, &dumpdate_path, post_diffs)
                        .await
                    {
                        print_error_and_exit!(
                            "[{wiki_name}] Failed post and pre validation for {path:?}"
                        )
                    }
                }
                exit(-1);
            } else {
                println!("{}", "Validation successful".green());
            }
        }
        DebugCommands::SearchDump {
            downloads_path,
            wiki,
            from,
            to,
            dump_date,
        } => {
            let dump_date = dump_date.unwrap_or(
                downloads_path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|s| s.to_str())
                    .expect(
                        "Failed extracting dumpdate from path. Please provide using --dump-date",
                    )
                    .to_string(),
            );

            let pl_sql_file_path =
                downloads_path.join(format!("{}-{}-pagelinks.sql", wiki, &dump_date));
            let lt_sql_file_path =
                downloads_path.join(format!("{}-{}-linktarget.sql", wiki, &dump_date));

            let filename_clone = pl_sql_file_path.clone();
            let filename = filename_clone.file_name().unwrap().to_str().unwrap();

            let wikiname = filename.split("-").next().unwrap();
            let wikiprefix = &wikiname[..2];
            let dump_date = parse_dump_date(dump_date.as_ref()).expect("Failed parsing dumpdate");

            let page_id: PageId = web::page_title_to_id(from.into(), wikiprefix).await;

            let pl_mmap: Mmap = unsafe { memory_map(pl_sql_file_path).unwrap() };
            let pl_map = load_links_map::<_, _, PageLink, _, _>(
                &pl_mmap,
                |pl| (pl.from, pl.target),
                |pl| pl.from_namespace.0 != 0 || pl.from != page_id,
            );

            println!("Loaded pl map");

            let lt_mmap: Mmap = unsafe { memory_map(lt_sql_file_path).unwrap() };
            let lt_map = load_map::<_, _, LinkTarget, _, _>(
                &lt_mmap,
                |lt| (lt.id, lt.title),
                |lt| {
                    lt.namespace.0 != 0
                        || if let Some(to) = &to {
                            lt.title != to.to_string().into()
                        } else {
                            false
                        }
                },
            );

            println!("Loaded lt map");

            for (page_id, link_targets) in pl_map.iter() {
                let page_title = web::page_id_to_title(*page_id, &wikiprefix).await;

                let links: HashSet<PageTitle> = HashSet::from_iter(
                    link_targets
                        .iter()
                        .filter_map(|lt| lt_map.get(&lt))
                        .cloned(),
                );

                if let Some(ref to) = to {
                    if links.contains(&PageTitle(to.clone())) {
                        println!(
                            "{}",
                            format!("{} -> {} in sql dump", page_title.0, to).green()
                        );
                    } else {
                        println!(
                            "{}",
                            format!("{} -> {} not in sql dump", page_title.0, to).red()
                        );
                    }
                }

                println!("{} links in {}", link_targets.len(), page_title.0);
                for link in links {
                    println!("{} -> {}", page_title.0, link.0);
                }
            }
        }
        DebugCommands::FindSmallestWiki { tables } => {
            // println!("{:?}", find_smallest_wikis(tables).await.unwrap());
            println!("> Sorting all wiki for which the sum of  tables {tables:?} is the smallest");
            for wiki in find_smallest_wikis(None, &tables).await.unwrap() {
                println!("{wiki:?}");
            }
        }
        DebugCommands::GenStatsJSONSchema => {
            let file = File::create("stats-schema.json").expect("Failed creating file");
            let schema = schema_for!(stats::Stats);
            serde_json::to_writer_pretty(file, &schema).expect("Failed writing schema to file");
            println!("Wrote json schema to stats-schema.json");
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
                                println!("You may delete the download dir {download_dir:?}");
                            }
                        } else {
                            println!(
                                "The following files were not processed correctly: {unprocessed_wikis:?}"
                            )
                        }
                    } else {
                        println!("{sqlite_dir:?} is empty!")
                    }
                }
            }
        }
    }
}
