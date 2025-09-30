use std::{fs, sync::{Arc, Mutex}, thread};

use crossbeam::{channel::unbounded, queue::ArrayQueue};
use futures::{pin_mut, StreamExt};
use fxhash::FxHashMap;
use indicatif::MultiProgress;
use log::{debug, info};
use parse_mediawiki_sql::field_types::PageId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::Instant};


use crate::{
    calc::bfs::{bfs, bfs_bidirectional, build_path, SpBiStream},
    sqlite::{
        page_links::get_cache,
        title_id_conv::{self, get_random_page},
    },
    stats::{
        stats::PageTitle,
        utils::{average_histograms, MaxMinAvg},
    },
    utils::{default_bar, ProgressBarBuilder},
    AvgDepthHistogram, DepthHistogram,
    WikiIdent
};

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

    let avg_depth_histogram = average_histograms(&depth_histograms);

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
    let bar2 = m.add(crate::utils::bar_color("magenta", pid_queue.len() as u64));

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
