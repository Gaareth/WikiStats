extern crate test;

use std::time::Instant;

use fxhash::{FxHashMap, FxHashSet};
use indicatif::ProgressIterator;
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use parse_mediawiki_sql::iterate_sql_insertions;
use parse_mediawiki_sql::schemas::PageLink;
use parse_mediawiki_sql::utils::memory_map;
use rusqlite::Connection;

use crate::calc::{bfs, build_path, MAX_SIZE};
use crate::sqlite;
use crate::sqlite::{db_sp_wiki_path, db_wiki_path};
use crate::sqlite::load::load_sql_part_map;
use crate::sqlite::page_links::{get_links_of_id, load_link_to_map_db_limit};
use crate::sqlite::paths::build_sp;
use crate::stats::select_link_count_groupby;
use crate::utils::default_bar;

// slow
pub fn test_load_link_map_in_memory_sql() {
    let t1 = Instant::now();

    let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-pagelinks.sql";
    let pagelinks_sql = unsafe { memory_map(path).unwrap() };

    let map=
        load_sql_part_map(pagelinks_sql, MAX_SIZE / 1, 1);

    dbg!(t1.elapsed());
}


// fast
pub fn test_load_link_map_in_memory_db() {
    let t1 = Instant::now();

    let map = sqlite::page_links::load_link_to_map_db(
        db_wiki_path("dewiki")
    );

    dbg!(t1.elapsed());
}

pub fn test_limit() {
    let wiki_name = "dewiki";
    let path = db_wiki_path(wiki_name);

    let t1 = Instant::now();
    let cached_entries: Vec<PageId> = select_link_count_groupby(1000, &wiki_name, "WikiLink.page_id")
        .into_iter().map(|(pid, _)| PageId(pid as u32)).collect();
    let cache = load_link_to_map_db_limit(&path, cached_entries, false);
    dbg!(&t1.elapsed());


    let t1 = Instant::now();
    let cached_entries: Vec<PageId> = select_link_count_groupby(100_000, &wiki_name, "WikiLink.page_id")
        .into_iter().map(|(pid, _)| PageId(pid as u32)).collect();
    let cache = load_link_to_map_db_limit(&path, cached_entries, false);


    dbg!(&t1.elapsed());
}

pub fn test_save_sp() {
    let wiki_name = "dewiki";
    let path = db_wiki_path(wiki_name);
    let path_sp = db_sp_wiki_path(wiki_name);

    // let cached_entries: Vec<PageId> = select_link_count_groupby(1000, &wiki_name, "WikiLink.page_id")
    //     .into_iter().map(|(pid, _)| PageId(pid as u32)).collect();


    // let start_link = PageTitle("Taylor_Swift".to_string());
    // let start_link_id = sqlite::title_id_conv::page_title_to_id(&start_link, &path).unwrap();
    let start_link_id = PageId(5802902);

    let conn = Connection::open(&path).unwrap();

    let end_link = PageTitle("Taiwan".to_string());
    let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link, &conn).unwrap();
    dbg!(&end_link_id);

    let cache = load_link_to_map_db_limit(&path, vec![], false);
    crate::calc::precalc_interlinks_most_popular_threaded(&path_sp, &path, &cache, wiki_name);


    // let (depth_histogram, prev_map, num_visited, deepest_id) =
    //     bfs(&start_link_id, None, None, &cache, &path);
    //
    // let mut links_map: FxHashMap<(PageId, PageId, PageId), usize> = FxHashMap::default();
    // save_shortest_paths_json(&path, &prev_map, &mut links_map, &start_link_id);

    build_sp(&start_link_id, &end_link_id);
}

pub fn test_bfs() {
    let wiki_name = "dewiki";
    let path = db_wiki_path(wiki_name);
    let path_sp = db_sp_wiki_path(wiki_name);
    let conn = Connection::open(&path).unwrap();

    let t1 = Instant::now();
    let cached_entries: Vec<PageId> = select_link_count_groupby(1000, &wiki_name, "WikiLink.page_id")
        .into_iter().map(|(pid, _)| PageId(pid as u32)).collect();

    let cache = load_link_to_map_db_limit(&path, cached_entries, false);
    // let cache = FxHashMap::default();
    dbg!(&t1.elapsed());

    let t1 = Instant::now();
    let start_link = PageTitle("Taylor_Swift".to_string());
    let start_link_id = sqlite::title_id_conv::page_title_to_id(&start_link, &conn).unwrap();
    dbg!(&start_link_id);

    let conn = Connection::open(&path).unwrap();
    dbg!(&get_links_of_id(&conn, &start_link_id).contains(&PageId(5767)));


    let end_link = PageTitle("Taiwan".to_string());
    let end_link_id = sqlite::title_id_conv::page_title_to_id(&end_link, &conn).unwrap();

    let r =
        bfs(&start_link_id, Some(&end_link_id), None, &cache, &path);
    dbg!(&r.num_visited);
    dbg!(&build_path(&end_link_id, &r.prev_map));

    dbg!(&t1.elapsed());
}

// [lib/src/bench.rs:48:5] &t1.elapsed() = 124.456426095s
// ⠲ [00:00:18] [47,987.8877/s]  922_440/?                                                                                                                                                                                             Found endlink
// [lib/src/bench.rs:59:5] &num_visited = 2_657_172
// [lib/src/bench.rs:60:5] &t1.elapsed() = 18.219622256s


fn test_title_to_id_in_memory_speed() {
    let t1 = Instant::now();
    let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/de_page_db.sqlite";
    let title_id_map = sqlite::title_id_conv::load_title_id_map(path);
    println!("Loading data in memory: {:#?}", t1.elapsed());
    let t2 = Instant::now();

    // 2 seconds
    for _ in (1..10_000).progress() {
        title_id_map.get(&PageTitle("$10.000_Vienna_2007".to_string()));
    }
    println!("Converting titles: {:#?}", t2.elapsed());
    println!("Loading hashmap + converting titles: {:#?}", t1.elapsed() + t2.elapsed());
}


fn test_get_links_db_speed() {
    let conn = Connection::open("/home/gareth/dev/Rust/WikiGame/pagelinks_full_ids.db").unwrap();

    let t1 = Instant::now();
    // 2.5 seconds => 0.4s reusing conn
    for _ in (1..10_000).progress() {
        sqlite::page_links::get_links_of_id(&conn, &PageId(655087));
    }
    dbg!(t1.elapsed());
}

// fn test_title_to_id_in_db_speed() {
//     let t1 = Instant::now();
//     // 3 seconds
//     for _ in (1..10_000).progress() {
//         sqlite::title_id_conv::page_title_to_id(&PageTitle("$10.000_Vienna_2007".to_string()));
//     }
//     dbg!(t1.elapsed());
// }


pub fn test_show_duplicates() {
    // wiki_stats::stats::create_stats().await
    let p = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/downloads/20240301/dewiki-20240301-pagelinks.sql";
    let page_sql = unsafe { memory_map(p).unwrap() };


    let mut set = FxHashSet::default();
    let bar = default_bar(136_343_429 as u64);
    for row in iterate_sql_insertions::<PageLink>(&page_sql).into_iter() {

        // || row.namespace.0 != 0
        // namespace zero are articles (what we want)
        if row.from_namespace.0 != 0 {
            continue;
        }

        let t = (row.from, row.target);
        if set.contains(&t) {
            println!("{:?}", t);
        } else {
            set.insert(t);
        }
        bar.inc(1);
    }
    bar.finish();
}

#[cfg(test)]
mod benchs {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    use test::Bencher;

    use async_stream::stream;
    use futures::{pin_mut, Stream, StreamExt};
    use fxhash::FxBuildHasher;
    use rand::seq::SliceRandom;

    use crate::calc::bfs_bidirectional;
    use crate::sqlite::page_links::load_link_to_map_db_wiki;
    use crate::sqlite::title_id_conv::get_random_page;

    use super::*;

    const WIKI_NAME: &str = "dewiki";

    fn cache() -> &'static HashMap<PageId, Vec<PageId>, FxBuildHasher> {
        dotenv::dotenv().unwrap();
        static CACHE: OnceLock<HashMap<PageId, Vec<PageId>, FxBuildHasher>> = OnceLock::new();
        CACHE.get_or_init(|| {
            let path = db_wiki_path(WIKI_NAME);

            let cache = load_link_to_map_db_wiki(&path);
            // Arc::new(Mutex::new(cache))
            cache
        })
    }

    fn random_pages() -> &'static Vec<u32> {
        static PAGES: OnceLock<Vec<u32>> = OnceLock::new();
        PAGES.get_or_init(|| {
            get_random_page(&WIKI_NAME, 1000).iter().map(|wp| wp.id).collect()
        })
    }


    // #[bench]
    fn bench_bfs(b: &mut Bencher) {
        dotenv::dotenv().unwrap();

        // let cache = FxHashMap::default();
        let mut ids = random_pages().choose_multiple(&mut rand::thread_rng(), 2);
        let start = ids.next().unwrap();
        let end = ids.next().unwrap();
        let cache = cache();
        let path = db_wiki_path(WIKI_NAME);

        b.iter(|| {
            bfs(&PageId(*start), Some(&PageId(*end)), None, cache, path.clone())
        })
    }

    #[tokio::test]
    async fn bench_bfs_bidirectional() {
        dotenv::dotenv().unwrap();

        let path = db_wiki_path(WIKI_NAME);

        let mut ids = random_pages().choose_multiple(&mut rand::thread_rng(), 2);
        let start = ids.next().unwrap();
        let end = ids.next().unwrap();
        

        let s = PageId(*start);
        let e = PageId(*end);
        let s = bfs_bidirectional(s, e, path.clone()).await;
        pin_mut!(s); // needed for iteration
        while let Some(v) = s.next().await {
            dbg!(&v);
        }
    }
}