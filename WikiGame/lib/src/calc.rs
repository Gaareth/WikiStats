#![feature(duration_constructors)]
#![feature(let_chains)]
#![allow(unused)]
extern crate core;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};
use std::ops::Add;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{env, thread};

use async_stream::stream;
use chrono::{Datelike, TimeZone};
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::queue::{ArrayQueue, SegQueue};
use futures::Stream;
use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use indicatif::{MultiProgress, ProgressBar};
use log::{debug, trace};
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sqlite::page_links::{get_links_of_ids, load_link_to_map_db_wiki};
use crate::sqlite::paths::SPStat;
use crate::sqlite::title_id_conv::{load_id_title_map, load_title_id_map};
use crate::sqlite::{db_sp_wiki_path, db_wiki_path, paths};
use crate::stats::top_linked_ids;
use crate::utils::{bar_color, default_bar, default_bar_unknown};
use crate::web::get_most_popular_pages;
use crate::{sqlite, DBCache, DepthHistogram, DistanceMap, PrevMap, PrevMapEntry};

// TODO: create sqlite3 database containing only pageid and pagetable

// mod utils;
// mod sqlite;
// mod download;
// mod stats;
// mod cli;

pub const MAX_SIZE: u32 = 210_712_457; // 225_574_049

// 136_343_429 with only namespace 0
// select distinct: 136_292_965
//     dbg!(&count_page_links_sqlfile(pagelinks_sql));
// 146_837_493
#[tokio::main]
async fn main() {
    // num rows: 167_081_462 -> 210_712_457
    // unique:  5_436_244
    // main content links:         4_258_651
    let t1 = Instant::now();
    // stats::create_stats().await;

    // dbg!(&load_title_id_map("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/de_database.sqlite").len());

    // // "de", "en", "fr", "es", "ja", "ru"
    // download::download_wikis(wikis(vec!["de", "en", "fr", "es", "ja", "ru"]),
    //                          "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/").await;
    // process_wikis(wikis(vec!["de", "en", "fr", "es", "ja", "ru"])).await;
    // sqlite::to_sqlite::post_insert("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240301/database-enwiki_dewiki_frwiki_eswiki_jawiki_ruwiki.sqlite");

    // sqlite::to_sqlite::create_db(
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/de_database_small.sqlite",
    //     "de",
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-pagelinks.sql",
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-page.sql",
    // );

    // sqlite::to_sqlite::create_db(
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240301/es_database2.sqlite",
    //     "eswiki",
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/eswiki-20240301-pagelinks.sql",
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/eswiki-20240301-page.sql",
    // );

    // sqlite::to_sqlite::post_insert("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240301/es_database2.sqlite", "eswiki");

    // sqlite::to_sqlite::create_db(
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/de_pagelinks.sqlite",
    //     "de",
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-pagelinks.sql",
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-page.sql",
    // );

    // let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-pagelinks.sql";
    // let pagelinks_sql = unsafe { memory_map(path).unwrap() };
    // sqlite::page_links::count_duplicates2("/run/media/gareth/7FD71CF32A89EF6A/dev/pagelinks_og_test2.db");
    // load_sql_part_set2(&pagelinks_sql, MAX_SIZE as usize, 1);
    // sqlite::to_sqlite::count_progress_bar::<PageLink>(&pagelinks_sql);
    // count()
    // dbg!(sqlite::title_id_conv::page_id_to_title(PageId(655087)));
    // let title_id_map = sqlite::title_id_conv::load_title_id_map();
    // sqlite::title_id_conv::test2();
    // dbg!(sqlite::page_links::get_links_of_id_and_connect(&PageId(51134)));

    // test_load_link_in_memory_sql();
    // let data= load_sql_part_map(pagelinks_sql, MAX_SIZE / 4, 1);
    // let data: FxHashSet<(PageId, PageTitle)> = load_sql_part(pagelinks_sql, MAX_SIZE / 4, 1);
    // println!("loaded bitches: {}", data.len());
    // //

    // // save_as_json(pagelinks_sql);
    // let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/dewiki-20221020-page.sql";
    // let page_sql = unsafe { memory_map(path).unwrap() };
    // sqlite::to_sqlite::count_progress_bar::<Page>(&page_sql);
    // sqlite::to_sqlite::create_db("/run/media/gareth/7FD71CF32A89EF6A/dev/de_pagelinks_test.db",
    //                              "de");
    // sqlite::to_sqlite::create_db(
    //     "/run/media/gareth/7FD71CF32A89EF6A/dev/de_db.db",
    //     "de");
    //
    // sqlite::title_id_conv::convert("/run/media/gareth/7FD71CF32A89EF6A/dev/page_db.db");
    // sqlite::title_id_conv::convert();
    // sqlite::title_id_conv::count_duplicates("/run/media/gareth/7FD71CF32A89EF6A/dev/pages_dups.db");

    // sqlite::page_links::count_duplicates("/run/media/gareth/7FD71CF32A89EF6A/dev/pagelinks_dups.db");

    // test_get_links_db_speed();
    // test_title_to_id_in_db_speed()
    // test_title_to_id_in_memory_speed();

    // top_linked(10_000);
    // test_load_link_in_memory_sql();
    // test_load_link_in_memory_db();

    // load_link_to_map_db();
    // sleep(Duration::new(5, 0));

    // get_most_popular_pages("dewiki");
    let wiki_name = "enwiki";
    let path = db_wiki_path(wiki_name);
    let path_sp = db_sp_wiki_path(wiki_name);

    let cache = load_link_to_map_db_wiki(&path);
    // let cache = FxHashMap::default();

    // paths::build_sp(&PageId(252620), &PageId(286541));

    // // let mut l1 = top_linked_ids(5);
    // // l1.extend(top_link_ids(5));
    // // dbg!(&l1.len());
    // // top_linked(100);
    // // floyd_warshall(&cache);
    //
    // precalc_interlinks_most_popular(&cache);
    // dbg!(&load_id_to_title(path, "ja").len());

    precalc_interlinks_most_popular_threaded(&path_sp, &path, &cache, wiki_name);

    // let start_link = PageTitle("Taylor_Swift".to_string());
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&start_link, &path).unwrap();
    // // // dbg!(&start_link_id);
    // // //
    // let end_link = PageTitle("Taiwan".to_string());
    // let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link, &path).unwrap(); // https://de.wikipedia.org/?curid=5885036
    // // dbg!(&end_link_id);
    // //
    // let (depth_histogram, prev_map, num_visited, deepest_id) = bfs(&start_link_id,
    //                                                                Some(&end_link_id), None, &cache, &path);
    //     dbg!(build_path(&end_link_id, &prev_map));

    //
    // let mut links_map: FxHashMap<(PageId, PageId), usize> = FxHashMap::default();
    // sqlite::paths::save_shortest_paths_json(db_sp_wiki_path(wiki_name), &prev_map,
    //                                         &mut links_map,
    //                                         &start_link_id);

    // paths::build_sp(&PageId(3454512), &end_link_id);

    //
    //

    //
    // let id_title_map = load_id_to_title(path, "ja");
    // let start_link_title = id_title_map.get(&start_link_id).unwrap();
    // let longest_path = build_path(&deepest_id, &prev).iter().skip(1)
    //     .map(|pid| id_title_map.get(pid).unwrap().clone().0).collect();
    //
    // sqlite::paths::save_stats("ja".to_string(), start_link_title.clone().0, SPStat {
    //     longest_path,
    //     num_visited,
    //     depth_histogram,
    // });

    // let mut leaves: HashSet<&PageId> = prev
    //     .iter().map(|(end, _start)| end).collect();
    // for (_end, start) in prev.iter() {
    //     leaves.remove(start);
    // }
    //
    // dbg!(&leaves.len());
    // dbg!(build_path(&end_link_id, &prev));
    // dbg!(&vc);

    //  let (_, prev, vc) = dijkstra(&start_link_id, None, Some(2), &cache);
    // dbg!(build_path(&end_link_id, &prev));
    // dbg!(&vc);

    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&PageTitle("Auto_(Begriffsklärung)".parse().unwrap())).unwrap();
    // let end_link_id = sqlite::title_id_conv::page_title_to_id(&PageTitle("Hot_Spot_(WLAN)".parse().unwrap())).unwrap();
    //
    // calc_iter(&start_link_id, &end_link_id, 4, &cache);
    //
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&PageTitle("Lionel_Messi".parse().unwrap())).unwrap();
    // let end_link_id = sqlite::title_id_conv::page_title_to_id(&PageTitle("Jérôme_Boateng".parse().unwrap())).unwrap();
    //
    // calc_iter(&start_link_id, &end_link_id, 4, &cache);
    //
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&PageTitle("Björn_Höcke".parse().unwrap())).unwrap();
    // let end_link_id = sqlite::title_id_conv::page_title_to_id(&PageTitle("Batman".parse().unwrap())).unwrap();
    //
    // calc_iter(&start_link_id, &end_link_id, 4, &cache);
    // calc();

    dbg!(t1.elapsed());
}

// fn precalc_interlinks_most_popular<S: BuildHasher>(cache: &HashMap<PageId, Vec<PageId>, S>) {
//     sqlite::paths::create_db();
//     let num_ids = 100;
//     let subset = top_link_ids(num_ids);
//     let precalced_ids = sqlite::paths::precalced_path_ids();
//     // dbg!(&precalced_ids.len());
//
//     let bar = default_bar(num_ids as u64);
//     for start_link_id in subset {
//         dbg!(&start_link_id);
//         bar.inc(1);
//         if precalced_ids.contains(&start_link_id) {
//             continue;
//         }
//
//         let (_, prev, _) = bfs(&start_link_id, None, None, cache);
//         // let end_link = PageTitle("Kuba".to_string());
//         // let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link).unwrap(); // https://de.wikipedia.org/?curid=5885036
//         //
//         // dbg!(build_path(&end_link_id, &prev));
//         sqlite::paths::save_shortest_paths(&prev, &start_link_id, "de");
//     }
// }

pub fn precalc_interlinks_most_popular_threaded_quick(wiki_name: impl AsRef<str>) {
    let wiki_name = wiki_name.as_ref();
    let path = db_wiki_path(wiki_name);
    let path_sp = db_sp_wiki_path(wiki_name);
    let cache = load_link_to_map_db_wiki(&path);
    precalc_interlinks_most_popular_threaded(path_sp, &path, &cache, wiki_name);
}

pub(crate) fn precalc_interlinks_most_popular_threaded(
    db_sp_path: impl AsRef<Path>,
    db_path: impl AsRef<Path>,
    cache: &DBCache,
    wiki_name: &str,
) {
    paths::create_db(&db_sp_path);
    let num_ids = 5;
    const NUM_THREADS: u32 = 5;

    let pid_queue: Arc<ArrayQueue<PageId>> = Arc::new(ArrayQueue::new(num_ids));
    // let prev_queue: PrevMapQueue = Arc::new(SegQueue::new());

    let arc_cache = Arc::new(cache.clone());

    let title_to_id_map = load_title_id_map(db_path);
    // let top_ids = top_link_ids(num_ids, wiki_name);
    let top_ids: Vec<&PageId> = get_most_popular_pages(wiki_name)
        .into_iter()
        .filter_map(|(p, c)| title_to_id_map.get(&PageTitle(p)))
        .take(num_ids)
        .collect();

    dbg!(&top_ids.len());
    // dbg!(&top_ids);

    // let precalced_ids = Arc::new(sqlite::paths::precalced_path_ids(&db_sp_path));
    let precalced_ids = Arc::new(FxHashSet::default());

    for pid in top_ids {
        if !precalced_ids.contains(pid) {
            pid_queue.push(*pid).unwrap();
        }
    }

    // pid_queue.push(PageId(3454512)).unwrap();
    //     pid_queue.push(PageId(1)).unwrap();

    println!("Already calculated: {}", precalced_ids.len());
    println!("Now in queue: {}", pid_queue.len());

    let m = Arc::new(MultiProgress::new());

    let bar = Arc::new(Mutex::new(m.add(default_bar(pid_queue.len() as u64))));
    let bar2 = Arc::new(Mutex::new(
        m.add(bar_color("magenta", pid_queue.len() as u64)),
    ));

    let mut thread_handles_aq: Vec<thread::JoinHandle<()>> = Vec::new();

    type ChannelContent = (DepthHistogram, PrevMapEntry, u32, VecDeque<PageId>);

    fn worker(
        tid: u32,
        queue: Arc<ArrayQueue<PageId>>,
        cache: Arc<DBCache>,
        precalced_ids: Arc<FxHashSet<PageId>>,
        bar: Arc<Mutex<ProgressBar>>,
        mbar: Arc<MultiProgress>,
        wiki_name: String,
        prev_queue_sender: Sender<ChannelContent>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            while !queue.is_empty() {
                let start_link_id = queue.pop().unwrap();
                // println!("[{tid}]: {:?}", start_link_id);
                let t1 = Instant::now();
                if precalced_ids.contains(&start_link_id) {
                    bar.lock().unwrap().inc(1);
                    continue;
                }

                let r = bfs(&start_link_id, None, None, &cache, db_wiki_path(&wiki_name));

                let mut longest_path = build_path(&r.deepest_id, &r.prev_map);
                longest_path.pop_front(); // skip first as its known

                // prev_queue.push((prev, start_link_id, wiki_name.clone()));
                prev_queue_sender
                    .send((
                        r.depth_histogram,
                        (r.prev_map, start_link_id, wiki_name.clone()),
                        r.num_visited,
                        longest_path,
                    ))
                    .unwrap();
                bar.lock().unwrap().inc(1);
                // println!("Send {start_link_id:?}");

                // let end_link = PageTitle("Kuba".to_string());
                // let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link).unwrap(); // https://de.wikipedia.org/?curid=5885036
                //
                // dbg!(build_path(&end_link_id, &prev));
                // println!("[{tid}]: Done: {:?} {:?}", start_link_id, t1.elapsed());
            }
            mbar.println(format!("[{tid}]: Done"));
            drop(prev_queue_sender);
        })
    }

    fn save(
        wiki_name: String,
        prev_queue_receiver: Receiver<ChannelContent>,
        pid_queue: Arc<ArrayQueue<PageId>>,
        bar2: Arc<Mutex<ProgressBar>>,
    ) -> thread::JoinHandle<()> {
        println!("Save thread started");
        thread::spawn(move || {
            let id_title_map = load_id_title_map(db_wiki_path(wiki_name));
            let mut links_map: FxHashMap<(PageId, PageId, PageId), usize> = FxHashMap::default();

            while let Ok(entry) = prev_queue_receiver.recv() {
                let (
                    depth_histogram,
                    (prev_map, start_link_id, wiki_name),
                    num_visited,
                    longest_path_ids,
                ) = entry;

                println!("Received {start_link_id:?}");
                paths::save_shortest_paths(
                    &db_sp_wiki_path(&wiki_name),
                    &prev_map,
                    &mut links_map,
                    &start_link_id,
                );

                // sqlite::paths::save_shortest_paths_json(&db_sp_wiki_path(&wiki_name),
                //                                         &prev_map,
                //                                         &mut links_map,
                //                                         &start_link_id);

                let start_link_title = id_title_map.get(&start_link_id).unwrap();
                let longest_path = longest_path_ids
                    .iter()
                    .map(|pid| id_title_map.get(pid).unwrap().clone().0)
                    .collect();

                sqlite::paths::save_stats(
                    wiki_name,
                    start_link_title.clone().0,
                    SPStat {
                        longest_path,
                        num_visited,
                        depth_histogram,
                    },
                );
                // println!("Saved {start_link_id:?}");

                bar2.lock().unwrap().inc(1);

                // if pid_queue.is_empty() {
                //     break;
                // }
            }
            println!("Receiver finished");
        })
    }

    let (s, r) = unbounded();

    thread_handles_aq.push(save(
        wiki_name.to_string(),
        r,
        pid_queue.clone(),
        bar2.clone(),
    ));

    for tid in 0..NUM_THREADS {
        println!("started thread {tid}");
        thread_handles_aq.push(worker(
            tid,
            pid_queue.clone(),
            arc_cache.clone(),
            precalced_ids.clone(),
            bar.clone(),
            m.clone(),
            wiki_name.to_string(),
            s.clone(),
        ));
    }

    dbg!(&pid_queue.is_empty());

    // let saved_thread = save(r);

    thread_handles_aq
        .into_iter()
        .for_each(|th| th.join().expect("can't join thread"));

    println!("Finished");
    // saved_thread.join();
}

// fn calc_interlinks<S: BuildHasher>(start: &str, end: &str, max_depth: i32,
//                                    cache: &HashMap<PageId, Vec<PageId>, S>) {
//     let start_link_id = sqlite::title_id_conv::page_title_to_id(
//         &PageTitle(String::from(start))).unwrap();
//     let end_link_id = sqlite::title_id_conv::page_title_to_id(
//         &PageTitle(String::from(end))).unwrap();
//
//     calc_iter(&start_link_id, &end_link_id, max_depth, &cache);
// }

static HITS: AtomicUsize = AtomicUsize::new(0);
static MISSES: AtomicUsize = AtomicUsize::new(0);

static WORKS: AtomicUsize = AtomicUsize::new(0);
static ACCESSES: AtomicUsize = AtomicUsize::new(0);
// TODO: fix redirect in database
// => replace all redircts with their target page id

fn get_incoming_links<T: BuildHasher>(
    conn: &Connection,
    page_id: &PageId,
    cache: &HashMap<PageId, Vec<PageId>, T>,
) -> Vec<PageId> {
    // sqlite::page_links::get_incoming_links_of_id(conn, page_id)


    return if cache.contains_key(page_id) {
        HITS.fetch_add(1, Ordering::SeqCst);
        cache.get(page_id).unwrap().clone()
    } else {
        MISSES.fetch_add(1, Ordering::SeqCst);
        sqlite::page_links::get_incoming_links_of_id(conn, page_id)
    };
  
}

fn get_links<T: BuildHasher>(
    conn: &Connection,
    page_id: &PageId,
    cache: &HashMap<PageId, Vec<PageId>, T>,
) -> Vec<PageId> {
    // cache.get(page_id).unwrap_or(&vec![]).clone()
    // let c: Vec<u32> = cache.get(page_id).unwrap_or(&vec![]).clone().iter().map(|p| p.0).collect();
    // let ch: HashSet<u32> = HashSet::from_iter(c);
    // let d: Vec<u32> = sqlite::page_links::get_links_of_id(conn, page_id).iter().map(|p| p.0).collect();
    // let dh: HashSet<u32> = HashSet::from_iter(d);
    // let diff: Vec<_> = ch.symmetric_difference(&dh).collect();
    // if !diff.is_empty() {
    //     dbg!(&diff);
    //     dbg!(&page_id);
    //     dbg!(&ch);
    //     dbg!(&dh);
    // }

    return if cache.contains_key(page_id) {
        HITS.fetch_add(1, Ordering::SeqCst);
        cache.get(page_id).unwrap().clone()
    } else {
        MISSES.fetch_add(1, Ordering::SeqCst);
        sqlite::page_links::get_links_of_id(conn, page_id)
    };
    // cache.get(page_id).unwrap_or(&vec![]).clone()

    // let res = sqlite::page_links::get_links_of_id(conn, page_id);
    // // dbg!(res.len());
    // ACCESSES.fetch_add(res.len(), Ordering::SeqCst);
    // res

    // sqlite::page_links::get_links_of_id(conn, page_id)
}

fn insert_rest_path(
    working_ids: &mut HashMap<PageId, Vec<PageId>, FxBuildHasher>,
    path: &Vec<PageId>,
) {
    for (i, id) in path.iter().enumerate() {
        let rest_path = &path[i + 1..];
        match working_ids.get(id) {
            None => {
                working_ids.insert(*id, rest_path.to_vec());
                // println!("inserting {:?} {:?}", id, rest_path);
            }
            Some(path) => {
                if path.len() > rest_path.len() {
                    working_ids.insert(*id, rest_path.to_vec());
                    // println!("inserting {:?} {:?}", id, rest_path);
                }
            }
        }
    }
    // dbg!(&working_ids);
}

fn get_2d(i: u32, j: u32, map: &mut FxHashMap<u32, FxHashMap<u32, u32>>) -> u32 {
    // let dist_i_j = map.entry(i as u32)
    //                .or_insert(FxHashMap::default()).get(&(j as u32)).unwrap_or(&u32::MAX);
    if map.get(&i).is_none() {
        map.insert(i, FxHashMap::default());
    }
    *map.get(&i).unwrap().get(&j).unwrap_or(&u32::MAX)
}

fn floyd_warshall<S: BuildHasher>(cache: &HashMap<PageId, Vec<PageId>, S>) {
    // let mut dist: Vec<Vec<u8>> = (0..4464087)
    // .map(|_| (0..4464087).map(|_| 0).collect())
    // .collect();
    let mut dist: FxHashMap<u32, FxHashMap<u32, u32>> = FxHashMap::default();
    // dbg!(&dist[0]);
    // return

    let all_ids = get_all_ids(cache);
    let num_ids = all_ids.len();
    println!("There are {num_ids} ids");

    //
    // for vertex in all_ids.iter() {
    //     // dist.get(vertex.0 as usize).unwrap_or(vec![])[vertex.0 as usize] = 0;
    //     dist.get()
    // }
    //
    // println!("initialized self edges");

    for (u, v) in get_all_edges(cache) {
        let uid = u.0;
        let vid = v.0;

        if dist.get(&uid).is_none() {
            let neighbours = FxHashMap::default();
            dist.insert(uid, neighbours);
        }
        let neighbours = dist.get_mut(&uid).unwrap();
        neighbours.insert(vid, 1);
    }
    println!("initialized other edges");

    let subset = top_linked_ids(5, Some("de"));
    dbg!(&subset.len());

    let bar = default_bar((subset.len().pow(2)) as u64);
    for k in subset.iter() {
        for i in subset.iter() {
            for j in subset.iter() {
                let dist_i_j = get_2d(i.0, j.0, &mut dist);
                let dist_i_k = get_2d(i.0, k.0, &mut dist);
                let dist_k_j = get_2d(k.0, j.0, &mut dist);

                if dist_i_j > (dist_i_k + dist_k_j) {
                    dist.get_mut(&(i.0))
                        .unwrap()
                        .insert(j.0, (dist_i_k + dist_k_j));
                    // dist[i][j] = dist[i][k] + dist[k][j]
                }
            }
            bar.inc(1);
        }
    }
    //  let start_link = PageTitle("Auto_(Begriffsklärung)".to_string());
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&start_link).unwrap();
    //
    // let end_link = PageTitle("Kuba".to_string());
    //  let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link).unwrap();

    dbg!(&dist.get(&6000499).unwrap().get(&5782088).unwrap());
    bar.finish();
}

fn get_all_edges<S: BuildHasher>(
    cache: &HashMap<PageId, Vec<PageId>, S>,
) -> HashSet<(PageId, PageId), FxBuildHasher> {
    let mut edges: FxHashSet<(PageId, PageId)> = FxHashSet::default();
    for (id, links) in cache {
        for edge in links {
            edges.insert((*id, *edge));
        }
    }
    return edges;
}

fn get_all_ids<S: BuildHasher>(
    cache: &HashMap<PageId, Vec<PageId>, S>,
) -> HashSet<PageId, FxBuildHasher> {
    let mut ids = FxHashSet::default();
    for (id, links) in cache {
        ids.insert(*id);
        ids.extend(links);
    }
    return ids;
}

fn dijkstra<S: BuildHasher>(
    start_link_id: &PageId,
    end_link_id_opt: Option<&PageId>,
    max_depth_opt: Option<u32>,
    cache: &HashMap<PageId, Vec<PageId>, S>,
) -> (DistanceMap, PrevMap, u32) {
    let mut dist: DistanceMap = FxHashMap::default();
    let mut prev: PrevMap = FxHashMap::default();

    let mut to_visit: VecDeque<(PageId, u32)> = VecDeque::from([(*start_link_id, 0)]);
    let conn = Connection::open("/home/gareth/dev/Rust/WikiGame/pagelinks_full_ids.db").unwrap();

    let mut visited: HashSet<PageId> = HashSet::default();
    let mut visited_counter = 0;

    while !to_visit.is_empty() {
        let (current_id, depth) = to_visit.pop_front().unwrap();

        if visited.contains(&current_id) {
            continue;
        }

        visited.insert(current_id);
        visited_counter += 1;

        // if visited_counter % 100_000 == 0 {
        //     dbg!(&visited_counter);
        //     dbg!(&visited.len());
        // }

        if let Some(end_link_id) = end_link_id_opt {
            if &current_id == end_link_id {
                println!("Found endlink");
                break;
            }
        }

        // skip adding additional links if they are out of "reach"
        if let Some(max_depth) = max_depth_opt {
            if depth + 1 > max_depth {
                continue;
            }
        }

        for link in get_links(&conn, &current_id, &cache) {
            // dbg!(&link);
            // && !to_visit.contains(&link)
            if !visited.contains(&link) {
                to_visit.push_back((link, depth + 1));
                let alternative = dist.get(&current_id).unwrap_or(&u32::MAX) + 1;
                if alternative < *dist.get(&link).unwrap_or(&u32::MAX) {
                    dist.insert(link, alternative);
                    prev.insert(link, current_id);
                }
            }
        }
    }
    dbg!(&visited.len());
    return (dist, prev, visited_counter);
}

fn bfs_worker(
    id_queue: Arc<SegQueue<(PageId, u32)>>,
    cache: Arc<DBCache>,
    max_depth_opt: Option<u32>,
    end_link_id_opt: Option<PageId>,
    visited: Arc<Mutex<FxHashSet<PageId>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let conn =
            Connection::open("/home/gareth/dev/Rust/WikiGame/pagelinks_full_ids.db").unwrap();

        while !id_queue.is_empty() {
            let (current_id, depth) = id_queue.pop().unwrap();
            // dbg!(&current_id);
            // if visited.contains(&current_id) {
            //     continue;
            // }

            // visited_counter += 1;

            // if visited_counter % 100_000 == 0 {
            //     dbg!(&visited_counter);
            //     dbg!(&visited.lock().unwrap().len());
            // }

            if let Some(end_link_id) = end_link_id_opt {
                if current_id == end_link_id {
                    println!("Found endlink");
                    break;
                }
            }
            //
            // skip adding additional links if they are out of "reach"
            if let Some(max_depth) = max_depth_opt {
                if depth + 1 > max_depth {
                    continue;
                }
            }

            for link in get_links(&conn, &current_id, &cache) {
                // && !to_visit.contains(&link)
                // dbg!(&link);

                if !visited.lock().unwrap().contains(&link) {
                    // dbg!(&link);
                    visited.lock().unwrap().insert(link);
                    id_queue.push((link, depth + 1));
                    // prev.insert(link, current_id);
                    // println!("{:?} -> {:?}", current_id, link);
                }
            }
        }
    })
}

fn bfs_parallel(
    start_link_id: &PageId,
    end_link_id_opt: Option<&PageId>,
    max_depth_opt: Option<u32>,
    cache: &DBCache,
) -> (DistanceMap, PrevMap, u32) {
    let mut dist: DistanceMap = FxHashMap::default();
    let mut prev: PrevMap = FxHashMap::default();

    // let mut to_visit: VecDeque<(PageId, u32)> = VecDeque::from([(*start_link_id, 0)]);
    // let conn = Connection::open("/home/gareth/dev/Rust/WikiGame/pagelinks_full_ids.db").unwrap();

    let mut visited: Arc<Mutex<FxHashSet<PageId>>> = Arc::new(Mutex::new(FxHashSet::default()));
    visited.lock().unwrap().insert(*start_link_id);

    dbg!(&visited.lock().unwrap().len());
    let mut visited_counter = 0;

    let id_queue: Arc<SegQueue<(PageId, u32)>> = Arc::new(SegQueue::new());
    id_queue.push((*start_link_id, 0));

    let mut thread_handles_aq: Vec<thread::JoinHandle<()>> = Vec::new();

    let arc_cache = Arc::new(cache.clone());

    for i in 0..6 {
        println!("started thread {i}");
        thread_handles_aq.push(bfs_worker(
            id_queue.clone(),
            arc_cache.clone(),
            max_depth_opt,
            end_link_id_opt.copied(),
            visited.clone(),
        ));
    }

    thread_handles_aq
        .into_iter()
        .for_each(|th| th.join().expect("can't join thread"));

    dbg!(&visited.lock().unwrap().len());
    dbg!(&prev.len());
    return (dist, prev, visited_counter);
}

pub struct BfsResult {
    pub visited: FxHashSet<PageId>,
    pub depth_histogram: DepthHistogram,
    pub prev_map: PrevMap,
    pub num_visited: u32,
    pub deepest_id: PageId,
    pub len_deepest_sp: u32,
}

fn get_paths(
    page_ids: &[Option<PageId>],
    visited_dict: &FxHashMap<PageId, Vec<Option<PageId>>>,
) -> Vec<Vec<PageId>> {
    let mut paths: Vec<Vec<PageId>> = Vec::new();

    for page_id_option in page_ids {
        if let Some(page_id) = page_id_option {
            let current_paths = get_paths(visited_dict.get(page_id).unwrap(), visited_dict);
            for current_path in current_paths {
                let mut new_path = current_path.clone();
                new_path.push(*page_id);
                paths.push(new_path);
            }
        } else {
            return vec![vec![]];
        }
    }

    paths
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpBiStream {
    pub visited: u64,
    pub elapsed_ms: u128,
    pub paths: Option<FxHashSet<Vec<String>>>,
}

pub async fn bfs_bidirectional(
    start_link_id: PageId,
    end_link_id_opt: PageId,
    db_path: impl AsRef<Path> + 'static,
) -> impl Stream<Item = SpBiStream> + 'static {
    stream! {
        let conn = Connection::open(db_path).unwrap();

        let mut unvisited_forward: FxHashMap<PageId, Vec<Option<PageId>>> = FxHashMap::default();
        unvisited_forward.insert(start_link_id, vec![None]);

        let mut unvisited_backward: FxHashMap<PageId, Vec<Option<PageId>>> = FxHashMap::default();
        unvisited_backward.insert(end_link_id_opt, vec![None]);

        let mut visited_forward: FxHashMap<PageId, Vec<Option<PageId>>> = FxHashMap::default();
        let mut visited_backward: FxHashMap<PageId, Vec<Option<PageId>>> = FxHashMap::default();

        let mut forward_depth = 0;
        let mut backward_depth = 0;

        let mut paths: FxHashSet<Vec<PageId>> = FxHashSet::default();

        let start_time = Instant::now();
        let mut total_visited = 0;

        while paths.is_empty() && (!unvisited_forward.is_empty() && !unvisited_backward.is_empty()) {
           // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            // this won't really yield?? or send a http request without delay IDK????
           tokio::time::sleep(tokio::time::Duration::from_millis(0)).await;

            let forward_links = get_links_of_ids(
                &conn,
                unvisited_forward.keys().cloned().collect::<Vec<PageId>>(), false);

            let backward_links = get_links_of_ids(
                &conn,
                unvisited_backward.keys().cloned().collect::<Vec<PageId>>(), true);

            if forward_links.len() < backward_links.len() {
                forward_depth += 1;

                for (page_id, parents) in &unvisited_forward {
                    visited_forward.insert(*page_id, parents.clone());
                }

                total_visited += unvisited_forward.len();
                yield SpBiStream {
                    visited: total_visited as u64,
                    elapsed_ms: start_time.elapsed().as_millis(),
                    paths: None
                };

                // dbg!(&total_visited);

                unvisited_forward.clear();

                for (source_page_id, target_page_id) in forward_links {
                    if !visited_forward.contains_key(&target_page_id) && !unvisited_forward.contains_key(&target_page_id) {
                        unvisited_forward.insert(target_page_id, vec![Some(source_page_id)]);
                    } else if unvisited_forward.contains_key(&target_page_id) {
                        unvisited_forward.get_mut(&target_page_id).unwrap().push(Some(source_page_id));
                    }
                }
            } else {
                backward_depth += 1;

                for (page_id, parents) in &unvisited_backward {
                    visited_backward.insert(*page_id, parents.clone());
                }

                total_visited += unvisited_backward.len();
                yield SpBiStream {
                    visited: total_visited as u64,
                    elapsed_ms: start_time.elapsed().as_millis(),
                    paths: None
                };
                // dbg!(&total_visited);


                unvisited_backward.clear();

                for (source_page_id, target_page_id) in backward_links {
                    if !visited_backward.contains_key(&source_page_id) && !unvisited_backward.contains_key(&source_page_id) {
                        unvisited_backward.insert(source_page_id, vec![Some(target_page_id)]);
                    } else if unvisited_backward.contains_key(&source_page_id) {
                        unvisited_backward.get_mut(&source_page_id).unwrap().push(Some(target_page_id));
                    }
                }
            }

            for (page_id, parents) in &unvisited_forward {
                if unvisited_backward.contains_key(page_id) {
                    let paths_from_source = get_paths(
                        unvisited_forward.get(page_id).unwrap(), &visited_forward);
                    let paths_from_target = get_paths(
                        unvisited_backward.get(page_id).unwrap(), &visited_backward);

                    for path_from_source in &paths_from_source {
                        for path_from_target in &paths_from_target {
                            let mut current_path = path_from_source.clone();
                            current_path.push(*page_id);
                            current_path.extend(path_from_target.iter().rev());

                            paths.insert(current_path);
                        }
                    }
                }
            }
        }

        yield SpBiStream {
            visited: total_visited as u64,
            elapsed_ms: start_time.elapsed().as_millis(),
            paths: Some(paths.iter().map(|v|
                v.iter().map(|pid| sqlite::title_id_conv::page_id_to_title(pid, &conn).unwrap().0).collect::<Vec<String>>()).collect())
        };
        // dbg!(&total_visited);

    }
}

pub fn bfs(
    start_link_id: &PageId,
    end_link_id_opt: Option<&PageId>,
    max_depth_opt: Option<u32>,
    cache: &DBCache,
    db_path: impl AsRef<Path>,
) -> BfsResult {
    let conn: Connection = Connection::open(db_path).unwrap();

    let mut dist: DistanceMap = FxHashMap::default();
    let mut prev: PrevMap = FxHashMap::default();
    let mut histogram: DepthHistogram = FxHashMap::default();

    let mut to_visit: VecDeque<(PageId, u32)> = VecDeque::from([(*start_link_id, 0)]);

    let mut visited: FxHashSet<PageId> = FxHashSet::default();
    visited.insert(*start_link_id);

    let mut visited_counter = 0;
    let mut longest_path: VecDeque<PageId> = VecDeque::new();

    let mut deepest_id: PageId = *start_link_id;
    let mut deepest_depth = 0;

    while !to_visit.is_empty() {
        let (current_id, depth) = to_visit.pop_front().unwrap();

        if let Some(end_link_id) = end_link_id_opt {
            if &current_id == end_link_id {
                trace!("Found endlink");
                break;
            }
        }

        // skip adding additional links if they are out of "reach"
        if let Some(max_depth) = max_depth_opt {
            if depth + 1 > max_depth {
                continue;
            }
        }

        for link in get_links(&conn, &current_id, cache) {
            if !visited.contains(&link) {
                visited.insert(link);
                to_visit.push_back((link, depth + 1));
                prev.insert(link, current_id);

                histogram
                    .entry(depth + 1)
                    .and_modify(|d| *d += 1)
                    .or_insert(1);

                if depth + 1 > deepest_depth {
                    deepest_depth = depth + 1;
                    deepest_id = link;
                }

                visited_counter += 1;
            }
        }
    }

    debug!(
        "Hits to misses ratio {:?}%",
        (HITS.load(Ordering::SeqCst) as f32
            / (HITS.load(Ordering::SeqCst) + MISSES.load(Ordering::SeqCst)) as f32)
            * 100.0
    );

    BfsResult {
        visited,
        depth_histogram: histogram,
        prev_map: prev,
        num_visited: visited_counter,
        deepest_id,
        len_deepest_sp: deepest_depth,
    }
}


pub fn bfs_undirected(
    start_link_id: &PageId,
    cache_outgoing: &DBCache,
    cache_incoming: &DBCache,
    db_path: impl AsRef<Path>,
) -> FxHashSet<PageId> {
    let conn: Connection = Connection::open(db_path).unwrap();

    let mut to_visit: VecDeque<PageId> = VecDeque::from([*start_link_id]);
    let mut visited: FxHashSet<PageId> = FxHashSet::default();
    visited.insert(*start_link_id);

    while let Some(current_id) = to_visit.pop_front() {
        // Outgoing neighbors
        for link in get_links(&conn, &current_id, cache_outgoing) {
            if visited.insert(link) {
                to_visit.push_back(link);
            }
        }

        // Incoming neighbors (backlinks)
        for backlink in get_incoming_links(&conn, &current_id, cache_incoming) {
            if visited.insert(backlink) {
                to_visit.push_back(backlink);
            }
        }
    }

    visited
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpStream {
    visited: u64,
    per_sec: f64,
    shortest_path: Option<Vec<String>>,
}

pub async fn bfs_stream<S: BuildHasher>(
    start_link_id: PageId,
    end_link_id: PageId,
    max_depth_opt: Option<u32>,
    cache: &HashMap<PageId, Vec<PageId>, S>,
    db_path: String,
) -> impl Stream<Item = SpStream> + '_ {
    stream! {
            let mut prev: PrevMap = FxHashMap::default();

        let mut to_visit: VecDeque<(PageId, u32)> = VecDeque::from([(start_link_id, 0)]);
        let conn = Connection::open(db_path).unwrap();

        let mut visited: FxHashSet<PageId> = FxHashSet::default();
        visited.insert(start_link_id);

        let bar = default_bar_unknown();

        while !to_visit.is_empty() {
            let (current_id, depth) = to_visit.pop_front().unwrap();
            bar.inc(1);
            let pos = bar.position();
            if pos % 10 == 0 || pos < 100 {
                yield SpStream {
                    visited: pos,
                    per_sec: bar.per_sec(),
                    shortest_path: None
                };
            }


            if current_id == end_link_id {
                println!("Found endlink");
                break;
            }


            // skip adding additional links if they are out of "reach"
            if let Some(max_depth) = max_depth_opt {
                if depth + 1 > max_depth {
                    continue;
                }
            }

            // dbg!(&current_id);
            // dbg!(get_links(&conn, &current_id, &cache).len());
            for link in get_links(&conn, &current_id, cache) {
                if !visited.contains(&link) {

                    // dbg!(&link);
                    visited.insert(link);
                    to_visit.push_back((link, depth + 1));
                    prev.insert(link, current_id);
                }
            }
        }


        println!("{:?}%", (HITS.load(Ordering::SeqCst) as f32 / (HITS.load(Ordering::SeqCst) + MISSES.load(Ordering::SeqCst)) as f32) * 100.0);
        // (prev, visited.len())
        let sp: Vec<String> = build_path(&end_link_id, &prev)
        .iter().map(|pid| sqlite::title_id_conv::page_id_to_title(pid, &conn).unwrap().0).collect();
        yield SpStream {
                visited: bar.position(),
                per_sec: bar.per_sec(),
                shortest_path: Some(sp)
        };
    }
}

pub fn build_path<S: BuildHasher>(
    end_link_id: &PageId,
    prev: &HashMap<PageId, PageId, S>,
) -> VecDeque<PageId> {
    let mut path = VecDeque::from([*end_link_id]);
    let mut current = end_link_id;
    while let Some(prev_id) = prev.get(current) {
        current = prev_id;
        path.push_front(*current);
    }
    path
}

fn calc_iter<T: BuildHasher>(
    start_link_id: &PageId,
    end_link_id: &PageId,
    max_depth: i32,
    cache: &HashMap<PageId, Vec<PageId>, T>,
) {
    let conn = Connection::open("/home/gareth/dev/Rust/WikiGame/pagelinks_full_ids.db").unwrap();

    // let max_depth = 20;
    // let start_link = PageTitle("Auto_(Begriffsklärung)".to_string());
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&start_link).unwrap();
    dbg!(&start_link_id);

    // let end_link = PageTitle("".to_string());
    // let end_link_id = PageId(5885036);
    // let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link).unwrap();

    // let cache = load_link_to_map_db(); /* top_linked(100_000); */
    // let cache = top_linked(1_000);
    // let cache = FxHashMap::default();

    let direct_links = get_links(&conn, start_link_id, cache);

    let mut to_visit: VecDeque<(Vec<PageId>, i32)> = direct_links
        .iter()
        .map(|l| (vec![*start_link_id, *l], 1))
        .collect();

    // let mut working_paths = FxHashMap::default();
    let mut working_paths = FxHashSet::default();

    let mut working_counter = 0;
    // let mut working_ids: HashMap<PageId, Vec<PageId>, FxBuildHasher> = FxHashMap::default();
    let mut working_ids: HashSet<PageId, FxBuildHasher> = FxHashSet::default();

    let mut visited: HashSet<PageId> = HashSet::default();

    // visited.insert(start_link_id, 0);
    visited.insert(*start_link_id);
    visited.extend(direct_links);

    let mut visited_counter = 0;

    let bar = default_bar_unknown();
    while !to_visit.is_empty() {
        let link = to_visit.pop_front().unwrap();
        // dbg!(&link);
        let path = link.0;
        let current = path.last().unwrap();
        // let current = link.0;
        // dbg!(&current);
        let depth = link.1;
        bar.inc(1);
        visited_counter += 1;
        // visited2.push(*current);
        // visited.insert(*current);

        if visited_counter % 50_000 == 0 {
            dbg!(&visited_counter);
        }

        if current == end_link_id {
            println!("Found endlink \n");
            dbg!(&path);
            working_paths.insert(path.clone());
            working_counter += 1;
            // insert_rest_path(&mut working_ids, &path);
            working_ids.extend(path);
            dbg!(&working_ids);
            continue;
        }
        if depth + 1 > max_depth {
            continue;
        }

        let sub_links = get_links(&conn, &current, &cache);
        // dbg!(sub_links.len());
        // to_visit.append(sub_links);
        sub_links.iter().for_each(|l| {
            if !visited.contains(l) {
                // println!("added {:?}", l);
                visited.insert(*l);
                let mut new_path = path.clone();
                new_path.push(*l);
                to_visit.push_back((new_path, depth + 1));
            } else if working_ids.contains(l) && !path.contains(l) {
                let rest_path = working_ids.get(l).unwrap();
                let mut new_path = path.clone();
                new_path.push(*l);
                // new_path.extend(rest_path);
                // println!("Found endlink {:?} \n", l);
                // dbg!(&new_path);
                // working_paths.insert(new_path.clone(), depth);
                // insert_rest_path(&mut working_ids, &new_path);
                working_ids.extend(new_path);
                working_counter += 1;
            }
        });
    }

    bar.finish();
    let accesses = ACCESSES.load(Ordering::SeqCst);
    let works = WORKS.load(Ordering::SeqCst);

    dbg!(accesses);
    dbg!(&visited_counter);
    dbg!(working_paths.len());
    dbg!(&working_counter);

    dbg!(visited.len());
    dbg!(&working_paths.iter().take(10).collect::<Vec<_>>());
    // dbg!(visited2.len());

    // let mut set = HashSet::new();
    // for e in &visited2 {
    //     if set.contains(e) {
    //         dbg!(&e);
    //     } else {
    //         set.insert(e);
    //     }
    // }
    //
    // for v in visited2 {
    //     if !visited.contains(&v) {
    //         dbg!(&v);
    //     }
    // }

    // for (visited, d) in visited {
    //     if !cache.contains_key(&visited) {
    //         dbg!(&visited);
    //     }
    // }
    // println!("Working path ratio {}/{}: {}%", works, accesses.to_formatted_string(&Locale::en),
    //          (works as f32/accesses as f32)*100.0);
}

fn traverse_links(
    conn: &Connection,
    link: PageId,
    end_link: PageId,
    max_depth: i32,
    depth: i32,
    bar: &ProgressBar,
    cache: &HashMap<PageId, Vec<PageId>, FxBuildHasher>,
    current_path: &mut [PageId],
    visited: &mut HashMap<PageId, i32>,
    working: &mut HashMap<PageId, i32>,
) -> Option<i32> {
    visited.insert(link, depth);
    // dbg!(link);
    let mut current_path = current_path.to_vec();

    if link == end_link {
        println!("Found endlick in {}", depth);
        let mut d = 1;
        for link in current_path.clone() {
            working.insert(link, d);
            d += 1;
        }
        current_path.push(link);
        dbg!(current_path);
        dbg!(visited.len());

        WORKS.fetch_add(1, Ordering::SeqCst);
        return Some(depth);
    }

    if depth >= max_depth {
        return None;
    }

    current_path.push(link);

    // let links = sqlite::page_links::get_links_of_id(&conn, &link);

    let links = get_links(conn, &link, cache);
    if depth == 1 {
        println!("[{:?}] Searching in sub link: {} \n", link, links.len());
    }

    for link in links {
        bar.inc(1);
        ACCESSES.fetch_add(1, Ordering::SeqCst);

        if depth == max_depth - 1 && link != end_link {
            continue;
        }

        if let Some(d) = working.get(&link) {
            if &depth <= d {
                println!("early exit Found endlick in {}", depth);
                dbg!(&current_path);
                dbg!(visited.len());
                continue;
            }
        }

        // if let Some(d) = visited.get(&link) {
        //     if &depth >= d {
        //         continue
        //     }
        // }

        let opt = traverse_links(
            conn,
            link,
            end_link,
            max_depth,
            depth + 1,
            bar,
            cache,
            &mut current_path,
            visited,
            working,
        );
        // if let Some(depth) = opt {
        //     return Some(depth);
        // }
    }

    None
}

// 131 in 34
// 71 in 30 without optimizations
