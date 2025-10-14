#![feature(test)]
// #![feature(async_closure)]

use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, ParseResult, Utc};
use fxhash::FxHashMap;
use parse_mediawiki_sql::field_types::PageId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::sqlite::join_db_wiki_path;

pub mod sqlite;

pub mod bench;
pub mod calc;
pub mod download;
pub mod process;
pub mod stats;
pub mod utils;
pub mod validate;
pub mod web;

#[derive(Clone)]
pub struct WikiIdent {
    pub wiki_name: String,
    pub db_path: PathBuf,
}

impl WikiIdent {
    pub fn new<S: Into<String>>(wiki_name: S, db_path: PathBuf) -> Self {
        Self {
            wiki_name: wiki_name.into(),
            db_path,
        }
    }
}

pub fn create_wiki_idents(db_path: &Path, wikis: Vec<String>) -> Vec<WikiIdent> {
    wikis
        .into_iter()
        .map(|wiki_name| WikiIdent {
            db_path: join_db_wiki_path(db_path, &wiki_name),
            wiki_name,
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AvgDepthStat {
    pub avg_occurences: f64,
    pub std_dev: f64,
}

pub type DepthHistogram = FxHashMap<u32, u64>;
pub type AvgDepthHistogram = FxHashMap<u32, AvgDepthStat>;

pub type DistanceMap = FxHashMap<PageId, u32>;
pub type PrevMap = FxHashMap<PageId, PageId>;
pub type PrevMapEntry = (PrevMap, PageId, String);

pub type DBCache = FxHashMap<PageId, Vec<PageId>>;

pub fn parse_dump_date(date_str: &str) -> ParseResult<DateTime<Utc>> {
    Ok(NaiveDate::parse_from_str(date_str, "%Y%m%d")?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc())
}

pub fn format_as_dumpdate(datetime: &DateTime<Utc>) -> String {
    datetime.format("%Y%m%d").to_string()
}
