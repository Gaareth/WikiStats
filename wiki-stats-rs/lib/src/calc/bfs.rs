use std::{
    collections::{HashMap, VecDeque},
    hash::BuildHasher,
    path::Path,
    sync::{Arc, Mutex, atomic::Ordering},
    thread,
    time::Instant,
};

use async_stream::stream;
use crossbeam::queue::SegQueue;
use futures::Stream;
use fxhash::{FxHashMap, FxHashSet};
use log::{debug, info, trace};
use parse_mediawiki_sql::field_types::PageId;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    DBCache, DepthHistogram, DistanceMap, PrevMap,
    calc::{HITS, MISSES, get_incoming_links, get_links},
    sqlite::{self, page_links::get_links_of_ids},
    utils::default_bar_unknown,
};

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

    info!(
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
