use crate::{stats::{stats::{Page, WikiName}}, utils::default_bar_unknown, WikiIdent};
use std::path::Path;
use fxhash::FxHashSet;
use parse_mediawiki_sql::field_types::PageId;
use rusqlite::Connection;
use tokio::time::Instant;

pub fn count_from(table_name: &str, db_path: impl AsRef<Path>, where_str: &str) -> u64 {
    let conn = Connection::open(db_path).unwrap();
    let stmt = format!("select count(*) from {table_name} {where_str}");
    conn.query_row(&stmt, [], |row| row.get(0)).unwrap()
}

pub fn get_dead_pages(wiki_ident: WikiIdent) -> Vec<Page> {
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

pub fn get_num_dead_pages(wiki_ident: WikiIdent) -> u64 {
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

pub fn get_num_orphan_pages(wiki_ident: WikiIdent) -> u64 {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let stmt = "select count(page_id) from WikiPage where page_id not in (select page_link from WikiLink);".to_string();

    let res = query_count(&stmt, &db_path);
    println!("DONE num root pages {:?}: {name:?}", t1.elapsed());

    res
}

pub fn get_num_dead_orphan_pages(wiki_ident: WikiIdent) -> u64 {
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

pub fn get_dead_orphan_pages(wiki_ident: WikiIdent) -> Vec<Page> {
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

pub fn get_num_linked_redirects(wiki_ident: WikiIdent) -> u64 {
    let t1 = Instant::now();
    let name = wiki_ident.wiki_name;
    let db_path = wiki_ident.db_path;

    let stmt = "select count(*) from WikiLink where page_link in (select page_id from WikiPage where is_redirect = 1);".to_string();

    let res = query_count(&stmt, &db_path);
    println!("DONE num linked redirects {:?}: {name:?}", t1.elapsed());

    res
}

pub fn longest_name(wiki_ident: WikiIdent, redirects: bool) -> Page {
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

pub fn query_page(stmt_str: &str, db_path: impl AsRef<Path>, wiki_name: WikiName) -> Vec<Page> {
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
pub fn top_linked_ids(top: usize, wiki_name: Option<&str>, db_path: impl AsRef<Path>) -> FxHashSet<PageId> {
    let mut link_count = FxHashSet::default();

    let where_wiki = wiki_name
        .map(|wiki_name| format!("WHERE wiki_name = '{wiki_name}'"))
        .unwrap_or_default();

    let conn = Connection::open(db_path).unwrap();
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
pub fn top_link_ids(top: usize, db_path: impl AsRef<Path>) -> FxHashSet<PageId> {
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
