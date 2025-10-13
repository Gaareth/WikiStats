use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

use chrono::{DateTime, Utc};
use colored::Colorize as _;
use log::{debug, info, warn};
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use parse_mediawiki_sql::schemas::PageLink;
use parse_mediawiki_sql::utils::{Mmap, memory_map};
use rusqlite::Connection;

use crate::sqlite::load::{
    load_links_map, load_linktarget_map, load_map, load_sql_full, load_sql_part_map,
};
use crate::sqlite::page_links::{get_incoming_links_of_id, get_links_of_id};
use crate::sqlite::to_sqlite::INFO_TABLE;
use crate::web::{
    WikipediaApiError, get_added_diff_to_current, get_creation_date, get_deleted_diff_to_current,
    get_latest_revision_id_before_date, get_links_on_webpage,
};
use crate::{parse_dump_date, web};

use anyhow::Result;
use futures::FutureExt;
use futures::future::BoxFuture;
use std::future::Future; // for .boxed()

/// Get the diff from the the wikipedia page at dumpdate til the current
async fn get_or_insert_diff<F>(
    map: &mut HashMap<String, Vec<String>>,
    revid_map: &mut HashMap<String, u64>,
    page: &str,
    wiki_prefix: &str,
    dump_date: &DateTime<Utc>,
    get_fn: F,
) -> anyhow::Result<(Vec<String>, Option<u64>)>
where
    F: for<'a> Fn(&'a str, &'a str, u64) -> BoxFuture<'a, Result<Vec<String>, WikipediaApiError>>
        + Send
        + Sync,
{
    let rev_id_opt = match revid_map.entry(page.to_string()) {
        Entry::Occupied(o) => Some(*o.get()),
        Entry::Vacant(v) => {
            let rev_id_opt = get_latest_revision_id_before_date(page, wiki_prefix, dump_date)
                .await
                .map_err(anyhow::Error::new)?;
            if let Some(rev_id) = rev_id_opt {
                v.insert(rev_id);
            }
            rev_id_opt
        }
    };

    if let Some(diff) = map.get(page) {
        return Ok((diff.clone(), rev_id_opt));
    }

    let diff = if let Some(rev_id) = rev_id_opt {
        get_fn(page, wiki_prefix, rev_id)
            .await
            .map_err(anyhow::Error::new)?
    } else {
        vec![]
    };

    map.insert(page.to_string(), diff.clone());
    Ok((diff, rev_id_opt))
}

async fn compare_links(
    page_title: &str,
    wiki_prefix: &str,
    dump_date: &DateTime<Utc>,
    expected_results: &HashSet<PageTitle>,
    db_results: &HashSet<PageTitle>,
    link_type: &str,
) -> bool {
    let mut success = true;

    debug!("expected: {:?}", expected_results);
    debug!("actual: {:?}", db_results);

    let mut revid_map: HashMap<String, u64> = HashMap::new();
    let mut links_on_old_page_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut diffs_added_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut diffs_deleted_map: HashMap<String, Vec<String>> = HashMap::new();

    // Links missing in DB
    for diff in expected_results.difference(db_results) {
        let ok = check_missing_link(
            page_title,
            diff.0.as_str(),
            link_type,
            wiki_prefix,
            dump_date,
            &mut diffs_added_map,
            &mut revid_map,
            &mut links_on_old_page_map,
        )
        .await;
        if !ok {
            success = false;
        }
    }

    // Links that exist in DB but not online
    for diff in db_results.difference(expected_results) {
        let ok = check_outdated_link(
            page_title,
            diff.0.as_str(),
            link_type,
            wiki_prefix,
            dump_date,
            &mut diffs_deleted_map,
            &mut revid_map,
            &mut links_on_old_page_map,
        )
        .await;
        if !ok {
            success = false;
        }
    }

    success
}

async fn check_missing_link(
    page_title: &str,
    other: &str,
    link_type: &str,
    wiki_prefix: &str,
    dump_date: &DateTime<Utc>,
    diffs_added_map: &mut HashMap<String, Vec<String>>,
    revid_map: &mut HashMap<String, u64>,
    links_cache: &mut HashMap<String, Vec<String>>,
) -> bool {
    let (from, to) = if link_type == "incoming" {
        (other, page_title)
    } else {
        (page_title, other)
    };

    // check if the to link was added after the dump to the from page
    let (diff_added, rev_id_opt) = get_or_insert_diff(
        diffs_added_map,
        revid_map,
        from,
        wiki_prefix,
        dump_date,
        |page, prefix, rev_id| get_added_diff_to_current(page, prefix, rev_id).boxed(),
    )
    .await
    .unwrap();

    if diff_added.iter().any(|d| d.contains(to)) {
        println!(
            "[{}] Link {} -> {} missing from db but added after {dump_date}",
            link_type, from, to
        );
        return true;
    }

    // if an incoming link is missing check if the from page was created after the dumpdate
    if link_type == "incoming" {
        if let Some(creation_date) = get_creation_date(from, wiki_prefix).await.unwrap() {
            if &creation_date > dump_date {
                println!(
                    "[{}] Link {} -> {} missing from db but created after {dump_date}",
                    link_type, from, to
                );
                return true;
            }
        }
    }

    // check if the old wikipedia page even contains this link
    // the problem is that the diff does not render links inside templates, so we need to look at the actual html
    if let Some(rev_id) = rev_id_opt {
        let links_on_old_page = get_or_cache_links(from, wiki_prefix, rev_id, links_cache).await;
        if !link_exists(&links_on_old_page, to) {
            return true;
        }
    }

    eprintln!(
        "{}",
        format!("[{}] Link {} -> {} missing from db", link_type, from, to).red()
    );

    if link_type == "incoming" {
        info!(
            "{}",
            format!(
                "https://{}.wikipedia.org/wiki/{}",
                wiki_prefix,
                urlencoding::encode(from)
            )
        );
    }

    false
}

async fn check_outdated_link(
    page_title: &str,
    other: &str,
    link_type: &str,
    wiki_prefix: &str,
    dump_date: &DateTime<Utc>,
    diffs_deleted_map: &mut HashMap<String, Vec<String>>,
    revid_map: &mut HashMap<String, u64>,
    links_cache: &mut HashMap<String, Vec<String>>,
) -> bool {
    let (from, to) = if link_type == "incoming" {
        (other, page_title)
    } else {
        (page_title, other)
    };

    let (diff_deleted, rev_id_opt) = get_or_insert_diff(
        diffs_deleted_map,
        revid_map,
        from,
        wiki_prefix,
        dump_date,
        |page, prefix, rev_id| get_deleted_diff_to_current(page, prefix, rev_id).boxed(),
    )
    .await
    .unwrap();

    if diff_deleted.iter().any(|d| d.contains(to)) {
        println!(
            "[{}] Link {} -> {} missing from db but seems to be deleted after {dump_date} db dump",
            link_type, from, to
        );
        return true;
    }

    // TODO: check if from or to was deleted after dumpdate

    if let Some(rev_id) = rev_id_opt {
        let links_on_old_page = get_or_cache_links(from, wiki_prefix, rev_id, links_cache).await;
        if !link_exists(&links_on_old_page, to) {
            return true;
        }
    }

    eprintln!(
        "{}",
        format!(
            "[{}] Link {} -> {} in db but not online (outdated?)",
            link_type, from, to
        )
        .red()
    );

    if link_type == "incoming" {
        info!(
            "{}",
            format!(
                "https://{}.wikipedia.org/wiki/{}",
                wiki_prefix,
                urlencoding::encode(from)
            )
        );
    }

    false
}

// Utility: fetch links, with caching
async fn get_or_cache_links(
    page: &str,
    wiki_prefix: &str,
    rev_id: u64,
    cache: &mut HashMap<String, Vec<String>>,
) -> Vec<String> {
    match cache.entry(page.to_string()) {
        Entry::Occupied(o) => o.get().clone(),
        Entry::Vacant(v) => {
            let links = get_links_on_webpage(page, wiki_prefix, rev_id)
                .await
                .unwrap();
            v.insert(links.clone());
            links
        }
    }
}

// Utility: compare link presence (handles underscores)
fn link_exists(links: &[String], to: &str) -> bool {
    links.contains(&to.to_string()) || links.contains(&to.replace(" ", "_"))
}

pub fn check_is_done(conn: &Connection) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM Info WHERE is_done = 1)",
        [],
        |row| {
            let v: i64 = row.get(0)?;
            Ok(v != 0)
        },
    )
}

pub fn check_is_validated(conn: &Connection) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM Info WHERE is_validated = 1)",
        [],
        |row| {
            let v: i64 = row.get(0)?;
            Ok(v != 0)
        },
    )
}

pub async fn post_validation(
    db_file: impl AsRef<Path>,
    dump_date: impl AsRef<str>,
    wiki_prefix: impl AsRef<str>,
    pages_to_test: &[PageTitle],
) -> bool {
    let t1 = Instant::now();

    let db_file = db_file.as_ref();
    if !fs::exists(&db_file).unwrap() {
        eprintln!("DB File {db_file:?} does not exist");
        return false;
    }

    let conn = Connection::open(db_file).unwrap();
    if !check_is_done(&conn).unwrap_or(false) {
        eprintln!("DB File {db_file:?} is not done");
        return false;
    }

    let wiki_prefix = wiki_prefix.as_ref();
    let dump_date = parse_dump_date(dump_date.as_ref()).expect("Failed parsing dumpdate");

    let mut success = true;

    // removes links that don't exist and converts from pageid -> pagetitle
    let filter_broken_pids = |pids: Vec<PageId>| async {
        let mut filtered = HashSet::new();
        for pid in pids {
            if let Some(pinfo) = web::get_page_info_by_id(pid.0 as u64, &wiki_prefix)
                .await
                .unwrap()
            {
                filtered.insert(PageTitle(pinfo.title));
            } else {
                warn!("DB contains link to {pid:?} but this pid seems not to exist");
            }
        }
        filtered
    };

    // removes links that don't exist
    let convert_pid_pt = |pts: Vec<PageTitle>| async {
        let mut filtered = HashSet::new();
        for pt in pts {
            if let Some(pinfo) = web::get_page_info_by_title(&pt.0, &wiki_prefix)
                .await
                .unwrap()
            {
                filtered.insert(PageTitle(pinfo.title));
            } else {
                warn!("webpage contains link to {pt:?} but this pagetitle seems not to exist");
            }
        }
        filtered
    };

    let mut pages = vec![];
    for pt in pages_to_test {
        let pid = web::get_page_info_by_title(&pt.0, wiki_prefix)
            .await
            .unwrap()
            .expect(&format!("No pageid for pagetitle {pt:?}"))
            .pageid;
        pages.push((pt.0.as_str(), pid as u32));
    }

    for (page_title, page_id) in pages {
        let page_url = format!(
            "https://{}.wikipedia.org/wiki/{}",
            &wiki_prefix,
            urlencoding::encode(&page_title)
        );
        info!("\n Checking page: {:?}   |   {}", &page_title, page_url);
        info!("Checking incoming links..");

        // ### [incoming links]
        // online results
        let all_links = web::get_incoming_links(&page_title, &wiki_prefix)
            .await
            .unwrap()
            .into_iter()
            .map(|link| PageId(link.pageid))
            .collect();
        // filter to only include links that exist
        let expected_incoming: HashSet<_> = filter_broken_pids(all_links).await;

        // database results
        let db_incoming: HashSet<PageTitle> =
            filter_broken_pids(get_incoming_links_of_id(&conn, &PageId(page_id))).await;
        success &= compare_links(
            &page_title,
            &wiki_prefix,
            &dump_date,
            &expected_incoming,
            &db_incoming,
            "incoming",
        )
        .await;

        info!("Checking outgoing links..");
        // ### [outgoing] links
        // online results
        let all_links = web::get_outgoing_links(&page_title, &wiki_prefix)
            .await
            .unwrap()
            .into_iter()
            .map(|link| PageTitle(link.title))
            .collect();
        // filter to only include links that exist
        let expected_outgoing: HashSet<_> = convert_pid_pt(all_links).await;

        // database results
        let db_outgoing: HashSet<PageTitle> =
            filter_broken_pids(get_links_of_id(&conn, &PageId(page_id))).await;
        success &= compare_links(
            &page_title,
            &wiki_prefix,
            &dump_date,
            &expected_outgoing,
            &db_outgoing,
            "outgoing",
        )
        .await;
    }

    conn.execute(INFO_TABLE, ()).unwrap();

    let r =conn.execute(
            "UPDATE Info SET is_validated = ?, num_pages_validated = ?, validation_time_s = ? WHERE id = 0",
            (success, pages_to_test.len(),t1.elapsed().as_secs_f64()),
        );
    if let Err(e) = r {
        log::error!("{}", format!("Error setting is_validated: {e}"));
    }

    success
}

/// Currently only checks outgoing links
pub async fn pre_validation(
    pl_sql_file_path: impl AsRef<Path>,
    lt_sql_file_path: impl AsRef<Path>,
    page_ids_to_check: &[PageId],
    dump_date: impl AsRef<str>,
) -> bool {
    let pl_sql_file_path = pl_sql_file_path.as_ref();
    let filename = &pl_sql_file_path.file_name().unwrap().to_str().unwrap();
    let wikiname = filename.split("-").next().unwrap();
    let wikiprefix = &wikiname[..2];
    let dump_date = parse_dump_date(dump_date.as_ref()).expect("Failed parsing dumpdate");

    let pl_mmap: Mmap = unsafe { memory_map(pl_sql_file_path).unwrap() };
    let pl_map = load_links_map::<_, _, PageLink, _, _>(
        &pl_mmap,
        |pl| (pl.from, pl.target),
        |pl| pl.from_namespace.0 != 0 || !page_ids_to_check.contains(&pl.from),
    );

    let lt_mmap: Mmap = unsafe { memory_map(lt_sql_file_path).unwrap() };
    let lt_map = load_linktarget_map(lt_mmap);

    println!("Loaded map");

    let mut success = true;

    let convert_pid_pt = move |pid: PageId| async move {
        let pageinfo: web::PageInfo = web::get_page_info_by_id(pid.0 as u64, &wikiprefix)
            .await
            .unwrap()
            .unwrap();
        pageinfo.title
    };

    for (page_id, link_targets) in pl_map.iter() {
        let page_title = convert_pid_pt(*page_id).await;

        dbg!(&page_id);

        let expected_results: HashSet<PageTitle> = HashSet::from_iter(
            web::get_outgoing_links_by_id(page_id.0, wikiprefix)
                .await
                .unwrap()
                .into_iter()
                .map(|link| link.title.replace(" ", "_").into()),
        );

        let sqlfile_results = HashSet::from_iter(
            link_targets
                .iter()
                .filter_map(|lt| lt_map.get(&lt))
                .cloned(),
        );

        success &= compare_links(
            &page_title,
            &wikiprefix,
            &dump_date,
            &expected_results,
            &sqlfile_results,
            "outgoing",
        )
        .await;
    }

    success
}

#[cfg(test)]
mod tests {
    use parse_mediawiki_sql::field_types::PageTitle;

    use crate::validate::post_validation;
    use crate::web;

    #[tokio::test]
    async fn test_post_validation() {
        let random_pages = web::get_random_wikipedia_pages(2, "pwn").await.unwrap();

        let res = post_validation(
            "tests/data/small/test_database.sqlite",
            "20240901",
            "pwn",
            &random_pages
                .into_iter()
                .map(|p| PageTitle(p.title))
                .collect::<Vec<PageTitle>>(),
        )
        .await;
        assert!(res);
    }

    #[tokio::test]
    async fn test_post_validation_negative() {
        let random_pages = web::get_random_wikipedia_pages(2, "pwn").await.unwrap();

        let res = post_validation(
            "tests/data/small/not_working_test_database.sqlite",
            "20240901",
            "pwn",
            &random_pages
                .into_iter()
                .map(|p| PageTitle(p.title))
                .collect::<Vec<PageTitle>>(),
        )
        .await;
        assert!(!res);
    }

    // #[tokio::test]
    // async fn test_pre_validation() {
    //     let res = pre_validation(
    //         "tests/data/dewiki-20240901-pagelinks-small.sql",
    //         "tests/data/dewiki-20240901-linktarget.sql").await;
    // }
}
