use std::{collections::HashMap, path::Path};

use fxhash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    stats::{
        queries::count_from,
        samples::{BfsSample, BiBfsSample},
    },
    web::WebWikiSize,
    WikiIdent,
};

pub type StatRecord<T> = FxHashMap<String, T>;

pub type WikiName = String;
pub type PageTitle = String;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Stats {
    pub num_pages: StatRecord<u64>,
    pub num_redirects: StatRecord<u64>,
    pub num_links: StatRecord<u64>,
    pub num_linked_redirects: StatRecord<u64>,

    pub most_linked: StatRecord<Vec<LinkCount>>,
    pub most_links: StatRecord<Vec<LinkCount>>,

    pub longest_name: StatRecord<Page>,
    pub longest_name_no_redirect: StatRecord<Page>,

    pub num_dead_pages: StatRecord<u64>,
    pub num_orphan_pages: StatRecord<u64>,
    pub num_dead_orphan_pages: StatRecord<u64>,

    pub max_num_pages: (WikiName, u64),
    pub min_num_pages: (WikiName, u64),
    pub max_num_links: (WikiName, u64),
    pub min_num_links: (WikiName, u64),

    /// utc timestamp
    pub created_at: i64,
    pub dump_date: String,
    pub wikis: Vec<WikiName>,
    pub seconds_taken: u64,

    // can take really long
    pub bfs_sample_stats: Option<StatRecord<BfsSample>>,
    pub bi_bfs_sample_stats: Option<StatRecord<BiBfsSample>>,

    pub web_wiki_sizes: Option<WebWikiSizes>,
    pub local_wiki_sizes: Option<WikiSizes>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WikiSize {
    name: String,
    download_size: Option<u64>,
    processed_size: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WikiSizes {
    pub sizes: Vec<WikiSize>,
    pub tables: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WebWikiSizes {
    pub sizes: Vec<WebWikiSize>,
    pub tables: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct LinkCount {
    pub page_title: PageTitle,
    pub page_id: u64,
    pub wiki_name: WikiName,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Page {
    pub page_title: PageTitle,
    pub page_id: u64,
    pub wiki_name: WikiName,
}

pub fn num_pages_stat(wiki: WikiIdent) -> u64 {
    count_from("WikiPage", &wiki.db_path, "")
}

pub fn num_redirects_stat(wiki: WikiIdent) -> u64 {
    count_from("WikiPage", &wiki.db_path, "WHERE is_redirect = 1")
}

pub fn num_links_stat(wiki: WikiIdent) -> u64 {
    count_from("WikiLink", &wiki.db_path, "")
}

pub async fn get_local_wiki_sizes(base_path: impl AsRef<Path>, tables: &[&str]) -> WikiSizes {
    let download_path = base_path.as_ref().join("downloads");

    let decompressed_sizes: Vec<(String, u64)> = if fs::exists(&download_path).unwrap_or(false) {
        fs::read_dir(&download_path)
            .expect("Failed reading wiki download dir")
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                    let metadata = fs::metadata(&path).ok()?;
                    let file_size = metadata.len();
                    let file_stem = path.file_stem()?.to_str()?.to_string();
                    let wiki_name = file_stem.split('-').next()?.to_string();
                    Some((wiki_name, file_size))
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };

    let mut decompressed_size_map: HashMap<String, u64> = HashMap::new();
    for (key, value) in decompressed_sizes {
        *decompressed_size_map.entry(key).or_insert(0) += value;
    }

    let sqlite_path = base_path.as_ref().join("sqlite");
    let processed_sizes: HashMap<String, u64> = fs::read_dir(&sqlite_path)
        .expect("Failed reading wiki sqlite dir")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sqlite") {
                let metadata = fs::metadata(&path).ok()?;
                let file_size = metadata.len();
                let file_stem = path.file_stem()?.to_str()?.to_string();
                let wiki_name = file_stem.split("_database").next()?.to_string();
                Some((wiki_name, file_size))
            } else {
                None
            }
        })
        .collect();

    let sizes = processed_sizes
        .iter()
        .map(|(name, size)| WikiSize {
            name: name.clone(),
            download_size: decompressed_size_map.get(name).cloned(),
            processed_size: Some(*size),
        })
        .collect();

    let wiki_sizes = WikiSizes {
        sizes,
        tables: tables.iter().map(|s| s.to_string()).collect(),
    };

    wiki_sizes
}

// TODO: add test with sample test db files
