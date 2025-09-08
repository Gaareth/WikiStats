use std::collections::HashSet;
use std::path::Path;

use chrono::{DateTime, Utc};
use futures::stream::{self, StreamExt};
use itertools::rev;
use log::{debug, info, warn};
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use parse_mediawiki_sql::schemas::Page;
use parse_mediawiki_sql::utils::{memory_map, Mmap};
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::{parse_dump_date, web};
use crate::sqlite::load::{load_linktarget_map, load_sql_part_map};
use crate::sqlite::page_links::{get_incoming_links_of_id, get_links_of_id};
use crate::web::{get_added_diff_to_current, get_latest_revision_id_before_date};

async fn compare_links(
    page_title: &str,
    wiki_prefix: &str,
    dump_date: &DateTime<Utc>,
    expected_results: &HashSet<PageTitle>,
    db_results: &HashSet<PageTitle>,
    link_type: &str,
) -> bool {
    let mut success = true;

    let missing = expected_results.difference(db_results);
    debug!("expected: {:?}", expected_results);
    debug!("actual: {:?}", db_results);

    let rev_id = get_latest_revision_id_before_date(page_title, wiki_prefix, dump_date).await.unwrap();
    let diff_page = if let Some(rev_id) = rev_id {
        get_added_diff_to_current(page_title, wiki_prefix, rev_id).await.unwrap()
    } else {
        vec![]
    };

    for diff in missing {
        let (from, to) = if link_type == "incoming" {
            (diff.0.as_str(), page_title)
        } else {
            (page_title, diff.0.as_str())
        };

        if link_type != "incoming" {
            if diff_page.iter().any(|d| d.contains(&diff.0)) {
                println!("[{}] Link {} -> {} missing from db but seems to be added after {dump_date} db dump", link_type, from, to);
                continue;
            }
        } else {
            let rev_id_of_incoming = get_latest_revision_id_before_date(&from, wiki_prefix, dump_date).await.unwrap();
            let diff_incoming = if let Some(rev_id) = rev_id_of_incoming {
                get_added_diff_to_current(&from, wiki_prefix, rev_id).await.unwrap()
            } else {
                vec![]
            };


            if diff_incoming.iter().any(|d| d.contains(&to)) {
                println!("[{}] Link {} -> {} missing from db but seems to be added after {dump_date} db dump", link_type, from, to);
                continue;
            }
        }


        eprintln!("[{}] Link {} -> {} missing from db", link_type, from, to);
        success = false;
    }

    let outdated = db_results.difference(expected_results);
    for diff in outdated {
        let (from, to) = if link_type == "incoming" {
            (diff.0.as_str(), page_title)
        } else {
            (page_title, diff.0.as_str())
        };


        eprintln!("[{}] Link {} -> {} in db but not online (outdated)?", link_type, from, to);
        success = false;
    }

    success
}


pub async fn post_validation(db_file: impl AsRef<Path>, dump_date: impl AsRef<str>,
                             wiki_prefix: impl AsRef<str>,
                             pages_to_test: &[PageTitle]) -> bool {
    let conn = Connection::open(db_file).unwrap();
    let wiki_prefix = wiki_prefix.as_ref();
    let dump_date = parse_dump_date(dump_date.as_ref()).expect("Failed parsing dumpdate");

    // don't set to high as operation is expensive (calling get_page_info for every returned result)
    let mut success = true;


    // removes links that don't exist and converts from pageid -> pagetitle
    let filter_broken_pids = |pids: Vec<PageId>| async {
        let mut filtered = HashSet::new();
        for pid in pids {
            if let Some(pinfo) = web::get_page_info_by_id(pid.0 as u64, &wiki_prefix).await.unwrap() {
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
            if let Some(pinfo) = web::get_page_info_by_title(&pt.0, &wiki_prefix).await.unwrap() {
                filtered.insert(PageTitle(pinfo.title));
            } else {
                warn!("webpage contains link to {pt:?} but this pagetitle seems not to exist");
            }
        }
        filtered
    };

    let mut pages = vec![];
    for pt in pages_to_test {
        let pid = web::get_page_info_by_title(&pt.0, wiki_prefix).await.unwrap().expect(&format!("No pageid for pagetitle {pt:?}")).pageid;
        pages.push((pt.0.as_str(), pid as u32));
    }

    for (page_title, page_id) in pages {
        info!("Checking page: {:?}", &page_title);
        info!("Checking incoming links..");

        // ### [incoming links]
        // online results
        let all_links = web::get_incoming_links(&page_title, &wiki_prefix).await.unwrap()
            .into_iter().map(|link| PageId(link.pageid)).collect();
        // filter to only include links that exist
        let expected_incoming: HashSet<_> = filter_broken_pids(all_links).await;

        // database results
        let db_incoming: HashSet<PageTitle> = filter_broken_pids(get_incoming_links_of_id(&conn, &PageId(page_id))).await;
        success &= compare_links(&page_title, &wiki_prefix, &dump_date, &expected_incoming, &db_incoming, "incoming").await;

        // info!("Checking outgoing links..");
        // // ### [outgoing] links
        // // online results
        // let all_links = web::get_outgoing_links(&page_title, &wiki_prefix).await.unwrap()
        //     .into_iter().map(|link| PageTitle(link.title)).collect();
        // // filter to only include links that exist
        // let expected_outgoing: HashSet<_> = convert_pid_pt(all_links).await;
        //
        // // database results
        // let db_outgoing: HashSet<PageTitle> = filter_broken_pids(get_links_of_id(&conn, &PageId(page_id))).await;
        // success &= compare_links(&page_title, &wiki_prefix, &dump_date, &expected_outgoing, &db_outgoing, "outgoing");
    }

    success
}

async fn pre_validation(pl_sql_file_path: impl AsRef<Path>, lt_sql_file_path: impl AsRef<Path>) -> bool {
    let pl_mmap: Mmap = unsafe { memory_map(pl_sql_file_path).unwrap() };
    let pl_map = load_sql_part_map(pl_mmap, 1, 1);

    let lt_mmap: Mmap = unsafe { memory_map(lt_sql_file_path).unwrap() };
    let lt_map = load_linktarget_map(lt_mmap);
    dbg!(&lt_map.len());

    println!("Loaded map");

    let mut success = true;

    for (page_id, link_targets) in pl_map.iter() {
        dbg!(&page_id);

        let expected_results: HashSet<PageTitle> = HashSet::from_iter(
            web::get_incoming_links_by_id(page_id.0, "de").await.unwrap().into_iter().map(|link| link.title.into()));

        dbg!(&expected_results);
        dbg!(&link_targets);

        let sqlfile_results = HashSet::from_iter(
            link_targets.iter().filter_map(|lt| lt_map.get(&lt)).cloned()
        );

        dbg!(&sqlfile_results);

        // the pl_map will only return some links, so we can only test if the part of returned links is also in the expected links 
        let diffs = sqlfile_results.difference(&expected_results);
        let mut diffs_count = 0;
        for diff in diffs {
            eprintln!("Diff: Link from {} -> {} not api results", page_id.0, diff.0);
            success = false;
            diffs_count += 1;
        }

        // println!("{} / {} = {}% WRONG", diffs_count, sqlfile_results.len(), (diffs_count / sqlfile_results.len()) * 100);
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

        let res = post_validation("tests/data/small/test_database.sqlite", "20240901",
                                  "pwn", &random_pages.into_iter().map(|p| PageTitle(p.title)).collect()).await;
        assert!(res);
    }

    #[tokio::test]
    async fn test_post_validation_negative() {
        let random_pages = web::get_random_wikipedia_pages(2, "pwn").await.unwrap();

        let res = post_validation("tests/data/small/not_working_test_database.sqlite", "20240901",
                                  "pwn", &random_pages.into_iter().map(|p| PageTitle(p.title)).collect()).await;
        assert!(!res);
    }

    // #[tokio::test]
    // async fn test_pre_validation() {
    //     let res = pre_validation(
    //         "tests/data/dewiki-20240901-pagelinks-small.sql",
    //         "tests/data/dewiki-20240901-linktarget.sql").await;
    // }
}
