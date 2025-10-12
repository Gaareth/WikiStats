use std::fmt::Debug;
use std::{
    cmp::Ordering,
    collections::HashSet,
    path::{Path, PathBuf},
};

use parse_mediawiki_sql::field_types::PageId;
use rusqlite::Connection;
use tokio::{
    join,
    time::{self, Instant},
};

use crate::download::ALL_DB_TABLES;
use crate::stats::stats::{Page, get_local_wiki_sizes};
use crate::{
    WikiIdent, create_wiki_idents,
    sqlite::title_id_conv::page_id_to_title,
    stats::{
        io::{save_stats, try_load_stats},
        queries::{
            get_num_dead_orphan_pages, get_num_dead_pages, get_num_linked_redirects,
            get_num_orphan_pages, longest_name, select_link_count_groupby,
        },
        stats::{LinkCount, StatRecord, num_links_stat, num_pages_stat, num_redirects_stat},
        utils::{GLOBAL, make_stat_record, max_min_value_record},
    },
};

mod io;
pub mod queries;
mod samples;
pub mod stats;
mod utils;

pub use io::{add_sample_bfs_stats, add_sample_bibfs_stats, add_web_wiki_sizes};
pub use stats::Stats;

pub async fn create_stats(
    path: impl AsRef<Path>,
    wikis: Vec<String>,
    database_path: impl Into<PathBuf>,
    dump_date: impl Into<String>,
) {
    fn global_adder(record: &mut StatRecord<u64>) {
        let v = record.iter().fold(0, |acc, (_, value)| acc + value);
        record.insert(GLOBAL.to_string(), v);
    }

    let dump_date = dump_date.into();
    let path = path.as_ref();
    let database_path = database_path.into();
    let existing_stats: Option<Stats> = try_load_stats(path);

    let base_path = database_path
        .clone()
        .parent()
        .expect("Failed extracting base path from db path")
        .to_path_buf();

    // Extract needed fields from stats before moving it
    let num_pages_prev = existing_stats.as_ref().map(|s| s.num_pages.clone());
    let num_redirects_prev = existing_stats.as_ref().map(|s| s.num_redirects.clone());
    let num_links_prev = existing_stats.as_ref().map(|s| s.num_links.clone());
    let most_linked_prev = existing_stats.as_ref().map(|s| s.most_linked.clone());
    let most_links_prev = existing_stats.as_ref().map(|s| s.most_links.clone());
    let longest_name_prev = existing_stats.as_ref().map(|s| s.longest_name.clone());
    let longest_name_no_redirect_prev = existing_stats
        .as_ref()
        .map(|s| s.longest_name_no_redirect.clone());
    let num_dead_pages_prev = existing_stats.as_ref().map(|s| s.num_dead_pages.clone());
    let num_orphan_pages_prev = existing_stats.as_ref().map(|s| s.num_orphan_pages.clone());
    let num_dead_orphan_pages_prev = existing_stats
        .as_ref()
        .map(|s| s.num_dead_orphan_pages.clone());
    let num_linked_redirects_prev = existing_stats
        .as_ref()
        .map(|s| s.num_linked_redirects.clone());

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

    let merged_wikis: Vec<String> = {
        let mut existing_wikis: HashSet<String> = existing_stats
            .as_ref()
            .map(|s| s.wikis.iter().cloned().collect())
            .unwrap_or_default();
        existing_wikis.extend(wikis.into_iter());
        existing_wikis.into_iter().collect()
    };

    let local_wiki_sizes = Some(get_local_wiki_sizes(&base_path, &ALL_DB_TABLES).await);

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
        wikis: merged_wikis,
        seconds_taken: time_taken.as_secs(),

        // bfs_sample_stats: Some(bfs_sample_stats.await),
        // bi_bfs_sample_stats: Some(bi_bfs_sample_stats.await),
        bfs_sample_stats: existing_stats
            .as_ref()
            .and_then(|s| s.bfs_sample_stats.clone()),
        bi_bfs_sample_stats: existing_stats
            .as_ref()
            .and_then(|s| s.bi_bfs_sample_stats.clone()),
        web_wiki_sizes: existing_stats.and_then(|s| s.web_wiki_sizes),
        local_wiki_sizes,
    };

    save_stats(&stats, path);
    println!(
        "Done generating stats. Total time elapsed: {:?}",
        time_taken
    );
}
