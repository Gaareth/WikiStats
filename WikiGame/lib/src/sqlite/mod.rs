use std::fmt::Display;
use std::path::{Path, PathBuf};

pub mod page_links;
pub mod title_id_conv;
pub mod to_sqlite;
pub mod paths;
pub mod wiki;
pub mod load;
mod category_links;
pub mod diff;

// static DB_WIKIS_DIR: &str = std::env::var("DB_WIKIS_DIR").expect("Please set DB_WIKIS_DIR to db wiki location");

pub static DB_PATH: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/de_wiki_server.db";
// pub static PAGELINKS_DB: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/de_pagelinks_ACTUAL_Dups.db";
pub static PAGELINKS_DB: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/de_database.sqlite";

pub static DATABASE_SUFFIX: &str = "_database.sqlite";

pub fn join_db_wiki_path(db_path: impl Into<PathBuf>, wiki_name: impl AsRef<str>) -> PathBuf {
    db_path.into().join(format!("{}{DATABASE_SUFFIX}", wiki_name.as_ref()))
}
// todo: refactor to pathbuf / path?
pub fn db_wiki_path(wiki_name: impl AsRef<str> + Display) -> String {
    let db_wikis_dir: String = std::env::var("DB_WIKIS_DIR").expect("Please set DB_WIKIS_DIR to db wiki location");
    join_db_wiki_path(db_wikis_dir, wiki_name).to_str().unwrap().to_string()
}

pub fn db_sp_wiki_path(wiki_name: impl AsRef<str> + Display) -> String {
    let db_wikis_dir: String = std::env::var("DB_WIKIS_DIR").expect("Please set DB_WIKIS_DIR to db wiki location");
    format!("{db_wikis_dir}/{wiki_name}_sp_database.sqlite")
}
