use clap::{ArgAction, Args, Parser, Subcommand, builder::styling};
use std::path::PathBuf;
use wiki_stats::download::ALL_DB_TABLES;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

#[derive(Parser, Debug)]
#[command(name = "WikiStats CLI")]
#[command(
    about = "Download and process wikipedia dumps",
    author = "Gaareth",
    version = clap::crate_version!(),
    styles = STYLES
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Logging verbosity -v to -vvvv (trace). Default is -vv (info)
    #[arg(short, long, action = ArgAction::Count, default_value_t = 2)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Download, unpack, create sqlite files
    ProcessDatabases {
        /// Path containing dump dates sub dirs with the download and sqlite sub directories
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,

        /// Names of the wikis to process (space separated so: e.g. enwiki, dewiki, jawiki)
        #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ', required = true)]
        wikis: Vec<String>,

        /// Remove downloads dir after processing
        #[arg(short, long, default_value_t = false)]
        remove_downloads: bool,

        /// Overwrite existing sqlite db files. Else an existing db file will be treated as already processed and the the program exits
        #[arg(long)]
        overwrite_sql: bool,

        /// Specify which dump date to use (defaults to latest). Format: YYYYMMDD
        #[arg(short, long)]
        dump_date: Option<String>,

        /// Validate
        #[arg(long, default_value_t = false, help_heading = "Validation Options")]
        validate: bool,

        /// Number of pages to validate
        #[arg(short, long, default_value_t = 2, help_heading = "Validation Options")]
        num_pages: u16,
    },

    /// Generate statistics about the dumps to a json file
    Stats {
        #[command(subcommand)]
        subcommands: StatsCommands,
    },

    /// Various task related commands
    Tasks {
        #[command(subcommand)]
        subcommands: TasksCommands,
    },

    /// Various debug related commands
    Debug {
        #[command(subcommand)]
        subcommands: DebugCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum TasksCommands {
    /// Returns all dump_dates that have the specified tables completed per wiki
    GetCompleteDumpdates {
        #[command(flatten)]
        args: WikisArgs,
    },

    /// Returns which dump_dates need to be done for the specified wikis and tables
    GetTasks {
        #[command(flatten)]
        args: WikisArgs,

        /// Path containing the json statistics files
        #[arg(long, value_name = "PATH")]
        stats_path: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum StatsCommands {
    /// Generate default stats
    Generate {
        #[command(flatten)]
        args: StatsArgs,

        /// Add bfs sample stats (quite expensive)
        #[arg(long, default_value_t = false, help_heading = "Sample Options")]
        add_sample: bool,

        #[command(flatten)]
        sample_args: SampleOptions,

        /// Add sizes of the online tables
        #[arg(long, default_value_t = false, help_heading = "WikiSizes Options")]
        add_web_wiki_sizes: bool,
    },

    /// Generate BFS sample stats (quite expensive). Make sure the output json file was already used for the normal stats
    AddSampleStats {
        #[command(flatten)]
        args: StatsArgs,

        #[command(flatten)]
        sample_args: SampleOptions,
    },

    /// Add sizes of the online table sizes and the downloaded sqlite files to the stats json file
    AddWebWikiSizes {
        #[command(flatten)]
        args: WikiSizesArgs,

        /// Output of the statistic json file
        #[arg(short, long, value_name = "PATH")]
        output_path: PathBuf,
    },
}

#[derive(Args, Debug)]
pub struct WikiSizesArgs {
    /// Path containing dump dates sub dirs with the download and sqlite sub directories
    #[arg(short, long, value_name = "PATH", help_heading = "WikiSizes Options")]
    pub base_path: PathBuf,

    /// Specify which dump date to use (defaults to latest). Format: YYYYMMDD
    #[arg(long, help_heading = "WikiSizes Options")]
    pub dump_date: Option<String>,
}

/// Arguments for the `stats` subcommand
#[derive(Args, Debug)]
pub struct StatsArgs {
    /// Output of the statistic json file
    #[arg(short, long, value_name = "PATH")]
    pub output_path: PathBuf,

    /// Path containing the sqlite db files
    #[arg(long, value_name = "PATH")]
    pub db_path: PathBuf,

    #[arg(short, long, num_args = 1.., value_delimiter = ' ', required = true, help = "Names of the wikis to process (space separated so: e.g. enwiki dewiki)")]
    pub wikis: Vec<String>,

    /// Generate stats for all wikis found in --db-path (conflicts with --wikis / -w)
    #[arg(long, conflicts_with = "wikis")]
    pub all_wikis: bool,

    /// Require validation
    #[arg(long, default_value_t = true)]
    pub require_validation: bool,
}

/// Common arguments for sample stats
#[derive(Args, Debug)]
pub struct SampleOptions {
    /// Specify sample size for stats
    #[arg(short, long, default_value_t = 500, help_heading = "Sample Options")]
    pub sample_size: usize,

    /// Specify the number of threads for sampling
    #[arg(short, long, default_value_t = 200, help_heading = "Sample Options")]
    pub threads: usize,

    /// Size of the cache how many links should be loaded into memory. Default is no limit
    #[arg(long, help_heading = "Sample Options")]
    pub cache_size: Option<usize>,

    /// Overwrite existing bfs stats in the output json file
    #[arg(long, default_value_t = false, help_heading = "Sample Options")]
    pub overwrite: bool,
}

/// Arguments wikis
#[derive(Args, Debug, Clone)]
pub struct WikisArgs {
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ', required = true)]
    pub wikis: Vec<String>,

    /// Specify which tables to download
    #[arg(short, long, num_args = 1.., default_values = &ALL_DB_TABLES)]
    pub tables: Vec<String>,
}

// /// Arguments for each wiki
// #[derive(Args, Debug, Clone)]
// pub struct WikiArgs {
//     /// Name of the wiki
//     #[arg(short, long)]
//     wiki: String,

//     /// Download all tables for this wiki
//     #[arg(short, long, conflicts_with = "tables")]
//     all_tables: bool,

//     /// Specify which tables to download (conflicts with --all-tables)
//     #[arg(short, long, requires = "wiki")]
//     tables: Vec<String>,
// }

#[derive(Subcommand, Debug)]
pub enum DebugCommands {
    GenTestData,
    GenStatsJSONSchema,
    FindSmallestWiki {
        #[arg(short, long, num_args = 1.., default_values = & ["pagelinks", "page"])]
        tables: Vec<String>,
    },
    /// Validate the processed sqlite fiels by checking if their links match whats is on the website
    ValidatePageLinks {
        /// Path of the sqlite db file
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,

        /// Which pages should be tested
        #[arg(short, long, group = "pages")]
        page_titles: Vec<String>,

        /// How many random pages should be tested
        #[arg(short, long, group = "pages")]
        num_pages: Option<u16>,

        /// Optionally specify which dump date to use (default: parsed from path). Format: YYYYMMDD
        #[arg(long)]
        dump_date: Option<String>,
    },

    /// Validate the downloaded sql dumps by checking if their links match whats is on the website
    PreValidate {
        /// Path containing the sql.gz and sql files
        #[arg(short, long, value_name = "PATH")]
        downloads_path: PathBuf,

        /// The wikiname, e.g. dewiki
        #[arg(short, long)]
        wiki: String,

        /// Which pages should be tested
        #[arg(short, long, group = "pages")]
        page_titles: Vec<String>,

        /// Optionally specify which dump date to use (default: parsed from downloads_path). Format: YYYYMMDD
        #[arg(long)]
        dump_date: Option<String>,
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

    SearchDump {
        /// Path containing the sql.gz and sql files
        #[arg(short, long, value_name = "PATH")]
        downloads_path: PathBuf,

        /// The wikiname, e.g. dewiki
        #[arg(short, long)]
        wiki: String,

        /// From title
        #[arg(long)]
        from: String,

        /// To title. If none print all
        #[arg(long)]
        to: Option<String>,

        /// Optionally specify which dump date to use (default: parsed from downloads_path). Format: YYYYMMDD
        #[arg(long)]
        dump_date: Option<String>,
    },
}
