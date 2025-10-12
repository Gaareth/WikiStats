use std::{
    cmp::{Ordering, max_by, min_by},
    future::Future,
    ops::AddAssign,
    path::{Path, PathBuf},
    thread,
};

use fxhash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    AvgDepthHistogram, AvgDepthStat, WikiIdent,
    sqlite::join_db_wiki_path,
    stats::stats::{StatRecord, WikiName},
};

pub static GLOBAL: &str = "global";

pub async fn make_stat_record_async<T, Fut, F>(
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

pub async fn make_stat_record_seq<T, F>(
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

pub async fn make_stat_record<T: Debug + Send + 'static>(
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

pub fn global_ignore<T>(_: &mut StatRecord<T>) {}

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

pub fn max_min_value_record<T: Clone + Debug, F: FnOnce(&T, &T) -> Ordering + Copy>(
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

pub fn average_histograms(depth_histograms: &[FxHashMap<u32, f64>]) -> AvgDepthHistogram {
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
