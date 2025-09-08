use tempfile::tempdir;

use wiki_stats::process::{process_threaded, process_wikis_seq};
use wiki_stats::sqlite::join_db_wiki_path;
use wiki_stats::validate::post_validation;

#[tokio::test]
async fn test_processing_seq() {
    // seems to be the smallest (34KB) wikipedia that is not closed
    let wiki_name = "pwnwiki";

    let tmp_dir = tempdir().expect("Failed creating tempdir");
    let base_dir = tmp_dir.path();

    let dump_date = process_wikis_seq(&[wiki_name], &base_dir, None, false).await;

    let db_path = base_dir.join(dump_date).join("sqlite");
    let valid = post_validation(join_db_wiki_path(db_path, wiki_name), "pwn", 1).await;
    assert!(valid);
}

#[tokio::test]
async fn test_processing_threaded() {
    // seems to be the smallest (34KB) wikipedia that is not closed
    let wiki_name = "pwnwiki";

    let tmp_dir = tempdir().expect("Failed creating tempdir");
    let base_dir = tmp_dir.path();

    let dump_date = process_threaded(&[wiki_name], &base_dir, None, false).await;

    let db_path = base_dir.join(dump_date).join("sqlite");
    let valid = post_validation(join_db_wiki_path(db_path, wiki_name), "pwn", 1).await;
    assert!(valid);
}