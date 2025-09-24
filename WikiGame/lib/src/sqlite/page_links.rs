use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use indicatif::ProgressStyle;
use itertools::Itertools;
use log::info;
use num_format::{Locale, ToFormattedString};
use parse_mediawiki_sql::field_types::PageId;
use rusqlite::{Connection, Transaction};

use crate::sqlite::title_id_conv::load_wiki_pages;
use crate::stats::queries::select_link_count_groupby;
use crate::utils::{default_bar, ProgressBarBuilder};
use crate::{sqlite, DBCache};

// type InsertData = FxHashMap<PageId, Vec<PageTitle>>;
// type InsertData = FxHashSet<(PageId, PageTitle)>;

type InsertData = FxHashSet<(u32, String)>;

pub static DB_PAGELINKS_PATH: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/de_pagelinks2024.db";

// pub fn insert_into_sqlite(data: Vec<(PageId, PageTitle)>) {

pub fn db_setup(conn: &Connection) {
    conn.execute(
        "CREATE TABLE if not exists WikiLink (
            page_id INTEGER,
            page_link INTEGER
        )",
        (),
    )
    .expect("Failed creating table");
    //            UNIQUE (page_id, page_link)
    // conn.execute(
    //         "CREATE UNIQUE INDEX WikiLinks_page_id_page_links_key ON
    //         WikiLinks(page_id, page_links)", ()
    // );

    // conn.execute("PRAGMA journal_mode = MEMORY", ()).unwrap();
}

pub fn create_unique_index(conn: &Connection) {
    conn.execute(
        "CREATE UNIQUE INDEX if not exists WikiLink_unique_index ON
           WikiLink(page_id, page_link)",
        (),
    )
    .expect("Failed creating unique index");
}

pub fn create_indices_post_setup(conn: &Connection) {
    println!("Creating index..");
    conn.execute(
        "CREATE INDEX if not exists idx_link_id ON WikiLink(page_id);",
        (),
    )
    .expect("Failed creating index");

    conn.execute(
        "CREATE INDEX if not exists idx_link_page ON WikiLink(page_link);",
        (),
    )
    .expect("Failed creating index");

    // println!("Creating WikiLink(page_id, wiki_name)");
    //
    // conn.execute("
    //     CREATE INDEX if not exists idx_link_id_wiki ON WikiLink(page_id, wiki_name);
    // ", ()).unwrap();
    //
    // println!("Creating  WikiLink(page_link, wiki_name)");
    //
    // conn.execute("
    //            CREATE INDEX if not exists idx_link_linkid_wiki ON WikiLink(page_link, wiki_name);
    // ", ()).unwrap();

    // println!("Creating WikiLink(wiki_name, page_link DESC)");
    //
    // conn.execute("
    //     CREATE INDEX WikiLink_idx_name_link_desc ON WikiLink(wiki_name, page_link DESC);
    // ", ()).unwrap();
    //
    // println!("Creating WikiLink(wiki_name, page_id DESC)");
    //
    // conn.execute("
    //     CREATE INDEX WikiLink_idx_name_id_desc ON WikiLink(wiki_name, page_id DESC);
    // ", ()).unwrap();
}

// <I: Iterator<Item=PageLink>>
fn insert_into_sqlite(conn: &mut Connection, data: InsertData, length: usize) -> u32 {
    let t1 = Instant::now();
    let tx = conn.transaction().unwrap();

    let num_inserted = insert(&tx, data, length);
    tx.commit().expect("Failed committing transaction :(");

    println!("Successfully inserted data into db");

    let elapsed = t1.elapsed();
    let total_secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
    println!(
        "Elapsed: {:#?} \nSpeed: {} rows/s",
        elapsed,
        ((num_inserted as f64 / total_secs) as u64).to_formatted_string(&Locale::de)
    );

    return num_inserted;
}

//fn insert(tx: &Transaction, data: Vec<(PageId, PageTitle)>) {
// <I: Iterator<Item=PageLink>>
fn insert(tx: &Transaction, data: InsertData, length: usize) -> u32 {
    println!("Writing to database..");
    // let length = data.len();

    let bar = indicatif::ProgressBar::new(length as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.red} {bar:40.green/green} [{elapsed_precise}] {pos:>7}/{len:7} [{percent}%] {eta_precise} {per_sec}",
        )
            .unwrap(),
    );

    let statement = "INSERT INTO WikiLink VALUES (?, ?)".to_string();
    let mut stmt = tx.prepare_cached(statement.as_str()).unwrap();

    // let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/dewiki-20221001-pagelinks.sql";
    // let pagelinks_sql = unsafe { memory_map(path).unwrap() };

    let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/ja_page_db.sqlite";
    let title_id_map = sqlite::title_id_conv::load_title_id_map(path);
    // let mut inserted: FxHashSet<(u32, u32)> = FxHashSet::default();

    let mut duplicates = 0;
    let mut count = 0;
    // let mut data = iterate_sql_insertions::<PageLink>(&pagelinks_sql);

    // (from_id, to_title)
    for link in data {
        bar.inc(1);
        // if count > 210_712_457 / 100 {
        //     break;
        // }
        // let from_id = link.from;
        // let to_title = link.title;
        //
        // if link.from_namespace.0 != 0 || link.namespace.0 != 0 {
        //     // dbg!(&link);
        //     // dbg!(&page_id);
        //     continue;
        // }

        // let i: u32 = to_title.0.chars().fold(0, |acc, c| acc + (c  as u32));
        // let link = (from_id.0, i);
        let res = stmt.execute(link);
        // if res.is_err() {
        //     duplicates += 1;
        // }

        // removes link_target which is not namespace 0
        // if let Some(page_id) = title_id_map.get(&to_title) {
        //     // assert!(link.from_namespace.into_inner() == 0);
        //
        //
        //     let link = (from_id.0, page_id.0);
        //     let res = stmt.execute(link);
        //     if res.is_err() {
        //         duplicates += 1;
        //     }
        //
        //
        //     count += 1;
        // }
        count += 1;
    }

    println!("duplicates: {duplicates}");

    bar.finish();
    return count;
}

// unique am anfang dann iter rein
// Elapsed: 1189.229839135s
// Speed: 177.183 rows/s
// pub fn create_db(data: Vec<(PageId, PageTitle)>) {
// pub fn create_db(pagelinks_sql: Mmap, db_path: &str) {
//     let t1 = Instant::now();
//
//     if Path::new(db_path).exists() {
//         eprintln!("sqlite file {db_path} already exists");
//         exit(-1);
//     }
//
//     let mut conn = Connection::open(db_path)
//         .expect("Failed creating database connection");
//
//     db_setup(&conn);
//
//
//     // let num_entries = count_progress_bar(&pagelinks_sql);
//     let num_entries = MAX_SIZE as usize;
//     // // count_page_links_sqlfile()
//     // let num_parts: f32 = 3.5;
//     let num_parts: usize = 4;
//     // let part_size: usize = (num_entries as f32 / num_parts).round() as usize;
//     let part_size: usize = (num_entries / num_parts);
//
//     // if num_parts > 1 {
//     //     conn.execute(
//     //         "CREATE UNIQUE INDEX if not exists WikiLinks_page_id_page_links_key ON
//     //        WikiLinks(page_id, page_links)", (),
//     //     ).expect("Failed creating unique index");
//     // }
//
//     let mut total_inserted = 0;
//     for part in 1..=num_parts {
//         println!("Loading {part_size} entries into memory..");
//         let data = load_sql_part_set2(&pagelinks_sql, part_size, part);
//         // let data: InsertData = FxHashSet::default();
//
//         let already_inserted = data.len();
//         total_inserted += insert_into_sqlite(&mut conn, data, already_inserted);
//
//         if part == 1 {
//             let mut sp = Spinner::new(Spinners::Dots, "Creating unique index".into());
//
//             conn.execute(
//                 "CREATE UNIQUE INDEX if not exists WikiLinks_page_id_page_links_key ON
//            WikiLink(page_id, page_link)", (),
//             ).expect("Failed creating unique index");
//
//             sp.stop();
//         }
//     }
//
//     // println!("Skipped {already_inserted} entries");
//
//
//     // let mut data = iterate_sql_insertions::<PageLink>(&pagelinks_sql);
//     // total_inserted += insert_into_sqlite(&mut conn, data.into_iter(),
//     //                                      num_entries);
//
//     create_indices_post_setup(&conn);
//
//     // can do the unique at the end because if you insert it all in one go, there are no duplicates
//     // if num_parts == 1 {
//     //     conn.execute(
//     //         "CREATE UNIQUE INDEX if not exists WikiLinks_page_id_page_links_key ON
//     //        WikiLinks(page_id, page_links)", (),
//     //     ).expect("Failed creating unique index");
//     // }
//
//     println!("Finished writing to database [!] :)");
//
//     let elapsed = t1.elapsed();
//     let total_secs = elapsed.as_secs() as f64
//         + elapsed.subsec_nanos() as f64 * 1e-9;
//     println!("Elapsed: {:#?} \nSpeed: {} rows/s", elapsed, ((total_inserted as f64 / total_secs) as u64).to_formatted_string(&Locale::de));
//
// //
// // // conn.execute(
// // //     "CREATE UNIQUE INDEX idx_links_id\
// // //         ON de_page_links(id);",
// // //     (),
// // // ).expect("Failed creating index");
// //
// //
// //
// // let statement = "INSERT INTO de_page_links VALUES (?, ?)".to_string();
// //
// // let mut stmt = conn.prepare_cached(statement.as_str()).unwrap();
// // // for pagelink in iterate_sql_insertions::<PageLink>(&pagelinks_sql).into_iter() {
// // //
// // // }
// //
// //
// // for link in data {
// //     stmt.execute((link.from.into_inner(), link.title.into_inner())).unwrap();
// //     bar.inc(1);
// // }
// //
//
// // for i in 0..100 {
// //     let f = File::open("/home/gareth/dev/Rust/WikiGame/test.txt").unwrap();
// //     let reader = BufReader::new(f);
// //     // let mut c = 0;
// //     for line in reader.lines() {
// //         // c += 1;
// //         // bar.inc(1);
// //         let line = line.unwrap();
// //         if line.contains("7709810") {
// //             dbg!("Omegalil");
// //             // dbg!(&line);
// //             break;
// //         }
// //     }
// // }
// //
// // // dbg!(c);
// }

pub fn load_link_to_map_db(
    db_path: impl AsRef<Path>,
) -> HashMap<PageId, Vec<PageId>, FxBuildHasher> {
    load_link_to_map_db_limit(db_path, vec![], false)
}

pub fn load_link_to_map_db_wiki(
    db_path: impl AsRef<Path>,
) -> HashMap<PageId, Vec<PageId>, FxBuildHasher> {
    load_link_to_map_db_limit(db_path, vec![], false)
}

/// Returns a map of pageid to all the pageids it links to
/// ### Args:
/// - If num_load_opt is None, load all entries
/// - If num_load_opt is Some(n), load the links of the n pages with the most links
/// - If num_load_opt is Some(0), load no entries
pub fn get_cache(path: impl AsRef<Path>, num_load_opt: Option<usize>, incoming: bool) -> DBCache {
    let cached_entries: Vec<PageId> = match num_load_opt {
        None => vec![],
        Some(num_load) => {
            if num_load == 0 {
                return FxHashMap::default();
            } else {
                select_link_count_groupby(num_load, &path, "WikiLink.page_id")
                    .into_iter()
                    .map(|(pid, _)| PageId(pid as u32))
                    .collect()
            }
        }
    };
    info!("Loaded the links of the top {num_load_opt:?} most links entries to cache");

    load_link_to_map_db_limit(path, cached_entries, incoming)
}

/// Returns a map of pageid to all the pageids it links to (or links from it)
/// ### Args:
/// - path: Database path
/// - select: If empty, load all entries. Otherwise, load only entries for these page IDs
/// - incoming: If true, map target -> sources (incoming links). If false, map source -> targets (outgoing links)
pub fn load_link_to_map_db_limit(
    path: impl AsRef<Path>, 
    select: Vec<PageId>,
    incoming: bool
) -> DBCache {
    let conn = Connection::open(path).unwrap();
    let mut limit_str = String::new();
    
    let (key_col, value_col) = if incoming {
        ("page_link", "page_id")  // target -> source
    } else {
        ("page_id", "page_link")  // source -> target
    };
    
    if !select.is_empty() {
        limit_str = format!(
            "where {} in ({})",
            key_col,
            select
                .iter()
                .map(|p| p.0.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
    }
    
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {}, {} FROM WikiLink {limit_str}",
            key_col, value_col
        ))
        .unwrap();

    let rows = stmt
        .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
        .unwrap();

    let len = 136_343_429;
    let bar = ProgressBarBuilder::new()
        .with_name("Loading cache")
        .with_length(len as u64)
        .build();

    let mut map = FxHashMap::default();
    for row in rows {
        let (key, value): (u32, u32) = row.unwrap();
        map.entry(PageId(key))
            .or_insert_with(Vec::new)
            .push(PageId(value));
        bar.inc(1);
    }

    bar.finish();
    map
}

// TODO: test with test db

/// Returns pageid of all pages that are linked by id (all outgoing links)
pub fn get_links_of_id(conn: &Connection, id: &PageId) -> Vec<PageId> {
    let mut stmt = conn
        .prepare("SELECT page_link FROM WikiLink WHERE page_id = ?1")
        .unwrap();

    let rows = stmt.query_map([id.0], |row| row.get(0)).unwrap();

    let mut links = vec![];
    for row in rows {
        links.push(PageId(row.unwrap()))
    }

    links
}

// Returns pageid of all pages that link to id (all incoming links)
pub fn get_incoming_links_of_id(conn: &Connection, id: &PageId) -> Vec<PageId> {
    let mut stmt = conn
        .prepare("SELECT page_id FROM WikiLink WHERE page_link = ?1")
        .unwrap();

    let rows = stmt.query_map([id.0], |row| row.get(0)).unwrap();

    let mut links = vec![];
    for row in rows {
        links.push(PageId(row.unwrap()))
    }

    links
}

pub fn get_links_of_ids(
    conn: &Connection,
    ids: Vec<PageId>,
    incoming: bool,
) -> Vec<(PageId, PageId)> {
    let what = if incoming { "page_id" } else { "page_link" };
    let where_str = if incoming { "page_link" } else { "page_id" };
    let ids_str = ids.iter().map(|pid| pid.0.to_string()).join(",");

    let mut stmt = conn
        .prepare(&format!(
            "SELECT page_id, page_link FROM WikiLink WHERE {where_str} in ({ids_str})"
        ))
        .unwrap();

    let rows = stmt
        .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
        .unwrap();

    let mut links = vec![];
    for row in rows {
        let (pid, pid_link) = row.unwrap();
        links.push((PageId(pid), PageId(pid_link)))
    }

    links
}

// dewiki
// 146_358_594
// redirects:  1_862_077
// linking to redirects: 7_084_425
pub fn count_redirects(db_path: &str) {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn.prepare("SELECT page_link FROM WikiLink").unwrap();

    let rows = stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap();

    let redirects_set: FxHashSet<u32> = load_wiki_pages(db_path)
        .iter()
        .filter(|wp| wp.is_redirect)
        .map(|wp| wp.id)
        .collect();

    let mut redirects = 0;
    let bar = default_bar(u32::MAX as u64);
    for row in rows {
        bar.inc(1);
        let res: u32 = row.unwrap();
        if redirects_set.contains(&res) {
            redirects += 1;
        }
    }
    dbg!(&bar.position());
    bar.finish();

    dbg!(&redirects);
}

pub fn count_duplicates(db_path: &str) {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT page_id, page_link, COUNT(*) FROM WikiLink \
            GROUP BY page_id, page_link HAVING COUNT(*) > 1",
        )
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get(0).unwrap(),
                row.get(1).unwrap(),
                row.get(2).unwrap(),
            ))
        })
        .unwrap();

    for row in rows {
        let res: (u32, u32, u32) = row.unwrap();
        dbg!(&res);
    }
}
