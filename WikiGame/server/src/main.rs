use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use axum_streams::*;
use clap::{ArgAction, Parser, crate_version};
use dirs::home_dir;
use dotenv::dotenv;
use futures::{pin_mut, Stream, StreamExt};
use lazy_static::lazy_static;
use log::info;
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::process::exit;

use wiki_stats::calc::{bfs_bidirectional, bfs_stream};
use wiki_stats::sqlite::page_links::{get_cache, load_link_to_map_db_limit};
use wiki_stats::sqlite::{db_wiki_path, get_all_database_files, join_db_wiki_path};
use wiki_stats::stats::select_link_count_groupby;
use wiki_stats::{sqlite, DBCache};

// TODO: remove redirects?

// unfortunately necessary, as *I* cant put the cache in the axum state.
// It seems to create a reference in the server function
lazy_static! {
    static ref CACHES: HashMap<String, DBCache> = {
        let cli = Cli::parse();
        let (_, wikis) = validate_cli_args(cli.db_path, cli.wikis);
        get_caches(wikis, cli.num_load)
    };
}

#[derive(Debug)]
struct StatusError(StatusCode, String);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for StatusError {
    fn into_response(self) -> Response {
        dbg!(&self);
        (self.0, self.1).into_response()
    }
}

async fn get_shortest_path(
    State(state): State<AppState>,
    axum::extract::Path((wiki_name, start_title, end_title)): axum::extract::Path<(
        String,
        String,
        String,
    )>,
) -> Result<impl IntoResponse, StatusError> {
    println!("[{wiki_name}] {start_title} -> {end_title}");

    let path = db_wiki_path(&wiki_name);

    let wikis = state.wikis;
    if !wikis.contains(&wiki_name) {
        return Err(StatusError(
            StatusCode::NOT_FOUND,
            format!(
                "Unsupported wiki {wiki_name}. Supported: wikis: {}",
                wikis.join(",")
            ),
        ));
    }

    let conn = Connection::open(&path).map_err(|_| {
        StatusError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed connecting to db".to_string(),
        )
    })?;

    let start_link = PageTitle(start_title.clone());
    let start_link_id =
        sqlite::title_id_conv::page_title_to_id(&start_link, &conn).ok_or(StatusError(
            StatusCode::NOT_FOUND,
            format!("{start_title} is not a valid page for the {wiki_name}"),
        ))?;

    let end_link = PageTitle(end_title.clone());
    let end_link_id =
        sqlite::title_id_conv::page_title_to_id(&end_link, &conn).ok_or(StatusError(
            StatusCode::NOT_FOUND,
            format!("{end_title} is not a valid page for the {wiki_name}"),
        ))?;

    let cache = CACHES.get(&wiki_name).unwrap();

    let stream = bfs_stream(start_link_id, end_link_id, None, cache, path).await;
    return Ok(StreamBodyAs::json_nl(stream));
}

#[derive(Deserialize)]
struct SPOptions {
    stream: Option<bool>,
    start_title: String,
    end_title: String,
}

async fn get_shortest_path_bidirectional(
    State(state): State<AppState>,
    axum::extract::Path(wiki_name): axum::extract::Path<String>,
    params: Query<SPOptions>,
) -> Result<impl IntoResponse, StatusError> {
    let start_title = &params.start_title;
    let end_title = &params.end_title;

    info!(
        "{}",
        format!("Bidirectional sp: [{wiki_name}] {start_title} -> {end_title}")
    );

    let base_path = state.path;
    let wikis = state.wikis;
    if !wikis.contains(&wiki_name) {
        return Err(StatusError(
            StatusCode::NOT_FOUND,
            format!(
                "Unsupported wiki {wiki_name}. Supported: wikis: {:?}",
                wikis
            ),
        ));
    }
    let path = join_db_wiki_path(base_path, &wiki_name);
    let conn = Connection::open(&path).map_err(|_| {
        StatusError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed connecting to db".to_string(),
        )
    })?;

    let start_link = PageTitle(start_title.clone());
    let start_link_id =
        sqlite::title_id_conv::page_title_to_id(&start_link, &conn).ok_or(StatusError(
            StatusCode::NOT_FOUND,
            format!("{start_title} is not a valid page for the {wiki_name}"),
        ))?;

    let end_link = PageTitle(end_title.clone());
    let end_link_id =
        sqlite::title_id_conv::page_title_to_id(&end_link, &conn).ok_or(StatusError(
            StatusCode::NOT_FOUND,
            format!("{end_title} is not a valid page for the {wiki_name}"),
        ))?;

    let stream = bfs_bidirectional(start_link_id, end_link_id, path).await;
    if !params.stream.unwrap_or(false) {
        pin_mut!(stream);
        let mut last = stream.next().await;
        while let Some(v) = stream.next().await {
            last = Some(v);
        }
        last.map(|s| Json(json!(s)).into_response())
            .ok_or(StatusError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "No results?".to_string(),
            ))
    } else {
        // Ok(StreamBodyAsOptions::new()
        //     .buffering_ready_items(1).json_nl(stream).into_response())
        Ok(StreamBodyAs::json_nl(stream).into_response())
    }
    // return Ok(StreamBodyAs::json_nl(stream));
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SpStream {
    some_test_field: String,
}

#[derive(Debug, Clone, Serialize)]
struct MyTestStructure {
    some_test_field: String,
}

// Your possibly stream of objects
fn my_source_stream() -> impl Stream<Item = MyTestStructure> {
    // Simulating a stream with a plain vector and throttling to show how it works
    use tokio_stream::StreamExt;
    futures::stream::iter(vec![
        MyTestStructure {
            some_test_field: ":) ".to_string()
        };
        1000
    ])
    .throttle(std::time::Duration::from_millis(50))
}

async fn test_json_nl_stream() -> impl IntoResponse {
    StreamBodyAs::json_nl(my_source_stream())
}

fn validate_cli_args(
    db_path: Option<PathBuf>,
    wikis: Option<Vec<String>>,
) -> (PathBuf, Vec<String>) {
    let db_dir = db_path.unwrap_or_else(|| {
        let dir = std::env::var("DB_WIKIS_DIR").unwrap_or_else(|_| {
            eprintln!("Error: Please set DB_WIKIS_DIR to db wiki location or use --db-path");
            exit(1);
        });
        info!("Using DB_WIKIS_DIR environment variable");
        PathBuf::from(dir)
    });

    info!("Configuration: db path: {:?}.", db_dir);

    let wikis_to_check = wikis.unwrap_or_else(|| {
        info!("Using all wikis in db path");
        get_all_database_files(db_dir.clone()).expect("Failed to get database files")
    });

    info!("Configuration: Supported wikis: {:?}", wikis_to_check);

    for wiki_name in &wikis_to_check {
        // todo
        let path = join_db_wiki_path(db_dir.clone(), wiki_name);
        if !path.exists() || path.metadata().unwrap().len() == 0 {
            eprintln!("{wiki_name} Database at {path:?} does not exist or is emtpy");
            std::process::exit(1);
        }
        let _ =
            Connection::open(&path).unwrap_or_else(|error| panic!("Failed connecting to DB {path:?}: {error}"));
    }

    (db_dir, wikis_to_check)
}

fn get_caches(wikis: impl AsRef<[String]>, num_load: Option<usize>) -> HashMap<String, DBCache> {
    let mut db_cache: HashMap<String, DBCache> = HashMap::new();

    for wiki in wikis.as_ref().iter() {
        let cache = get_cache(db_wiki_path(wiki), num_load, false);
        db_cache.insert(wiki.to_string(), cache);
    }
    db_cache
}

#[derive(Clone)]
struct AppState {
    wikis: Vec<String>,
    path: PathBuf,
    // cache: Arc<HashMap<String, DBCache>>,
}

#[tokio::main]
async fn main() {
    if dotenv().is_err() {
        println!("Warning: No .env file found");
    }

    let cli = Cli::parse();

    let logfile_path = cli.logfile;
    println!("Logging to {logfile_path:?}");

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

    let (db_path, wikis) = validate_cli_args(cli.db_path, cli.wikis);

    let state = AppState {
        wikis: wikis.clone(),
        path: db_path,
        // cache: Arc::new(get_cache(cli.wikis.clone(), cli.num_load)),
    };

    let addr = format!("{}:{}", cli.host, cli.port);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World! The shortest path endpoint is at /path/<wiki_name>" }))
        .route("/path/:wiki", get(get_shortest_path_bidirectional))
        // .route("/test", get(test_json_nl_stream))
        .with_state(state);

    println!("Starting server at: {addr} with load: {:?} | Version: {}", cli.num_load, crate_version!());
    println!("Supported wikis: {:?}", wikis);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|error| {
        eprintln!("Error: Failed to bind to address: {error}");
        exit(1)
    });
    axum::serve(listener, app).await.unwrap();
}

#[derive(Parser, Debug)]
#[command(name = "Shortest Path Server")]
#[command(author = "Gaareth")]
#[command(about = "Start a WebServer returning the shortest path between two wikipedia pages")]
#[command(version = crate_version!())]
struct Cli {
    /// Server address
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Server port
    #[arg(short, long, default_value_t = 1870)]
    port: u16,

    /// Cache links of num_loads pages.
    #[arg(long)]
    num_load: Option<usize>,

    /// Path containing the sqlite db files. Use env var DB_WIKIS_DIR if not set.
    #[arg(short, long, value_name = "PATH")]
    db_path: Option<PathBuf>,

    /// List of supported wiki. E.g.: dewiki,jawiki (No space)
    #[arg(long, value_delimiter = ',')]
    wikis: Option<Vec<String>>,

    /// Logging verbosity -v to -vvvv (trace), default is -vv (info)
    #[arg(short, long, action = ArgAction::Count, default_value_t = 2)]
    verbose: u8,

    // Log file path
    #[arg(long, value_name = "PATH", default_value = "wiki-stats-sp-server.log")]
    logfile: PathBuf,
}
//TODO: implement cache for bidirectional
