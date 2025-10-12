use std::{
    fs,
    path::{Path, PathBuf},
};

use log::info;

use crate::{
    download::ALL_DB_TABLES,
    stats::{
        WikiIdent, create_wiki_idents,
        samples::{sample_bfs_stats, sample_bidirectional_bfs_stats},
        stats::{Stats, WebWikiSizes},
        utils::{global_ignore, make_stat_record_async, make_stat_record_seq},
    },
    web::find_smallest_wikis,
};

pub async fn add_web_wiki_sizes(output_path: impl AsRef<Path>, dump_date: Option<String>) {
    let output_path = output_path.as_ref();
    let mut stats = load_stats(output_path);

    let web_wiki_sizes = find_smallest_wikis(dump_date, &ALL_DB_TABLES)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed fetching wiki sizes from web: {e}");
        });

    stats.web_wiki_sizes = Some(WebWikiSizes {
        sizes: web_wiki_sizes,
        tables: ALL_DB_TABLES.iter().map(|s| s.to_string()).collect(),
    });
    save_stats(&stats, output_path);
}

/// Calculates bfs stats and adds or overwrites stats to/of existing json file
pub async fn add_sample_bfs_stats(
    output_path: impl AsRef<Path>,
    db_path: impl Into<PathBuf>,
    wikis: Vec<String>,
    sample_size: usize,
    num_threads: usize,
    cache_max_size: Option<usize>,
    always: bool,
) {
    let database_path = db_path.into();
    let wiki_idents: Vec<WikiIdent> = create_wiki_idents(&database_path, wikis);
    let path: &Path = output_path.as_ref();

    let mut stats = load_stats(path);

    let bfs_sample_stats = make_stat_record_seq(
        wiki_idents,
        |w_id: WikiIdent| sample_bfs_stats(w_id, sample_size, num_threads, cache_max_size),
        global_ignore,
        if !always {
            stats.bfs_sample_stats.clone()
        } else {
            None
        },
    );

    stats.bfs_sample_stats = Some(bfs_sample_stats.await);
    save_stats(&stats, path);
}

/// Calculates bidirectional bfs stats and adds or overwrites stats to/of existing json file
/// Not as interesting as doing a full bfs
pub async fn add_sample_bibfs_stats(
    output_path: impl AsRef<Path>,
    db_path: impl Into<PathBuf>,
    wikis: Vec<String>,
    sample_size: usize,
    num_threads: usize,
) {
    let output_path = output_path.as_ref();
    let database_path = db_path.into();
    let wiki_idents: Vec<WikiIdent> = create_wiki_idents(&database_path, wikis);

    let mut stats = load_stats(output_path);

    let bi_bfs_sample_stats = make_stat_record_async(
        wiki_idents,
        move |w_id: WikiIdent| sample_bidirectional_bfs_stats(w_id, sample_size, num_threads),
        global_ignore,
        stats.bi_bfs_sample_stats.clone(),
    );

    stats.bi_bfs_sample_stats = Some(bi_bfs_sample_stats.await);
    save_stats(&stats, output_path);
}

pub fn save_stats(stats: &Stats, path: impl AsRef<Path>) {
    let json = serde_json::to_string_pretty(&stats).unwrap();
    info!("Written to {:?}", path.as_ref());
    fs::write(&path, json).expect(&format!(
        "Failed writing stats to file {}",
        path.as_ref().display()
    ));
}

pub fn try_load_stats(path: &Path) -> Option<Stats> {
    if path.exists() {
        Some(load_stats(path))
    } else {
        None
    }
}

pub fn load_stats(path: &Path) -> Stats {
    serde_json::from_str(
        &fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed loading stats file: {path:?}")),
    )
    .unwrap_or_else(|_| panic!("Failed deserializing stats file: {path:?}"))
}
