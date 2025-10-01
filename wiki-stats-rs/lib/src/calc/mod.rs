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

use crate::calc::bfs::build_path;
use crate::sqlite::page_links::{get_links_of_ids, load_link_to_map_db_wiki};
use crate::sqlite::paths::SPStat;
use crate::sqlite::title_id_conv::{load_id_title_map, load_title_id_map};
use crate::sqlite::{db_sp_wiki_path, db_wiki_path, paths};
use crate::stats::queries::top_linked_ids;
use crate::utils::{bar_color, default_bar, default_bar_unknown};
use crate::web::get_most_popular_pages;
use crate::{sqlite, DBCache, DepthHistogram, DistanceMap, PrevMap, PrevMapEntry};

mod floyd_warshall;
pub mod bfs;
pub mod connected_components;
// TODO: create sqlite3 database containing only pageid and pagetable

// mod utils;
// mod sqlite;
// mod download;
// mod stats;
// mod cli;

pub const MAX_SIZE: u32 = 210_712_457; // 225_574_049


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

                let r = bfs::bfs(&start_link_id, None, None, &cache, db_wiki_path(&wiki_name));

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

    return if cache.contains_key(page_id) {
        HITS.fetch_add(1, Ordering::SeqCst);
        cache.get(page_id).unwrap().clone()
    } else {
        MISSES.fetch_add(1, Ordering::SeqCst);
        sqlite::page_links::get_links_of_id(conn, page_id)
    };

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
