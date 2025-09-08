#![feature(test)]
#![feature(async_closure)]

use chrono::{DateTime, FixedOffset, NaiveDate, ParseResult, Utc};
use fxhash::FxHashMap;
use parse_mediawiki_sql::field_types::PageId;

pub mod sqlite;

pub mod calc;
pub mod utils;
pub mod stats;
pub mod download;
pub mod bench;
pub mod process;
pub mod validate;
pub mod web;

pub type DepthHistogram = FxHashMap<u32, u64>;
pub type AvgDepthHistogram = FxHashMap<u32, f64>;

pub type DistanceMap = FxHashMap<PageId, u32>;
pub type PrevMap = FxHashMap<PageId, PageId>;
pub type PrevMapEntry = (PrevMap, PageId, String);

pub type DBCache = FxHashMap<PageId, Vec<PageId>>;

pub fn parse_dump_date(date_str: &str) -> ParseResult<DateTime<Utc>> {
    Ok(NaiveDate::parse_from_str(date_str, "%Y%m%d")?.and_hms_opt(0,0,0).unwrap().and_utc())
}