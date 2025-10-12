use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use parse_mediawiki_sql::field_types::PageId;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::sqlite::db_wiki_path;
use crate::sqlite::title_id_conv::page_id_to_title;
use crate::{DepthHistogram, PrevMap};

// static PATHS_DB_LOCATION: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/paths.db";
// static PATHS_DB_LOCATION: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/de_paths.db";

// static PATHS_DB_LOCATION: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/de_paths.db";

pub(crate) fn create_db(path: impl AsRef<Path>) {
    let conn = Connection::open(path).expect("Failed creating database connection");

    // conn.execute(
    //     "CREATE TABLE if not exists SP_LinkData (
    //         id integer primary key,
    //         previous_id INTEGER NOT NULL,
    //         page_id INTEGER NOT NULL
    //     )",
    //     (),
    // ).expect("Failed creating table 'SP_LinkData'");
    //
    // conn.execute(
    //     "CREATE TABLE if not exists SP_Link (
    //         source_id INTEGER NOT NULL,
    //         data_id integer not null
    //     )",
    //     (),
    // ).expect("Failed creating table 'SP_Link'");

    conn.execute(
        "CREATE TABLE if not exists SP_Link (
            source_id INTEGER NOT NULL,
             previous_id INTEGER NOT NULL,
            page_id INTEGER NOT NULL
        )",
        (),
    )
    .expect("Failed creating table 'SP_Link'");

    conn.execute("PRAGMA synchronous = OFF", ()).unwrap();
}

pub fn create_index(conn: &Connection) {
    conn.execute(
        "CREATE INDEX SP_Link_pid_sid ON SP_Link(page_id, source_id);",
        (),
    )
    .expect("Failed creating INDEX SP_Link_pid_sid");
}

pub fn precalced_path_ids(path: impl AsRef<Path>) -> FxHashSet<PageId> {
    let conn = Connection::open(path).expect("Failed creating database connection");
    let mut stmt = conn.prepare("SELECT source_id FROM SP_Link").unwrap();

    let rows = stmt.query_map([], |row| row.get(0)).unwrap();

    let mut ids = FxHashSet::default();
    for id in rows {
        ids.insert(PageId(id.unwrap()));
    }

    ids
}

pub fn build_sp(start_link_id: &PageId, end_link_id: &PageId) {
    let p = "./test-paths.json";

    let existing_json_data = fs::read_to_string(&p).unwrap();
    let data: Data = serde_json::from_str(&existing_json_data).unwrap();
    let mut end_ids = vec![];
    let possible_indices = data.source_ids.get(start_link_id).unwrap();
    // dbg!(&possible_indices);
    let mut to_find = end_link_id.0;
    let mut path = vec![end_link_id.0];
    let conn = Connection::open(db_wiki_path("dewiki")).unwrap();

    'outer: loop {
        println!("looping");
        for (i, (pid, prev)) in data.data.iter().enumerate() {
            if pid == &to_find && possible_indices.contains(&i) {
                end_ids.push((pid, prev, i));
                path.push(*prev);
                to_find = *prev;
                dbg!(page_id_to_title(&PageId(*prev), &conn).unwrap().0);
                if prev == &start_link_id.0 {
                    print!("found");
                    break 'outer;
                }
            }
        }
    }

    // dbg!(&path.iter()
    //     .map(|pid| page_id_to_title(PageId(*pid), &conn).unwrap().0).collect::<Vec<String>>());
    // dbg!(path);
    dbg!(&end_ids);
}

#[derive(Serialize, Debug, Deserialize)]
struct Data {
    data: Vec<(u32, u32)>,
    source_ids: FxHashMap<PageId, FxHashSet<usize>>,
}

pub fn save_shortest_paths_json(
    path: impl AsRef<Path>,
    prev_map: &PrevMap,
    link_set: &mut FxHashMap<(PageId, PageId, PageId), usize>,
    start_link_id: &PageId,
) {
    let p = "./test-paths.json";
    let mut data: Data;

    if PathBuf::from(&p).exists() {
        let existing_json_data = fs::read_to_string(&p).unwrap();
        data = serde_json::from_str(&existing_json_data).unwrap();
    } else {
        data = Data {
            data: Vec::with_capacity(link_set.len()),
            source_ids: FxHashMap::default(),
        }
    }
    // dbg!(&data);

    for (pid, previous_id) in prev_map {
        link_set.insert((*start_link_id, *pid, *previous_id), link_set.len());
    }

    data.data = Vec::with_capacity(link_set.len());
    for ((source_id, pid, prev), idx) in link_set.iter().sorted_by_key(|x| x.1) {
        data.data.push((pid.0, prev.0));
    }

    // let mut source_ids: FxHashMap<PageId, Vec<usize>> = FxHashMap::default();

    for (pid, previous_id) in prev_map {
        let data_idx = *link_set.get(&(*start_link_id, *pid, *previous_id)).unwrap();
        // dbg!(&data_idx);
        data.source_ids
            .entry(*start_link_id)
            .or_default()
            .insert(data_idx);
    }

    // let json = json!({
    //     "data": data,
    //    "source_ids": source_ids,
    // });
    let json = serde_json::to_string_pretty(&data).unwrap();

    fs::write(&p, json).unwrap();
}

pub fn save_shortest_paths(
    path: impl AsRef<Path>,
    prev_map: &PrevMap,
    link_set: &mut FxHashMap<(PageId, PageId, PageId), usize>,
    start_link_id: &PageId,
) {
    create_db(&path);

    let mut conn = Connection::open(path).expect("Failed creating database connection");
    let tx = conn.transaction().unwrap();

    // {
    //     let mut stmt = tx.prepare_cached(
    //         "INSERT OR IGNORE INTO SP_LinkData (id, previous_id, page_id) VALUES (?1, ?2, ?3)").unwrap();
    //
    //     // dbg!(&prev_map.len());
    //     for (pid, previous_id) in prev_map {
    //         if link_set.contains_key(&(*pid, *previous_id)) {
    //             continue;
    //         }
    //
    //         link_set.insert((*pid, *previous_id), link_set.len());
    //         // dbg!(&pid);
    //         let previous_id = previous_id.0;
    //         let page_id = pid.0;
    //         stmt.execute((link_set.len() - 1, previous_id, page_id)).unwrap();
    //     }
    // }

    // {
    //     let mut stmt = tx.prepare_cached(
    //         "INSERT INTO SP_Link (source_id, data_id) VALUES (?1, ?2)").unwrap();
    //
    //     for (pid, previous_id) in prev_map {
    //         stmt.execute((start_link_id.0, link_set.get(&(*pid, *previous_id)).unwrap())).unwrap();
    //     }
    // }

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO SP_Link (source_id, previous_id, page_id) VALUES (?1, ?2, ?3)",
            )
            .unwrap();

        for (pid, previous_id) in prev_map {
            stmt.execute((start_link_id.0, previous_id.0, pid.0))
                .unwrap();
        }
    }

    tx.commit().unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SPStat {
    pub longest_path: VecDeque<String>,
    pub num_visited: u32,
    pub depth_histogram: DepthHistogram,
}

pub fn save_stats(wiki_name: String, page_title: String, stat: SPStat) {
    let path = format!("./sp-stats.json");
    type WikiName = String;
    type PageTitle = String;
    let mut data: FxHashMap<WikiName, FxHashMap<PageTitle, SPStat>> = FxHashMap::default();

    if PathBuf::from(&path).exists() {
        let existing_json_data = fs::read_to_string(&path).unwrap();
        data = serde_json::from_str(&existing_json_data).unwrap();
    }

    data.entry(wiki_name).or_default().insert(page_title, stat);
    let json = serde_json::to_string_pretty(&data).unwrap();
    // dbg!(&json);
    fs::write(&path, json).unwrap_or_else(|_| panic!("Failed writing to {}", &path));
}
