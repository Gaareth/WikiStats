use core::num;
use std::cmp::{max, max_by, min_by, Ordering};
use std::collections::HashSet;
use std::fmt::Debug;
use std::future::Future;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{self, Instant};
use std::{fs, thread};

use chrono::format;
use crossbeam::channel::{unbounded, Sender};
use crossbeam::queue::ArrayQueue;
use futures::{pin_mut, StreamExt};
use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use indicatif::MultiProgress;
use log::{debug, info};
use parse_mediawiki_sql::field_types::PageId;
use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{de, Deserialize, Serialize};
use tokio::join;
use tokio::sync::mpsc;

use crate::calc::{bfs, bfs_bidirectional, bfs_undirected, build_path, SpBiStream};
use crate::download::ALL_DB_TABLES;
use crate::sqlite::page_links::{get_cache, load_link_to_map_db};
use crate::sqlite::title_id_conv::{get_random_page, load_rows_from_page, page_id_to_title};
use crate::sqlite::{join_db_wiki_path, title_id_conv, wiki};
use crate::utils::{bar_color, default_bar, default_bar_unknown, ProgressBarBuilder};
use crate::web::find_smallest_wikis;
use crate::{AvgDepthHistogram, AvgDepthStat, DepthHistogram};
use std::collections::HashMap;

static DB_STATS: &str =
    "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240301/es_database2.sqlite";

// static wikis: [&str; 3] = ["dewiki", "ruwiki", "frwiki"];
// static WIKIS: [&str; 2] = ["dewiki", "ruwiki"];
// static WIKIS: [&str; 1] = ["eswiki"];

#[derive(Clone)]
pub struct WikiIdent {
    wiki_name: String,
    db_path: PathBuf,
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

static GLOBAL: &str = "global";

fn count_from(table_name: &str, db_path: impl AsRef<Path>, where_str: &str) -> u64 {
    let conn = Connection::open(db_path).unwrap();
    let stmt = format!("select count(*) from {table_name} {where_str}");
    conn.query_row(&stmt, [], |row| row.get(0)).unwrap()
}

fn get_dead_pages(wiki_ident: WikiIdent) -> Vec<Page> {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let res = query_page(
        "select * from WikiPage where page_id not in (select page_id from WikiLink);",
        &db_path,
        name.clone(),
    );
    println!("DONE dead pages {:?}: {name:?}", t1.elapsed());
    res
}

pub fn get_orphan_pages(wiki_ident: WikiIdent) -> Vec<Page> {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let res = query_page(
        "select * from WikiPage where page_id not in (select page_link from WikiLink);",
        &db_path,
        name.clone(),
    );
    println!("DONE root pages {:?}: {name:?}", t1.elapsed());
    res
}

fn get_num_dead_pages(wiki_ident: WikiIdent) -> u64 {
    let t1 = Instant::now();

    let stmt =
        "select count(page_id) from WikiPage where page_id not in (select page_id from WikiLink);"
            .to_string();

    let res = query_count(&stmt, &wiki_ident.db_path);
    println!(
        "DONE num dead pages {:?}: {:?}",
        t1.elapsed(),
        wiki_ident.wiki_name
    );

    res
}

fn get_num_orphan_pages(wiki_ident: WikiIdent) -> u64 {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let stmt = "select count(page_id) from WikiPage where page_id not in (select page_link from WikiLink);".to_string();

    let res = query_count(&stmt, &db_path);
    println!("DONE num root pages {:?}: {name:?}", t1.elapsed());

    res
}

fn get_num_dead_orphan_pages(wiki_ident: WikiIdent) -> u64 {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let stmt = "select count(page_id) from WikiPage \
            where page_id not in (select page_link from WikiLink) AND \
            page_id not in (select page_id from WikiLink);"
        .to_string();

    let res = query_count(&stmt, &db_path);
    println!("DONE num root pages {:?}: {name:?}", t1.elapsed());

    res
}

fn get_dead_orphan_pages(wiki_ident: WikiIdent) -> Vec<Page> {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let stmt = "select * from WikiPage \
            where page_id not in (select page_link from WikiLink) AND \
            page_id not in (select page_id from WikiLink) LIMIT 20;"
        .to_string();

    let res = query_page(&stmt, &db_path, name.clone());
    println!("DONE dead orphan pages {:?}: {name:?}", t1.elapsed());

    res
}

fn get_num_linked_redirects(wiki_ident: WikiIdent) -> u64 {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let stmt = "select count(*) from WikiLink where page_link in (select page_id from WikiPage where is_redirect = 1);".to_string();

    let res = query_count(&stmt, &db_path);
    println!("DONE num linked redirects {:?}: {name:?}", t1.elapsed());

    res
}

// fn count_from(table_name: &str, wiki_name: Option<&str>) -> (u64, String) {
//     dbg!(&wiki_name);
//     let mut conn = Connection::open(DB_STATS).unwrap();
//     if let Some(wiki_name) = wiki_name {
//         let stmt = format!("select count(*) from {table_name} where wiki_name = ?1");
//         (conn.query_row(&stmt, [wiki_name], |row| row.get(0)).unwrap(), wiki_name.to_string())
//     } else {
//         let stmt = format!("select count(*) from {table_name}");
//         (conn.query_row(&stmt, [], |row| row.get(0)).unwrap(), GLOBAL.to_string())
//     }
//
//     // return 2;
// }

// async fn make_stat_record<T: Debug, F: Future<Output=(T, String)>>(func: impl FnOnce(String) -> F)
//                                                                                     -> HashMap<String, T> {
//     let mut tasks = vec![];
//     let mut record = HashMap::new();
//     for wiki in wikis {
//         // func(wiki.to_string()).await;
//         // tasks.push(tokio::spawn(func(wiki.to_string())));
//     }
//
//     // dbg!(&tasks);
//     // for task in tasks {
//     //     let (res, wname) = task.await.unwrap();
//     //     record.insert(wname, res);
//     // }
//
//     record
// }

async fn make_stat_record_async<T, Fut, F>(
    wikis: Vec<WikiIdent>,
    func: F,
    global_func: fn(&mut StatRecord<T>) -> (),
    existing_stat_record: Option<StatRecord<T>>,
) -> StatRecord<T>
where
    T: Debug + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    F: Fn(WikiIdent) -> Fut + Send + Sync + 'static + Clone,
{
    let mut tasks = vec![];
    let mut record = existing_stat_record.unwrap_or_else(FxHashMap::default);
    let completed_wikis: FxHashSet<String> = record.keys().cloned().collect::<FxHashSet<String>>();

    for wiki in wikis {
        if !completed_wikis.contains(&wiki.wiki_name) {
            let func = func.clone();
            tasks.push(tokio::spawn(async move {
                (func(wiki.clone()).await, wiki.wiki_name)
            }));
        }
    }

    for task in tasks {
        let (res, wname) = task.await.unwrap();
        record.insert(wname, res);
    }
    global_func(&mut record);

    record
}

// async fn make_stat_record_async<T: Debug + Send + 'static, Fut: Future<Output=T> + Send + 'static>(
//     wikis: Vec<WikiIdent>,
//     func: fn(WikiIdent) -> Fut,
//     global_func: fn(&mut StatRecord<T>) -> (),
// ) -> StatRecord<T> {
//     let mut tasks = vec![];
//     let mut record = FxHashMap::default();
//
//
//     for wiki in wikis {
//         tasks.push(tokio::spawn(async move {
//             (func(wiki.clone()).await, wiki.wiki_name)
//         }));
//     }
//
//     for task in tasks {
//         let (res, wname) = task.await.unwrap();
//         record.insert(wname, res);
//     }
//     global_func(&mut record);
//
//     record
// }

async fn make_stat_record_seq<T, F>(
    wikis: Vec<WikiIdent>,
    func: F,
    global_func: fn(&mut StatRecord<T>) -> (),
    existing_stat_record: Option<StatRecord<T>>,
) -> StatRecord<T>
where
    T: Debug + Send + 'static,
    F: Fn(WikiIdent) -> T,
{
    let mut record = existing_stat_record.unwrap_or_else(FxHashMap::default);
    let completed_wikis: FxHashSet<String> = record.keys().cloned().collect::<FxHashSet<String>>();

    for wiki in wikis {
        if !completed_wikis.contains(&wiki.wiki_name) {
            record.insert(wiki.wiki_name.clone(), func(wiki.clone()));
        }
    }

    global_func(&mut record);

    record
}

async fn make_stat_record<T: Debug + Send + 'static>(
    wikis: Vec<WikiIdent>,
    func: fn(WikiIdent) -> T,
    global_func: fn(&mut StatRecord<T>) -> (),
    existing_stat_record: Option<StatRecord<T>>,
) -> StatRecord<T> {
    let mut tids = vec![];
    let mut record = existing_stat_record.unwrap_or_else(FxHashMap::default);
    let completed_wikis: FxHashSet<String> = record.keys().cloned().collect::<FxHashSet<String>>();

    for wiki in wikis {
        if !completed_wikis.contains(&wiki.wiki_name) {
            tids.push(thread::spawn(move || {
                (func(wiki.clone()), wiki.wiki_name.clone())
            }));
        }
    }

    tids.into_iter().for_each(|th| {
        let (res, wname) = th.join().expect("can't join thread");
        record.insert(wname, res);
    });

    global_func(&mut record);

    record
}

type StatRecord<T> = FxHashMap<String, T>;

type WikiName = String;
type PageTitle = String;

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

    pub wiki_sizes: WikiSizes,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct WikiSize {
    name: String,
    compressed_total_size: u64,
    compressed_selected_tables_size: u64,
    decompressed_size: Option<u64>,
    processed_size: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct WikiSizes {
    pub sizes: Vec<WikiSize>,
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
struct Page {
    page_title: PageTitle,
    page_id: u64,
    wiki_name: WikiName,
}

pub async fn create_stats(
    path: impl AsRef<Path>,
    wikis: Vec<String>,
    database_path: impl Into<PathBuf>,
    dump_date: impl Into<String>,
) {
    fn num_pages_stat(wiki: WikiIdent) -> u64 {
        count_from("WikiPage", &wiki.db_path, "")
    }
    fn num_redirects_stat(wiki: WikiIdent) -> u64 {
        count_from("WikiPage", &wiki.db_path, "WHERE is_redirect = 1")
    }

    fn num_links_stat(wiki: WikiIdent) -> u64 {
        count_from("WikiLink", &wiki.db_path, "")
    }

    fn global_adder(record: &mut StatRecord<u64>) {
        let v = record.iter().fold(0, |acc, (_, value)| acc + value);
        record.insert(GLOBAL.to_string(), v);
    }

    let dump_date = dump_date.into();
    let path = path.as_ref();
    let stats: Option<Stats> = try_load_stats(path);

    // Extract needed fields from stats before moving it
    let num_pages_prev = stats.as_ref().map(|s| s.num_pages.clone());
    let num_redirects_prev = stats.as_ref().map(|s| s.num_redirects.clone());
    let num_links_prev = stats.as_ref().map(|s| s.num_links.clone());
    let most_linked_prev = stats.as_ref().map(|s| s.most_linked.clone());
    let most_links_prev = stats.as_ref().map(|s| s.most_links.clone());
    let longest_name_prev = stats.as_ref().map(|s| s.longest_name.clone());
    let longest_name_no_redirect_prev = stats.as_ref().map(|s| s.longest_name_no_redirect.clone());
    let num_dead_pages_prev = stats.as_ref().map(|s| s.num_dead_pages.clone());
    let num_orphan_pages_prev = stats.as_ref().map(|s| s.num_orphan_pages.clone());
    let num_dead_orphan_pages_prev = stats.as_ref().map(|s| s.num_dead_orphan_pages.clone());
    let num_linked_redirects_prev = stats.as_ref().map(|s| s.num_linked_redirects.clone());

    let database_path = database_path.into();
    let wiki_idents: Vec<WikiIdent> = create_wiki_idents(&database_path, wikis.clone());

    let pages_stat_future = make_stat_record(
        wiki_idents.clone(),
        num_pages_stat,
        global_adder,
        num_pages_prev,
    );

    let redirects_stat_future = make_stat_record(
        wiki_idents.clone(),
        num_redirects_stat,
        global_adder,
        num_redirects_prev,
    );

    let link_stat_future = make_stat_record(
        wiki_idents.clone(),
        num_links_stat,
        global_adder,
        num_links_prev,
    );

    fn top_ten_linked(wiki_ident: WikiIdent) -> Vec<LinkCount> {
        let t1 = Instant::now();
        let db_path = wiki_ident.db_path;
        let name = wiki_ident.wiki_name;

        println!("Top linked: {name:?}");
        let conn = Connection::open(&db_path).unwrap();
        let res = select_link_count_groupby(10, &db_path, "WikiLink.page_link")
            .into_iter()
            .map(|(page_id, count)| {
                let page_title = page_id_to_title(&PageId(page_id as u32), &conn)
                    .unwrap_or_else(|| panic!("Failed retrieving page title from id {page_id}"))
                    .0;
                LinkCount {
                    page_title,
                    page_id,
                    wiki_name: name.to_string(),
                    count,
                }
            })
            .collect();
        println!("DONE. {:?} Top linked: {name:?}", t1.elapsed());

        res
    }

    fn top_ten_links(wiki_ident: WikiIdent) -> Vec<LinkCount> {
        let t1 = Instant::now();
        let name = wiki_ident.wiki_name;
        let db_path = wiki_ident.db_path;

        println!("Top links: {name:?}");
        let conn = Connection::open(&db_path).unwrap();

        let res = select_link_count_groupby(10, &db_path, "WikiLink.page_id")
            .into_iter()
            .map(|(page_id, count)| {
                let page_title = page_id_to_title(&PageId(page_id as u32), &conn).unwrap().0;
                LinkCount {
                    page_title,
                    page_id,
                    wiki_name: name.to_string(),
                    count,
                }
            })
            .collect();
        println!("DONE. {:?} Top links: {name:?}", t1.elapsed());

        res
    }

    fn global_max_list(record: &mut StatRecord<Vec<LinkCount>>) {
        let mut global_list = vec![];
        for list in record.clone().into_values() {
            global_list.extend(list);
        }
        global_list.sort_by(|c1, c2| c2.count.cmp(&c1.count)); // descending
        record.insert(GLOBAL.to_string(), global_list);
    }

    fn global_max<T: Clone + Debug, F: FnOnce(&T, &T) -> Ordering + Copy>(
        record: &mut StatRecord<T>,
        cmp_fn: F,
    ) {
        let ((_, max_element), _) = max_min_value_record(record, cmp_fn);
        record.insert(GLOBAL.to_string(), max_element.clone());
    }

    let most_linked_future = make_stat_record(
        wiki_idents.clone(),
        top_ten_linked,
        global_max_list,
        most_linked_prev,
    );
    // dbg!(&most_linked_future.await);
    let most_links_future = make_stat_record(
        wiki_idents.clone(),
        top_ten_links,
        global_max_list,
        most_links_prev,
    );

    fn global_longest_name(record: &mut StatRecord<Page>) {
        global_max(record, |p1, p2| {
            p1.page_title.len().cmp(&p2.page_title.len())
        })
    }

    let longest_name_future = make_stat_record(
        wiki_idents.clone(),
        |w| longest_name(w, true),
        global_longest_name,
        longest_name_prev,
    );

    let longest_name_no_redirect_future = make_stat_record(
        wiki_idents.clone(),
        |w| longest_name(w, false),
        global_longest_name,
        longest_name_no_redirect_prev,
    );

    let num_dead_pages = make_stat_record(
        wiki_idents.clone(),
        get_num_dead_pages,
        global_adder,
        num_dead_pages_prev,
    );

    let num_orphan_pages = make_stat_record(
        wiki_idents.clone(),
        get_num_orphan_pages,
        global_adder,
        num_orphan_pages_prev,
    );
    let num_dead_orphan_pages = make_stat_record(
        wiki_idents.clone(),
        get_num_dead_orphan_pages,
        global_adder,
        num_dead_orphan_pages_prev,
    );
    let num_linked_redirects = make_stat_record(
        wiki_idents.clone(),
        get_num_linked_redirects,
        global_adder,
        num_linked_redirects_prev,
    );

    let t1 = Instant::now();

    let (
        num_pages,
        num_redirects,
        num_links,
        most_linked,
        most_links,
        longest_name,
        longest_name_no_redirect,
        num_dead_pages,
        num_orphan_pages,
        num_dead_orphan_pages,
        num_linked_redirects,
    ) = join!(
        tokio::spawn(pages_stat_future),
        tokio::spawn(redirects_stat_future),
        tokio::spawn(link_stat_future),
        tokio::spawn(most_linked_future),
        tokio::spawn(most_links_future),
        tokio::spawn(longest_name_future),
        tokio::spawn(longest_name_no_redirect_future),
        tokio::spawn(num_dead_pages),
        tokio::spawn(num_orphan_pages),
        tokio::spawn(num_dead_orphan_pages),
        tokio::spawn(num_linked_redirects)
    );

    let (max_num_pages, min_num_pages) =
        max_min_value_record(num_pages.as_ref().unwrap(), |a, b| a.cmp(b));

    let (max_num_links, min_num_links) =
        max_min_value_record(num_links.as_ref().unwrap(), |a, b| a.cmp(b));

    // let bfs_sample_stats = make_stat_record(wikis.clone(),
    //                                         |n: String| sample_bfs_stats(n, 100, 100), global_ignore);
    //
    // let bi_bfs_sample_stats = make_stat_record_async(wikis.clone(), async_bibfs_name_wrapper, global_ignore);

    let wiki_sizes = get_wiki_sizes(
        database_path.parent().unwrap(),
        Some(dump_date.clone()),
        &ALL_DB_TABLES,
    )
    .await;

    let time_taken: time::Duration = t1.elapsed();

    let stats = Stats {
        num_pages: num_pages.unwrap(),
        num_redirects: num_redirects.unwrap(),
        num_links: num_links.unwrap(),
        num_linked_redirects: num_linked_redirects.unwrap(),

        most_linked: most_linked.unwrap(),
        most_links: most_links.unwrap(),

        longest_name: longest_name.unwrap(),
        longest_name_no_redirect: longest_name_no_redirect.unwrap(),

        num_dead_pages: num_dead_pages.unwrap(),
        num_orphan_pages: num_orphan_pages.unwrap(),

        num_dead_orphan_pages: num_dead_orphan_pages.unwrap(),

        max_num_pages,
        min_num_pages,
        max_num_links,
        min_num_links,

        created_at: chrono::Utc::now().timestamp(),
        dump_date,
        wikis,
        seconds_taken: time_taken.as_secs(),

        // bfs_sample_stats: Some(bfs_sample_stats.await),
        // bi_bfs_sample_stats: Some(bi_bfs_sample_stats.await),
        bfs_sample_stats: None,
        bi_bfs_sample_stats: None,
        wiki_sizes,
    };

    save_stats(&stats, path);
    println!(
        "Done generating stats. Total time elapsed: {:?}",
        time_taken
    );
}

async fn get_wiki_sizes(
    base_path: impl AsRef<Path>,
    dump_date: Option<String>,
    tables: &[&str],
) -> WikiSizes {
    let web_wiki_sizes = find_smallest_wikis(dump_date, tables)
        .await
        .expect("Failed finding smallest wikis");

    let download_path = base_path.as_ref().join("downloads");
    let decompressed_sizes: Vec<(String, u64)> = fs::read_dir(&download_path)
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
        .collect();

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

    let wiki_sizes = WikiSizes {
        sizes: web_wiki_sizes
            .into_iter()
            .map(|ws| WikiSize {
                name: ws.name.clone(),
                compressed_total_size: ws.total_size,
                compressed_selected_tables_size: ws.selected_tables_size,
                decompressed_size: decompressed_size_map.get(&ws.name).cloned(),
                processed_size: processed_sizes.get(&ws.name).cloned(),
            })
            .collect(),
        tables: tables.iter().map(|s| s.to_string()).collect(),
    };

    wiki_sizes
}

fn try_load_stats(path: &Path) -> Option<Stats> {
    if path.exists() {
        Some(load_stats(path))
    } else {
        None
    }
}

fn load_stats(path: &Path) -> Stats {
    serde_json::from_str(
        &fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed loading stats file: {path:?}")),
    )
    .unwrap_or_else(|_| panic!("Failed deserializing stats file: {path:?}"))
}

fn global_ignore<T>(_: &mut StatRecord<T>) {}

/// Calculates bfs stats and adds or overwrites stats to/of existing json file
pub async fn add_sample_bfs_stats(
    path: impl AsRef<Path>,
    db_path: impl Into<PathBuf>,
    wikis: Vec<String>,
    sample_size: usize,
    num_threads: usize,
    cache_max_size: Option<usize>,
    always: bool,
) {
    let database_path = db_path.into();
    let wiki_idents: Vec<WikiIdent> = create_wiki_idents(&database_path, wikis);
    let path: &Path = path.as_ref();

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

fn save_stats(stats: &Stats, path: impl AsRef<Path>) {
    let json = serde_json::to_string_pretty(&stats).unwrap();
    info!("Written to {:?}", path.as_ref());
    fs::write(&path, json).expect(&format!(
        "Failed writing stats to file {}",
        path.as_ref().display()
    ));
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct MaxMinAvg<T, C: PartialOrd> {
    pub min: (T, C),
    pub max: (T, C),
    pub avg: f64,
}

impl<T, C> MaxMinAvg<T, C>
where
    C: PartialOrd + AddAssign + Clone,
    T: Clone,
    f64: From<C>,
{
    pub fn new(key: T, value: C) -> Self {
        MaxMinAvg {
            avg: value.clone().into(),
            min: (key.clone(), value.clone()),
            max: (key, value),
        }
    }

    pub fn add(&mut self, key: T, value: C) {
        self.avg += f64::from(value.clone());
        self.avg /= 2.0;

        if value < self.min.1 {
            self.min.0 = key;
            self.min.1 = value;
        } else if value > self.max.1 {
            self.max.0 = key;
            self.max.1 = value;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct BfsSample {
    pub sample_size: u32,

    // max, min, avg of deepest bfs path found. storing the (start, end) page titles with the depth
    pub deep_stat: MaxMinAvg<(PageTitle, PageTitle), u32>,
    pub path_depth_map: FxHashMap<u32, Vec<PageTitle>>,

    // how many page were visited starting from this page
    pub visit_stat: MaxMinAvg<PageTitle, u32>,
    pub num_visited_map: FxHashMap<u32, PageTitle>,

    pub avg_depth_histogram: AvgDepthHistogram,
    pub seconds_taken: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct BiBfsSample {
    pub sample_size: u32,
    pub longest_path_stat: MaxMinAvg<(PageTitle, PageTitle), u32>,
    // pub visit_stat: MaxMinAvg<PageTitle, u32>,
    pub path_length_histogram: DepthHistogram,
    pub seconds_taken: u64,
}

pub async fn sample_bidirectional_bfs_stats(
    wiki_ident: WikiIdent,
    sample_size: usize,
    mut num_threads: usize,
) -> BiBfsSample {
    let t1 = Instant::now();

    let db_path = wiki_ident.db_path;
    let wiki_name = wiki_ident.wiki_name;

    // let sample_size = 1067;

    if num_threads > sample_size {
        num_threads = sample_size;
    }

    println!("Sample size: {sample_size}");

    let mut longest_path_stat: Option<MaxMinAvg<(PageTitle, PageTitle), u32>> = None;
    let mut path_length_histogram: DepthHistogram = FxHashMap::default();

    let pid_queue: Arc<ArrayQueue<(PageId, PageId)>> = Arc::new(ArrayQueue::new(sample_size));
    for (start_page, end_page) in get_random_page(&db_path, sample_size as u32)
        .iter()
        .zip(get_random_page(&db_path, sample_size as u32))
    {
        if *start_page == end_page {
            continue;
        }

        pid_queue
            .push((PageId(start_page.id), PageId(end_page.id)))
            .unwrap()
    }

    let m = Arc::new(MultiProgress::new());
    let bar = Arc::new(Mutex::new(m.add(default_bar(pid_queue.len() as u64))));
    let bar2 = m.add(bar_color("magenta", pid_queue.len() as u64));

    // let num_threads: usize = 500;
    println!("num_threads: {num_threads}");

    let (sender, mut receiver) = mpsc::channel::<((PageId, PageId), SpBiStream)>(pid_queue.len());

    for tid in 0..num_threads {
        let s = sender.clone();
        // println!("started thread {tid}");
        let pid_queue = pid_queue.clone();
        let bar = bar.clone();
        let m = m.clone();
        let db_path = db_path.clone();

        tokio::spawn(async move {
            let t1 = Instant::now();

            while !pid_queue.is_empty() {
                let (start_link_id, end_link_id) = pid_queue.pop().unwrap();
                // println!("[{tid}]: {:?}", start_link_id);

                let stream = bfs_bidirectional(start_link_id, end_link_id, db_path.clone()).await;
                pin_mut!(stream);
                let mut result = stream.next().await;
                while let Some(v) = stream.next().await {
                    result = Some(v);
                }

                s.send(((start_link_id, end_link_id), result.unwrap()))
                    .await
                    .unwrap();

                bar.lock().unwrap().inc(1);
            }
            // m.println(format!("[{tid}]: Done: {:?}", t1.elapsed())).unwrap();
        });
    }

    drop(sender);

    let id_title_map = title_id_conv::load_id_title_map(&db_path);

    while let Some(((start_link_id, end_link_id), result)) = receiver.recv().await {
        // dbg!(&pid);
        let path_length = result
            .paths
            .and_then(|paths| paths.into_iter().next())
            .map(|p| p.len())
            .unwrap_or(0) as u32;

        let start_title = id_title_map.get(&start_link_id).unwrap().clone().0;
        let end_title = id_title_map.get(&end_link_id).unwrap().clone().0;

        if let Some(ref mut longest_path_stat) = longest_path_stat {
            longest_path_stat.add((start_title, end_title), path_length);
        } else {
            longest_path_stat = Some(MaxMinAvg::new((start_title, end_title), path_length));
        }

        path_length_histogram
            .entry(path_length)
            .and_modify(|l| *l += 1)
            .or_insert(1);

        bar2.inc(1);
    }

    bar2.finish();
    bar.lock().unwrap().finish();

    let time_taken = t1.elapsed();
    println!(
        "DONE: Bidirectional BFS SAMPLE after {:?} with sample_size = {sample_size}",
        time_taken
    );

    BiBfsSample {
        sample_size: sample_size as u32,
        longest_path_stat: longest_path_stat.unwrap(),
        path_length_histogram,
        seconds_taken: time_taken.as_secs(),
    }
}

pub fn find_wcc(wiki_ident: WikiIdent) {
    let db_path = wiki_ident.clone().db_path;

    let all_pages = load_rows_from_page(&db_path)
        .into_iter()
        .map(|p| p.0 .0)
        .collect::<FxHashSet<_>>();
    let redirects = query_page(
        "SELECT * FROM WikiPage WHERE is_redirect = 1;",
        &db_path,
        wiki_ident.wiki_name.clone(),
    )
    .into_iter()
    .map(|p| p.page_id as u32)
    .collect::<FxHashSet<_>>();

    let mut components: Vec<FxHashSet<u32>> = Vec::new();
    let mut visited: FxHashSet<u32> = FxHashSet::default();
    let cache = get_cache(&db_path, None, false);
    let incoming_cache = get_cache(&db_path, None, true);

    let bar = ProgressBarBuilder::new()
        .with_name("Finding WCC")
        .with_length(all_pages.len() as u64)
        .build();

    for page in all_pages {
        if !visited.contains(&page) {
            let connected_component: FxHashSet<u32> =
                bfs_undirected(&PageId(page), &cache, &incoming_cache, db_path.clone())
                    .into_iter()
                    .map(|pid| pid.0)
                    .collect();

            // bar.println(format!("Found WCC: {:?}", connected_component.len()));
            // bar.inc(connected_component.len() as u64);

            components.push(connected_component.clone());
            visited.extend(connected_component);
        }
        bar.inc(1);
    }
    info!("Num components: {}", components.len());

    // remove all redirects
    components.retain_mut(|c| {
        c.retain(|p| !redirects.contains(p));
        c.len() > 1
    });

    components.sort_by(|a, b| a.len().cmp(&b.len())); // ascending
    components.pop(); // remove last, so largest component
                      // todo: also remove Begriffsklärungsseiten

    info!("Num components after pruning: {}", components.len());

    let path = "components.json";
    let json = serde_json::to_string_pretty(&components).unwrap();
    fs::write(&path, json).expect(&format!("Failed writing stats to file {}", path));
}

pub fn find_scc(wiki_ident: WikiIdent) {
    todo!()
}

pub fn sample_bfs_stats(
    wiki_ident: WikiIdent,
    sample_size: usize,
    num_threads: usize,
    cache_max_size: Option<usize>,
) -> BfsSample {
    let t1 = Instant::now();

    let db_path = &wiki_ident.db_path;
    let wiki_name = wiki_ident.wiki_name;

    let cache = get_cache(&db_path, cache_max_size, false);
    // let cache = load_link_to_map_db(db_path);
    info!("Cache size: {}", cache.len());

    let arc_cache = Arc::new(cache.clone());

    let num_threads = num_threads.clamp(1, sample_size); // at least 1 thread, at most sample_size threads
    info!("Sample size: {sample_size}");
    info!("NUM_THREADS: {num_threads}");

    let mut deep_stat: Option<MaxMinAvg<(PageTitle, PageTitle), u32>> = None;
    let mut path_depth_map: FxHashMap<u32, Vec<PageTitle>> = FxHashMap::default();

    let mut visit_stat: Option<MaxMinAvg<PageTitle, u32>> = None;
    let mut num_visited_map: FxHashMap<u32, PageTitle> = FxHashMap::default();

    let mut depth_histograms: Vec<FxHashMap<u32, f64>> = Vec::new();

    let pid_queue: Arc<ArrayQueue<PageId>> = Arc::new(ArrayQueue::new(sample_size));
    for page in get_random_page(&db_path, sample_size as u32) {
        pid_queue.push(PageId(page.id)).unwrap()
    }

    let m = MultiProgress::new();

    let bfs_bar: Arc<Mutex<indicatif::ProgressBar>> = Arc::new(Mutex::new(
        m.add(
            ProgressBarBuilder::new()
                .with_name("BFS     ")
                .with_length(pid_queue.len() as u64)
                .build(),
        ),
    ));
    let receiver_bar = m.add(
        ProgressBarBuilder::new()
            .with_name("Receiver")
            .with_bar_color_fg("white")
            .with_bar_color_bg("magenta")
            .with_length(pid_queue.len() as u64)
            .build(),
    );

    let (s, r) = unbounded();

    thread::scope(|scope| {
        for tid in 0..num_threads {
            debug!("started thread {tid}");
            let thread_sender = s.clone();
            let pid_queue = &pid_queue;
            let cache = &arc_cache;
            let bar = &bfs_bar;
            let m = &m;
            let wiki_name: &String = &wiki_name;

            scope.spawn(move || {
                let t1 = Instant::now();

                while !pid_queue.is_empty() {
                    let start_link_id = pid_queue.pop().unwrap();
                    // println!("[{tid}]: {:?}", start_link_id);

                    let result = bfs(&start_link_id, None, None, cache, &db_path);

                    thread_sender.send((start_link_id, result)).unwrap();

                    // thread_sender.send("a".to_string()).unwrap();
                    bar.lock().unwrap().inc(1);
                }
                m.println(format!("[{tid}]: Done: {:?}", t1.elapsed()))
                    .unwrap();
            });
        }
        // drop og sender
        drop(s);

        // receiver thread
        scope.spawn(|| {
            let id_title_map = title_id_conv::load_id_title_map(&db_path);
            let num_pages = id_title_map.len();
            info!("Num pages in id_title_map: {num_pages}");

            while let Ok((pid, result)) = r.recv() {
                let start_page_title = id_title_map.get(&pid).unwrap().clone().0;
                let end_page_title = id_title_map.get(&result.deepest_id).unwrap().clone().0;

                num_visited_map.insert(result.num_visited, start_page_title.clone());

                let deepest_path: Vec<String> = build_path(&result.deepest_id, &result.prev_map)
                    .iter()
                    .map(|pid| id_title_map.get(&pid).unwrap().clone().0)
                    .collect();

                // i think the deepest_path includes the start page, so its length is len_deepest_sp + 1
                assert_eq!(
                    deepest_path.len() as u32,
                    result.len_deepest_sp + 1,
                    "Length of deepest path {} does not match len_deepest_sp {}",
                    deepest_path.len(),
                    result.len_deepest_sp + 1
                );
                path_depth_map.insert(deepest_path.len() as u32, deepest_path);

                let deepest_stat_key = (
                    start_page_title.clone(), // start
                    end_page_title.clone(),   // end
                );

                if let Some(ref mut deep_stat) = deep_stat {
                    deep_stat.add(deepest_stat_key, result.len_deepest_sp);
                } else {
                    deep_stat = Some(MaxMinAvg::new(deepest_stat_key, result.len_deepest_sp));
                }

                if let Some(ref mut visit_stat) = visit_stat {
                    visit_stat.add(start_page_title.clone(), result.num_visited);
                } else {
                    visit_stat = Some(MaxMinAvg::new(start_page_title.clone(), result.num_visited));
                }

                depth_histograms.push(
                    result
                        .depth_histogram
                        .iter()
                        .map(|(&depth, &count)| (depth, count as f64 / num_pages as f64)) // normalize by number of pages
                        .collect(),
                );

                receiver_bar.inc(1);
            }
        });
    });

    // dbg!(&depth_histograms);

    let avg_depth_histogram = average_histograms(&depth_histograms);

    let path = "/home/gareth/dev/WikiStats/igraph/hist/depth_histogram_bfs.json";
    let json = serde_json::to_string_pretty(&depth_histograms).unwrap();
    fs::write(&path, json).expect(&format!("Failed writing stats to file {}", path));

    let time_taken = t1.elapsed();
    info!(
        "DONE: BFS SAMPLE after {:?} with sample_size = {sample_size}",
        time_taken
    );

    BfsSample {
        sample_size: sample_size as u32,
        deep_stat: deep_stat.unwrap(),
        path_depth_map,
        visit_stat: visit_stat.unwrap(),
        num_visited_map,
        avg_depth_histogram: avg_depth_histogram,
        seconds_taken: time_taken.as_secs(),
    }
}

fn average_histograms(depth_histograms: &[FxHashMap<u32, f64>]) -> AvgDepthHistogram {
    // First, sum up all histograms
    let mut sum_hist: FxHashMap<u32, f64> = FxHashMap::default();
    let mut count_hist: FxHashMap<u32, u32> = FxHashMap::default();

    for hist in depth_histograms {
        for (&depth, &count) in hist {
            *sum_hist.entry(depth).or_insert(0.0) += count as f64;
            *count_hist.entry(depth).or_insert(0) += 1;
        }
    }

    let mut avg_depth_histogram: FxHashMap<u32, f64> = FxHashMap::default();

    // Calculate average
    for (&depth, &sum) in &sum_hist {
        let count = count_hist[&depth];
        avg_depth_histogram.insert(depth, (sum / count as f64));
    }

    // Calculate std deviation for each depth
    let mut avg_std_dev_hist: AvgDepthHistogram = FxHashMap::default();
    for (&depth, &avg) in &avg_depth_histogram {
        let mut sum_sq = 0.0;

        let mut n: i32 = 0;
        for hist in depth_histograms {
            if let Some(&count) = hist.get(&depth) {
                let diff = count as f64 - avg;
                sum_sq += diff * diff;
                n += 1;
            }
        }
        if n > 1 {
            avg_std_dev_hist.insert(
                depth,
                AvgDepthStat {
                    avg_occurences: avg,
                    std_dev: (sum_sq / (n as f64 - 1.0)).sqrt(),
                },
            );
        } else {
            avg_std_dev_hist.insert(
                depth,
                AvgDepthStat {
                    avg_occurences: avg,
                    std_dev: 0.0,
                },
            );
        }
    }
    avg_std_dev_hist
}

fn max_min_value_record<T: Clone + Debug, F: FnOnce(&T, &T) -> Ordering + Copy>(
    record: &StatRecord<T>,
    cmp_fn: F,
) -> ((WikiName, T), (WikiName, T)) {
    // dbg!(&record);
    let mut iter = record.iter().filter(|(wname, _)| wname.as_str() != GLOBAL);
    let mut max_element = iter.next().unwrap();
    let mut min_element = max_element;
    // let (mut max_element, mut max_value) = (max_element, max_value);

    // let mut max_value = r;

    for element in iter {
        max_element = max_by(element, max_element, |a, b| cmp_fn(a.1, b.1));
        min_element = min_by(element, min_element, |a, b| cmp_fn(a.1, b.1));
    }

    (
        (max_element.0.clone(), max_element.1.clone()),
        (min_element.0.clone(), min_element.1.clone()),
    )
}

fn longest_name(wiki_ident: WikiIdent, redirects: bool) -> Page {
    let t1 = Instant::now();
    let wiki_name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    // let where_wiki = wiki_name_opt.clone().map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'")).unwrap_or_default();
    let where_str = if !redirects {
        "WHERE is_redirect = 0"
    } else {
        ""
    };
    let stmt_str = format!("SELECT page_title, page_id FROM WikiPage {where_str} ORDER BY length(page_title) DESC LIMIT 1");

    let conn = Connection::open(db_path).unwrap();

    let (page_title, page_id) = conn
        .query_row(&stmt_str, [], |row| {
            Ok((row.get(0).unwrap(), row.get(1).unwrap()))
        })
        .unwrap();

    let page = Page {
        page_title,
        page_id,
        wiki_name: wiki_name.clone(),
    };
    println!("DONE longest name {:?}: {:?}", t1.elapsed(), wiki_name);

    page
}

fn query_count(stmt_str: &str, db_path: impl AsRef<Path>) -> u64 {
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare(stmt_str).unwrap();
    // dbg!(&stmt);

    stmt.query_row([], |row| Ok(row.get(0).unwrap())).unwrap()
}

fn query_page(stmt_str: &str, db_path: impl AsRef<Path>, wiki_name: WikiName) -> Vec<Page> {
    let conn = Connection::open(db_path).unwrap();
    // let stmt_str = format!("SELECT {group_by}, WikiLink.wiki_name FROM WikiLink \
    //     GROUP BY {group_by} HAVING {having_str}");
    dbg!(&stmt_str);
    let mut stmt = conn.prepare(stmt_str).unwrap();

    let rows = stmt
        .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
        .unwrap();

    let mut res = vec![];
    let bar = default_bar_unknown();

    for row in rows {
        let (page_id, page_title): (u64, String) = row.unwrap();
        bar.inc(1);

        // let page_title = String::new();
        res.push(Page {
            page_title,
            page_id,
            wiki_name: wiki_name.to_string(),
        });
    }
    bar.finish();
    res
}

// fn links_groupby_having(wiki_name: Option<String>, group_by: &str, having_str: &str) -> Vec<Page> {
//     let conn = Connection::open(db_path).unwrap();
//     let stmt_str = format!("SELECT {group_by}, WikiLink.wiki_name FROM WikiLink \
//         GROUP BY {group_by} HAVING {having_str}");
//     dbg!(&stmt_str);
//     let mut stmt = conn.prepare(&stmt_str).unwrap();
//
//     let rows = stmt.query_map(
//         [], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap();
//
//     let mut res = vec![];
//     let mut bar = default_bar_unknown();
//
//     for row in rows {
//         let (page_id, wiki_name): (u32, String) = row.unwrap();
//         bar.inc(1);
//
//         // if let Some(ref wname) = where_wiki_name {
//         //     if wname != &wiki_name {
//         //         continue;
//         //     }
//         // }
//         //
//         // let page_title = page_id_to_title(PageId(page_id as u32)).unwrap().0;
//         // res.push(Page {
//         //     page_title,
//         //     page_id,
//         //     wiki_name,
//         // });
//     }
//     bar.finish();
//     res
// }
//

/// returns ids of pages with the most links
pub fn select_link_count_groupby(
    top: usize,
    db_path: impl AsRef<Path>,
    groupby: &str,
) -> Vec<(u64, u64)> {
    let mut link_count = vec![];

    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {groupby}, COUNT(*) FROM WikiLink \
            GROUP BY {groupby} ORDER BY count(*) DESC LIMIT {top}"
        ))
        .unwrap();

    let rows = stmt
        .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
        .unwrap();

    for row in rows {
        link_count.push(row.unwrap())
    }

    link_count
}

/// return ids of the most linked page. linked by other pages the most times
pub fn top_linked_ids(top: usize, wiki_name: Option<&str>) -> HashSet<PageId, FxBuildHasher> {
    let mut link_count = FxHashSet::default();

    let where_wiki = wiki_name
        .map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'"))
        .unwrap_or_default();

    let conn = Connection::open(DB_STATS).unwrap();
    let stmt_str = format!(
        "SELECT page_link, COUNT(*) FROM WikiLink \
            {where_wiki} GROUP BY page_link ORDER BY count(*) DESC LIMIT {top}"
    );
    // dbg!(&stmt_str);
    let mut stmt = conn.prepare(&stmt_str).unwrap();

    let rows = stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap();

    println!("query done");
    let bar = default_bar_unknown();

    for row in rows {
        link_count.insert(PageId(row.unwrap()));
        bar.inc(1);
    }
    // bar.finish();

    // dbg!(&link_count.len());
    link_count
}

/// returns ids of pages with the most links
pub fn top_link_ids(top: usize, db_path: impl AsRef<Path>) -> HashSet<PageId, FxBuildHasher> {
    let mut link_count = FxHashSet::default();

    // let where_wiki = wiki_name.map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'")).unwrap_or_default();

    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn
        .prepare(&format!(
            "SELECT page_id, COUNT(*) FROM WikiLink \
    GROUP BY page_id ORDER BY count(*) DESC LIMIT {top}"
        ))
        .unwrap();
    // stmt.execute([top]).unwrap();

    let rows = stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap();

    for row in rows {
        link_count.insert(PageId(row.unwrap()));
    }

    link_count
}

// TODO: add test with sample test db files
