use std::{fs, path::Path};

use anyhow::anyhow;
use reqwest::StatusCode;
use rusqlite::Connection;
use wiki_stats::{
    sqlite::join_db_wiki_path,
    validate::{check_is_done, check_is_validated},
};

/// Checks if wikimedia has sql dumps for the wiki
/// Returns `Ok(())` if valid, or an `Err` with an error message if invalid.
pub async fn validate_wiki_name(wiki: &str) -> anyhow::Result<()> {
    if wiki.is_empty() {
        return Err(anyhow!("Wiki name cannot be empty."));
    }

    let resp = reqwest::get(format!("https://dumps.wikimedia.org/{wiki}/")).await?;

    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else if status == StatusCode::NOT_FOUND {
        Err(anyhow!(
            "There are no dumps for '{wiki}' on https://dumps.wikimedia.org/. Check https://dumps.wikimedia.org/backup-index.html for available wikis"
        ))
    } else {
        Err(anyhow!(
            "Error checking '{wiki}' on https://dumps.wikimedia.org/{wiki}/. StatusCode: {status}"
        ))
    }
}

pub async fn validate_wiki_names(wikis: &[impl AsRef<str>]) -> Result<(), String> {
    if wikis.is_empty() {
        return Err("Please provide at least one name".to_string());
    }

    for wiki in wikis {
        if let Err(e) = validate_wiki_name(wiki.as_ref()).await {
            return Err(e.to_string());
        }
    }
    Ok(())
}

/// Validate the sqlite file
/// Returns `Ok(())` if valid, or an `Err` with an error message if invalid.
pub async fn validate_sqlite_file(db_path: impl AsRef<Path>, wiki: &str) -> anyhow::Result<()> {
    if wiki.is_empty() {
        return Err(anyhow!("Wiki name cannot be empty."));
    }
    let db_path = db_path.as_ref().to_path_buf();
    let path = join_db_wiki_path(db_path, wiki);
    if !fs::exists(&path).unwrap() {
        return Err(anyhow!("DB File {path:?} does not exist"));
    }

    let conn = Connection::open(&path)?;
    if check_is_done(&conn)? && check_is_validated(&conn)? {
        return Ok(());
    }
    return Err(anyhow!("sqlite file {path:?} is not done"));
}

pub async fn validate_sqlite_files(
    db_path: impl AsRef<Path>,
    wikis: &[impl AsRef<str>],
) -> Result<(), String> {
    if wikis.is_empty() {
        return Err("Please provide at least one name".to_string());
    }

    for wiki in wikis {
        if let Err(e) = validate_sqlite_file(&db_path, wiki.as_ref()).await {
            return Err(e.to_string());
        }
    }
    Ok(())
}
