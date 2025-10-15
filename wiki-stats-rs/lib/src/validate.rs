use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

use chrono::{DateTime, Utc};
use colored::Colorize as _;
use log::{debug, error, info, warn};
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use parse_mediawiki_sql::schemas::PageLink;
use parse_mediawiki_sql::utils::{Mmap, memory_map};
use rusqlite::Connection;

use crate::download::ALL_DB_TABLES;
use crate::sqlite::load::{
    load_links_map, load_linktarget_map, load_map, load_sql_full, load_sql_part_map,
};
use crate::sqlite::page_links::{get_incoming_links_of_id, get_links_of_id};
use crate::sqlite::title_id_conv;
use crate::sqlite::to_sqlite::INFO_TABLE;
use crate::web::{
    WikipediaApiError, get_added_diff_to_current, get_creation_date, get_deleted_diff_to_current,
    get_dump_finish_date, get_latest_revision_id_before_date, get_links_on_webpage,
};
use crate::{format_as_dumpdate, parse_dump_date, web};

use anyhow::Result;
use futures::FutureExt;
use futures::future::{BoxFuture, join_all};
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
            let dump_date_str = format_as_dumpdate(dump_date);
            let wiki_name = format!("{}wiki", wiki_prefix); // this really does not always hold
            // the dumps are typically finished 1 day or so after the dumpdate actually says
            // this help getting the likely actual revision the dump is based on? more mostly equal with
            let latest_all_tables_done_date =
                get_dump_finish_date(&wiki_name, &dump_date_str, &ALL_DB_TABLES)
                    .await
                    .unwrap();
            let rev_id_opt =
                get_latest_revision_id_before_date(page, wiki_prefix, &latest_all_tables_done_date)
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
) -> (bool, Vec<(PageTitle, PageTitle)>) {
    let mut success = true;

    debug!("expected: {:?}", expected_results);
    debug!("actual: {:?}", db_results);

    let mut revid_map: HashMap<String, u64> = HashMap::new();
    let mut links_on_old_page_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut diffs_added_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut diffs_deleted_map: HashMap<String, Vec<String>> = HashMap::new();

    let missing_diffs: Vec<&PageTitle> = expected_results.difference(db_results).collect();
    let outdated_diffs: Vec<&PageTitle> = db_results.difference(expected_results).collect();

    let mut actually_diffs: Vec<(PageTitle, PageTitle)> = vec![];
    let sum_diffs = missing_diffs.len() + outdated_diffs.len();
    if sum_diffs > 0 {
        info!(
            "Found {} differences that are now going to be validated",
            sum_diffs
        );
    }
    // Links missing in DB
    for diff in missing_diffs {
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
            if link_type == "incoming" {
                actually_diffs.push((diff.clone(), PageTitle(page_title.to_string())));
            } else {
                actually_diffs.push((PageTitle(page_title.to_string()), diff.clone()));
            }
        }
    }

    // Links that exist in DB but not online
    for diff in outdated_diffs {
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
            if link_type == "incoming" {
                actually_diffs.push((diff.clone(), PageTitle(page_title.to_string())));
            } else {
                actually_diffs.push((PageTitle(page_title.to_string()), diff.clone()));
            }
        }
    }

    (success, actually_diffs)
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

    // check if the old wikipedia page even contains this link. If not we are fine.
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

// links that are in the db but not returned by the api call
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

    // if the old wikipedia pages contains it, it is probably fine
    if let Some(rev_id) = rev_id_opt {
        let links_on_old_page = get_or_cache_links(from, wiki_prefix, rev_id, links_cache).await;
        if link_exists(&links_on_old_page, to) {
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
) -> (bool, Vec<(PageTitle, PageTitle)>) {
    let t1 = Instant::now();

    let db_file = db_file.as_ref();
    if !fs::exists(&db_file).unwrap() {
        eprintln!("DB File {db_file:?} does not exist");
        return (false, vec![]);
    }

    let conn = Connection::open(db_file).unwrap();
    if !check_is_done(&conn).unwrap_or(false) {
        eprintln!("DB File {db_file:?} is not done");
        return (false, vec![]);
    }

    let wiki_prefix = wiki_prefix.as_ref();
    let dump_date = parse_dump_date(dump_date.as_ref()).expect("Failed parsing dumpdate");

    let mut success = true;

    // TODO: maybe just use the sql file???
    // removes links that don't exist and converts from pageid -> pagetitle
    let filter_broken_pids = |pids: Vec<PageId>| async {
        let mut filtered = HashSet::new();
        for pid in pids {
            // web::get_page_info_by_id(pid.0 as u64, &wiki_prefix).await.unwrap()
            // title_id_conv::page_id_to_title(&pid, &conn)
            if let Some(p) = title_id_conv::page_id_to_title(&pid, &conn) {
                filtered.insert(p.0.replace("_", " ").into());
                // filtered.insert(PageTitle(p.title));
            } else {
                warn!("DB contains link to {pid:?} but this pid seems not to exist");
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

    let mut diffs = vec![];

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
        let expected_incoming = web::get_incoming_links(&page_title, &wiki_prefix)
            .await
            .unwrap()
            .into_iter()
            .map(|link| PageTitle(link.title))
            .collect();
        // filter to only include links that exist
        // let expected_incoming: HashSet<_> = filter_broken_pids(all_links).await;
        // let expected_incoming = all_links.

        // database results
        let db_incoming: HashSet<PageTitle> =
            filter_broken_pids(get_incoming_links_of_id(&conn, &PageId(page_id))).await;
        let (valid, mut d) = compare_links(
            &page_title,
            &wiki_prefix,
            &dump_date,
            &expected_incoming,
            &db_incoming,
            "incoming",
        )
        .await;
        success &= valid;
        diffs.append(&mut d);

        info!("Checking outgoing links..");
        // ### [outgoing] links
        // online results
        let expected_outgoing = web::get_outgoing_links(&page_title, &wiki_prefix)
            .await
            .unwrap()
            .into_iter()
            .map(|link| PageTitle(link.title))
            .collect();
        // filter to only include links that exist

        // database results
        let db_outgoing: HashSet<PageTitle> =
            filter_broken_pids(get_links_of_id(&conn, &PageId(page_id))).await;
        let (valid, mut d) = compare_links(
            &page_title,
            &wiki_prefix,
            &dump_date,
            &expected_outgoing,
            &db_outgoing,
            "outgoing",
        )
        .await;
        success &= valid;
        diffs.append(&mut d);
    }

    conn.execute(INFO_TABLE, ()).unwrap();

    let r =conn.execute(
            "UPDATE Info SET is_validated = ?, num_pages_validated = ?, validation_time_s = ? WHERE id = 0",
            (success, pages_to_test.len(),t1.elapsed().as_secs_f64()),
        );
    if let Err(e) = r {
        log::error!("{}", format!("Error setting is_validated: {e}"));
    }

    (success, diffs)
}

/// Currently only checks outgoing links
/// # Args
/// - verify_recency: if true, checks whether each detected difference is simply caused by the web version being more recent than the SQL dumps.
///   This involves making API calls to confirm data freshness, and e.g., checking if a link was added/deleted after the dumpdate
/// # Returns
/// - Returns: (valid, outgoing_diffs)
/// - outgoing_diffs: links that are missing or outdated
pub async fn pre_validation(
    pl_sql_file_path: impl AsRef<Path>,
    lt_sql_file_path: impl AsRef<Path>,
    page_titles_to_check: &[PageTitle],
    dump_date: impl AsRef<str>,
    verify_recency: bool,
) -> (bool, Vec<(PageTitle, PageTitle)>) {
    let pl_sql_file_path = pl_sql_file_path.as_ref();
    let filename = &pl_sql_file_path.file_name().unwrap().to_str().unwrap();
    let wikiname = filename.split("-").next().unwrap();
    let wikiprefix = &wikiname[..2];
    let dump_date = parse_dump_date(dump_date.as_ref()).expect("Failed parsing dumpdate");

    let page_ids_to_check: Vec<PageId> = join_all(
        page_titles_to_check
            .into_iter()
            .map(|pt| web::page_title_to_id(pt.clone(), wikiprefix)),
    )
    .await;

    let pl_mmap: Mmap = unsafe { memory_map(pl_sql_file_path).unwrap() };
    let pl_map = load_links_map::<_, _, PageLink, _, _>(
        &pl_mmap,
        |pl| (pl.from, pl.target),
        |pl| pl.from_namespace.0 != 0 || !page_ids_to_check.contains(&pl.from),
    );

    let lt_mmap: Mmap = unsafe { memory_map(lt_sql_file_path).unwrap() };
    let lt_map = load_linktarget_map(lt_mmap);

    let mut success = true;

    let mut outgoing_diffs = vec![];

    for (page_id, link_targets) in pl_map.iter() {
        let page_title = web::page_id_to_title(*page_id, &wikiprefix).await;

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

        if verify_recency {
            let (valid, mut diffs) = compare_links(
                &page_title.0,
                &wikiprefix,
                &dump_date,
                &expected_results,
                &sqlfile_results,
                "outgoing",
            )
            .await;
            success &= valid;
            outgoing_diffs.append(&mut diffs);
        } else {
            let mut diffs: Vec<(PageTitle, PageTitle)> = expected_results
                .symmetric_difference(&sqlfile_results)
                .cloned()
                .map(|to| (page_title.clone(), to))
                .collect();
            success = false;
            outgoing_diffs.append(&mut diffs);
        }
    }

    (success, outgoing_diffs)
}

pub async fn validate_post_validation(
    dump_date: &str,
    wiki: &str,
    dumpdate_path: &std::path::PathBuf,
    post_diffs: Vec<(PageTitle, PageTitle)>,
) -> bool {
    let downloads_path = dumpdate_path.join("downloads");

    let pl_sql_file_path = downloads_path.join(format!("{}-{}-pagelinks.sql", wiki, &dump_date));
    let lt_sql_file_path = downloads_path.join(format!("{}-{}-linktarget.sql", wiki, &dump_date));

    if !pl_sql_file_path.exists() || !lt_sql_file_path.exists() {
        error!("Missing download artifacts to check if differences also exist in the raw dump");
        return false;
    }

    let pages_to_check: Vec<PageTitle> = post_diffs.iter().map(|(from, _)| from.clone()).collect();

    let (valid, pre_diffs) = pre_validation(
        pl_sql_file_path,
        lt_sql_file_path,
        &pages_to_check,
        dump_date,
        false, // skip recency check as done already by post validation
    )
    .await;

    let set_post: HashSet<_> = post_diffs
        .iter()
        .map(|(from, to)| {
            (
                PageTitle(from.0.replace(" ", "_")),
                PageTitle(to.0.replace(" ", "_")),
            )
        })
        .collect();
    let set_pre: HashSet<_> = pre_diffs
        .into_iter()
        .map(|(from, to)| {
            (
                PageTitle(from.0.replace(" ", "_")),
                PageTitle(to.0.replace(" ", "_")),
            )
        })
        .collect();

    if set_post.is_subset(&set_pre) {
        info!(
            "All wrong links inside the database are also in the dumps. This is treated as a success. Yay!"
        );
        return true;
    } else {
        error!("There are some wrong links inside the database that are not in the dump");
        let diffs: HashSet<_> = set_post.difference(&set_pre).collect();

        dbg!(diffs);
        // dbg!(&set_pre);
        return false;
    }
}

#[cfg(test)]
mod tests {
    use parse_mediawiki_sql::field_types::PageTitle;

    use crate::validate::post_validation;
    use crate::web;

    #[tokio::test]
    async fn test_post_validation() {
        let random_pages = web::get_random_wikipedia_pages(2, "pwn").await.unwrap();

        let (valid, _) = post_validation(
            "tests/data/small/test_database.sqlite",
            "20240901",
            "pwn",
            &random_pages
                .into_iter()
                .map(|p| PageTitle(p.title))
                .collect::<Vec<PageTitle>>(),
        )
        .await;
        assert!(valid);
    }

    #[tokio::test]
    async fn test_post_validation_negative() {
        let random_pages = web::get_random_wikipedia_pages(2, "pwn").await.unwrap();

        let (valid, _) = post_validation(
            "tests/data/small/not_working_test_database.sqlite",
            "20240901",
            "pwn",
            &random_pages
                .into_iter()
                .map(|p| PageTitle(p.title))
                .collect::<Vec<PageTitle>>(),
        )
        .await;
        assert!(!valid);
    }

    // #[tokio::test]
    // async fn test_pre_validation() {
    //     let res = pre_validation(
    //         "tests/data/dewiki-20240901-pagelinks-small.sql",
    //         "tests/data/dewiki-20240901-linktarget.sql").await;
    // }
}
