#![feature(test)]
// #![feature(async_closure)]

use chrono::{DateTime, FixedOffset, NaiveDate, ParseResult, Utc};
use fxhash::FxHashMap;
use parse_mediawiki_sql::field_types::PageId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod sqlite;

pub mod bench;
pub mod calc;
pub mod download;
pub mod process;
pub mod stats;
pub mod utils;
pub mod validate;
pub mod web;

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
