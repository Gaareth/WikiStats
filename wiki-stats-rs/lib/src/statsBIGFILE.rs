use std::collections::HashSet;
use std::fmt::Debug;
use std::{fs, thread};

use futures::StreamExt;
use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use parse_mediawiki_sql::field_types::PageId;
use rusqlite::Connection;
use serde::Serialize;
use tokio::join;
use tokio::time::Instant;

use crate::sqlite::title_id_conv::page_id_to_title;
use crate::utils::default_bar_unknown;

static DB_STATS: &str = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/20240301/es_database2.sqlite";
static wikis: [&str; 1] = ["eswiki"];

static GLOBAL: &str = "global";

fn count_from(table_name: &str, wiki_name: Option<&str>) -> (u64, String) {
    dbg!(&wiki_name);
    let mut conn = Connection::open(DB_STATS).unwrap();
    if let Some(wiki_name) = wiki_name {
        let stmt = format!("select count(*) from {table_name} where wiki_name = ?1");
        (conn.query_row(&stmt, [wiki_name], |row| row.get(0)).unwrap(), wiki_name.to_string())
    } else {
        let stmt = format!("select count(*) from {table_name}");
        (conn.query_row(&stmt, [], |row| row.get(0)).unwrap(), GLOBAL.to_string())
    }

    // return 2;
}


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

async fn make_stat_record<T: Debug + Send + 'static>(func: fn(Option<String>) -> (T, String)) -> StatRecord<T> {
    let mut tids = vec![];
    let mut record = FxHashMap::default();
    for wiki in wikis {
        tids.push(thread::spawn(move || func(Some(wiki.to_string().clone()))));
    }

    tids.push(thread::spawn(move || func(None)));


    tids
        .into_iter()
        .for_each(|th| {
            let (res, wname) = th.join().expect("can't join thread");
            record.insert(wname, res);
        });
    record
}

type StatRecord<T> = FxHashMap<String, T>;

type WikiName = String;
type PageTitle = String;

#[derive(Serialize)]
struct Stats {
    num_pages: StatRecord<u64>,
    num_links: StatRecord<u64>,
    most_linked: StatRecord<Vec<LinkCount>>,
    most_links: StatRecord<Vec<LinkCount>>,

    longest_name: StatRecord<Page>,
    num_dead_pages: StatRecord<u64>,
    num_root_pages: StatRecord<u64>,

    max_num_pages: (WikiName, u64),
    min_num_pages: (WikiName, u64),
    max_num_links: (WikiName, u64),
    min_num_links: (WikiName, u64),
}

#[derive(Serialize, Debug)]
struct LinkCount {
    page_title: PageTitle,
    page_id: u64,
    wiki_name: WikiName,
    count: u64,
}

#[derive(Serialize, Debug)]
struct Page {
    page_title: PageTitle,
    page_id: u64,
    wiki_name: WikiName,
}


pub async fn create_stats() {
    fn num_pages_stat(name: Option<String>) -> (u64, String) {
        count_from("WikiPage", name.as_deref())
    }

    fn num_links_stat(name: Option<String>) -> (u64, String) {
        count_from("WikiLink", name.as_deref())
    }

    let pages_stat_future = make_stat_record(num_pages_stat);
    // dbg!(&pages_stat_future.await);

    let link_stat_future = make_stat_record(num_links_stat);
    // dbg!(&link_stat_future.await);

    fn top_ten_linked(name: Option<String>) -> (Vec<LinkCount>, String) {
        let t1 = Instant::now();
        println!("Top linked: {name:?}");
        let res = (select_link_count_groupby(10, name.as_deref(), "WikiLink.page_link"), name.clone().unwrap_or(GLOBAL.to_string()));
        println!("DONE. {:?} Top linked: {name:?}", t1.elapsed());

        res
    }

    fn top_ten_links(name: Option<String>) -> (Vec<LinkCount>, String) {
        let t1 = Instant::now();
        println!("Top links: {name:?}");
        let res = select_link_count_groupby(10, name.as_deref(), "WikiLink.page_id");
        println!("DONE. {:?} Top links: {name:?}", t1.elapsed());


        (res, name.unwrap_or(GLOBAL.to_string()))
    }

    let most_linked_future = make_stat_record(top_ten_linked);
    // dbg!(&most_linked_future.await);
    let most_links_future = make_stat_record(top_ten_links);

    let longest_name_future = make_stat_record(longest_name);

    fn get_dead_pages(name: Option<String>) -> (Vec<Page>, String) {
        let t1 = Instant::now();
        let res = query_page("select * from WikiPage where page_id not in (select page_id from WikiLink);", name.clone());
        println!("DONE dead pages {:?}: {name:?}", t1.elapsed());
        (res, name.unwrap_or(GLOBAL.to_string()))
    }

    fn get_root_pages(name: Option<String>) -> (Vec<Page>, String) {
        let t1 = Instant::now();

        let res = query_page("select * from WikiPage where page_id not in (select page_link from WikiLink);", name.clone());
        println!("DONE root pages {:?}: {name:?}", t1.elapsed());
        (res, name.unwrap_or(GLOBAL.to_string()))
    }

    fn get_num_dead_pages(name: Option<String>) -> (u64, String) {
        let t1 = Instant::now();

        let stmt = if let Some(name) = name.clone() {
            format!("select count(page_id) from WikiPage where wiki_name = '{name}' AND page_id not in (select page_id from WikiLink);")
        } else {
            "select count(page_id) from WikiPage where page_id not in (select page_id from WikiLink);".to_string()
        };
        let res = query_count(&stmt);
        println!("DONE num dead pages {:?}: {name:?}", t1.elapsed());

        (res, name.unwrap_or(GLOBAL.to_string()))
    }

    fn get_num_root_pages(name: Option<String>) -> (u64, String) {
        let t1 = Instant::now();

        let stmt = if let Some(name) = name.clone() {
            format!("select count(page_id) from WikiPage where wiki_name = '{name}' AND page_id not in (select page_link from WikiLink);")
        } else {
            "select count(page_id) from WikiPage where page_id not in (select page_link from WikiLink);".to_string()
        };

        let res = query_count(&stmt);
        println!("DONE num root pages {:?}: {name:?}", t1.elapsed());

        (res, name.unwrap_or(GLOBAL.to_string()))
    }

    let num_dead_pages = make_stat_record(get_num_dead_pages);

    let num_root_pages = make_stat_record(get_num_root_pages);

    async fn max_min_table_wiki(table_name: &str) -> ((String, u64), (String, u64)) {
        let t1 = Instant::now();


        let conn = Connection::open(DB_STATS).unwrap();
        let stmt_str = format!("\
            SELECT wiki_name, count(wiki_name) \
            FROM {table_name} \
            GROUP BY wiki_name \
            ORDER BY count(wiki_name) DESC");
        let mut stmt = conn.prepare(&stmt_str).unwrap();

        let mut rows = stmt.query_map(
            [], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap();


        let max = rows.next().unwrap().unwrap();
        let min = rows.last().unwrap_or(Ok(max.clone())).unwrap();

        println!("DONE max min pages {:?} {table_name}", t1.elapsed());

        (max, min)
    }

    let max_min_pages_future = max_min_table_wiki("WikiPage");
    let max_min_links_future = max_min_table_wiki("WikiLink");

    // dbg!(&links_groupby_having(Some("ja".to_string()), "WikiLink.page_link"));
    // dbg!(&get_num_dead_pages(Some("ja".to_string())));

    // dbg!(&top_ten_links(Some("dewiki".to_string())));

    // dbg!(&max_min_pages_future.await);
    // dbg!(&max_min_links_future.await);

    let (
        num_pages,
        num_links,
        most_linked,
        most_links,
        longest_name,
        num_dead_pages,
        num_root_pages,
        max_min_pages,
        max_min_links
    ) = join!(
        tokio::spawn(pages_stat_future),
        tokio::spawn(link_stat_future),
        tokio::spawn(most_linked_future),
        tokio::spawn(most_links_future),
        tokio::spawn(longest_name_future),
        tokio::spawn(num_dead_pages),
        tokio::spawn(num_root_pages),
        tokio::spawn(max_min_pages_future),
        tokio::spawn(max_min_links_future)
    );

    let (max_num_pages, min_num_pages) = max_min_pages.unwrap();
    let (max_num_links, min_num_links) = max_min_links.unwrap();

    let stats = Stats {
        num_pages: num_pages.unwrap(),
        num_links: num_links.unwrap(),
        most_linked: most_linked.unwrap(),
        most_links: most_links.unwrap(),
        longest_name: longest_name.unwrap(),
        num_dead_pages: num_dead_pages.unwrap(),
        num_root_pages: num_root_pages.unwrap(),

        max_num_pages,
        min_num_pages,
        max_num_links,
        min_num_links,
    };
    let json = serde_json::to_string_pretty(&stats).unwrap();
    dbg!(&json);
    fs::write("./stats.json", json);


    // top_ten_linked(None);
    // dbg!(&top_linked_titles2(10, None));

    // let (
    //     most_linked,
    //     most_links
    // ) = join!(
    //     tokio::spawn(most_linked_future),
    //     tokio::spawn(most_links_future),
    // );
    //
    // // let stats = Stats {
    // //     most_linked: tokio::spawn(most_linked_future).await.unwrap(),
    // //     most_links: tokio::spawn(most_links_future).await.unwrap(),
    // // };
    //
    // let stats = Stats {
    //     most_linked: most_linked.unwrap(),
    //     most_links: most_links.unwrap(),
    // };
    //


    // let mut tids = [
    //     thread::spawn(|| count_from("WikiLink", Some("ja"))),
    //     // thread::spawn(|| count_from("WikiPage", None))
    // ];
    // let wikilinks_task = tokio::spawn(f("a".to_string()));
    // let wikilinks_de_task = tokio::spawn(count_from("WikiLink", Some("ja")));
    //
    //
    // let wikilinks = wikilinks_task.await.unwrap();
    // let wikilinks_de = wikilinks_de_task.await.unwrap();
    // dbg!(&wikilinks);
    // dbg!(&wikilinks_de);
    // tids
    //     .into_iter()
    //     .for_each(|th| {
    //         let res = th.join().expect("can't join thread");
    //         println!("{}", res)
    //     });
}


fn longest_name(wiki_name_opt: Option<String>) -> (Page, WikiName) {
    let t1 = Instant::now();
    let where_wiki = wiki_name_opt.clone().map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'")).unwrap_or_default();
    let stmt_str = format!("SELECT page_title, page_id, wiki_name FROM WikiPage {where_wiki} ORDER BY length(page_title) DESC LIMIT 1");

    let conn = Connection::open(DB_STATS).unwrap();

    let (page_title, page_id, wiki_name) = conn
        .query_row(&stmt_str, [], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap()))).unwrap();

    let page = Page {
        page_title,
        page_id,
        wiki_name,
    };
    println!("DONE longest name {:?}: {:?}", t1.elapsed(), wiki_name_opt);

    (page, wiki_name_opt.unwrap_or(GLOBAL.to_string()))
}

fn query_count(stmt_str: &str) -> u64 {
    let conn = Connection::open(DB_STATS).unwrap();
    let mut stmt = conn.prepare(stmt_str).unwrap();
    // dbg!(&stmt);

    stmt.query_row(
        [], |row| Ok(row.get(0).unwrap())).unwrap()
}

fn query_page(stmt_str: &str, where_wiki_name: Option<String>) -> Vec<Page> {
    let conn = Connection::open(DB_STATS).unwrap();
    // let stmt_str = format!("SELECT {group_by}, WikiLink.wiki_name FROM WikiLink \
    //     GROUP BY {group_by} HAVING {having_str}");
    dbg!(&stmt_str);
    let mut stmt = conn.prepare(stmt_str).unwrap();

    let rows = stmt.query_map(
        [], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap();

    let mut res = vec![];
    let mut bar = default_bar_unknown();

    for row in rows {
        let (page_id, wiki_name): (u64, String) = row.unwrap();
        bar.inc(1);

        if let Some(ref wname) = where_wiki_name {
            if wname != &wiki_name {
                continue;
            }
        }

        let page_title = page_id_to_title(PageId(page_id as u32), where_wiki_name.as_deref()).unwrap().0;
        // let page_title = String::new();
        res.push(Page {
            page_title,
            page_id,
            wiki_name,
        });
    }
    bar.finish();
    res
}


fn links_groupby_having(where_wiki_name: Option<String>, group_by: &str, having_str: &str) -> Vec<Page> {
    let conn = Connection::open(DB_STATS).unwrap();
    let stmt_str = format!("SELECT {group_by}, WikiLink.wiki_name FROM WikiLink \
        GROUP BY {group_by} HAVING {having_str}");
    dbg!(&stmt_str);
    let mut stmt = conn.prepare(&stmt_str).unwrap();

    let rows = stmt.query_map(
        [], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap();

    let mut res = vec![];
    let mut bar = default_bar_unknown();

    for row in rows {
        let (page_id, wiki_name): (u32, String) = row.unwrap();
        bar.inc(1);

        // if let Some(ref wname) = where_wiki_name {
        //     if wname != &wiki_name {
        //         continue;
        //     }
        // }
        //
        // let page_title = page_id_to_title(PageId(page_id as u32)).unwrap().0;
        // res.push(Page {
        //     page_title,
        //     page_id,
        //     wiki_name,
        // });
    }
    bar.finish();
    res
}


/// returns ids of pages with the most links
pub fn select_link_count_groupby(top: usize, where_wiki_name: Option<&str>, groupby: &str) -> Vec<LinkCount> {
    let t1 = Instant::now();

    let mut link_count = vec![];

    let where_wiki = where_wiki_name.map(|wiki_name| format!("WHERE WikiLink.wiki_name = '{wiki_name}'")).unwrap_or_default();

    let conn = Connection::open(DB_STATS).unwrap();
    let mut stmt = conn.prepare(&format!("SELECT {groupby}, WikiLink.wiki_name, COUNT(*) FROM WikiLink \
            {where_wiki}
            GROUP BY {groupby} ORDER BY count(*) DESC LIMIT {top}")).unwrap();
    // stmt.execute([top]).unwrap();
// WikiLink.page_id
    let rows = stmt.query_map(
        [], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap()))).unwrap();


    for row in rows {
        let (page_id, wiki_name, count) = row.unwrap();

        // if let Some(wname) = where_wiki_name {
        //     if wname != wiki_name {
        //         continue;
        //     }
        // }

        let page_title = page_id_to_title(PageId(page_id as u32), where_wiki_name).unwrap().0;
        link_count.push(LinkCount {
            page_title,
            page_id,
            wiki_name,
            count,
        });
    }

    // println!("top links  {:?}", t1.elapsed());
    // dbg!(&link_count);
    link_count
}


/// return ids of the most linked page. linked by other pages the most times
pub fn top_linked_ids(top: usize, wiki_name: Option<&str>) -> HashSet<PageId, FxBuildHasher> {
    let mut link_count = FxHashSet::default();

    let where_wiki = wiki_name.map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'")).unwrap_or_default();

    let conn = Connection::open(DB_STATS).unwrap();
    let stmt_str = format!("SELECT page_link, COUNT(*) FROM WikiLink \
            {where_wiki} GROUP BY page_link ORDER BY count(*) DESC LIMIT {top}");
    // dbg!(&stmt_str);
    let mut stmt = conn.prepare(&stmt_str).unwrap();

    let rows = stmt.query_map(
        [], |row| Ok(row.get(0).unwrap())).unwrap();

    println!("query done");
    let mut bar = default_bar_unknown();

    for row in rows {
        link_count.insert(PageId(row.unwrap()));
        bar.inc(1);
    }
    // bar.finish();

    // dbg!(&link_count.len());
    link_count
}

/// returns ids of pages with the most links
pub fn top_link_ids(top: usize, wiki_name: Option<&str>) -> HashSet<PageId, FxBuildHasher> {
    let mut link_count = FxHashSet::default();

    let where_wiki = wiki_name.map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'")).unwrap_or_default();

    let conn = Connection::open(DB_STATS).unwrap();
    let mut stmt = conn.prepare(&format!("SELECT page_id, COUNT(*) FROM WikiLink \
            {where_wiki} GROUP BY page_id ORDER BY count(*) DESC LIMIT {top}")).unwrap();
    // stmt.execute([top]).unwrap();

    let rows = stmt.query_map(
        [], |row| Ok(row.get(0).unwrap())).unwrap();


    for row in rows {
        link_count.insert(PageId(row.unwrap()));
    }

    link_count
}
