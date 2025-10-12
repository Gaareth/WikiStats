use std::env;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use chrono::{DateTime, Datelike, FixedOffset, SecondsFormat, TimeZone, Utc};
use indicatif::ProgressBar;
use log::{debug, info, trace};
use num_format::Locale::{el, ta};
use regex::Regex;
use reqwest::{Request, Response};
use schemars::JsonSchema;
use scraper::{Html, Selector};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::process::split_workload;
use crate::utils::ProgressBarBuilder;
use crate::web::WikipediaApiError::{MissingAttribute, MissingContinue};

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct LinkHere {
    pub pageid: u32,
    pub ns: u32,
    pub title: String,
    pub redirect: bool,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub ns: u32,
    pub title: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct RandomPage {
    pub id: u32,
    pub ns: u32,
    pub title: String,
}

#[derive(Error, Debug)]
pub enum WikipediaApiError {
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON Deserialization error")]
    DeserializeError(#[from] serde_json::Error),

    #[error("JSON malformed")]
    MissingAttribute,

    #[error("Missing continue field")]
    MissingContinue,

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

pub async fn get_random_wikipedia_pages(
    limit: u16,
    wiki_prefix: impl AsRef<str>,
) -> Result<Vec<RandomPage>, WikipediaApiError> {
    assert!(
        limit <= 500,
        "Can only return maximal 500 random pages in one call"
    ); // todo: ? error ? anyhow ?
    let wiki_prefix = wiki_prefix.as_ref();

    let url = format!(
        "https://{wiki_prefix}.wikipedia.org/w/api.php?action=query&list=random&rnnamespace=0&rnlimit={}&format=json",
        limit
    );

    let response = get_wikipedia_async(&url).await?;

    let json: Value = response.json().await?;
    let random_pages = json
        .get("query")
        .and_then(|q| q.get("random"))
        .ok_or(MissingAttribute)?;

    Ok(serde_json::from_value(random_pages.clone())?)
}

pub async fn query_wikipedia_api<T: DeserializeOwned>(
    wiki_prefix: impl AsRef<str>,
    prop: &str,
    extra_params_str: &str,
    continue_key: Option<&str>,
) -> Result<Vec<T>, WikipediaApiError> {
    let wiki_prefix = wiki_prefix.as_ref();

    let mut continue_param: Option<String> = None;
    let mut results: Vec<T> = Vec::new();

    let client = reqwest::Client::new();

    let prop = urlencoding::encode(prop).into_owned();

    loop {
        let url = format!(
            "https://{wiki_prefix}.wikipedia.org/w/api.php?action=query&format=json\
            &prop={prop}&list=&continue=\
            &formatversion=2{}{}",
            extra_params_str,
            continue_param.clone().unwrap_or_default()
        );

        info!("Requesting url: {}", &url);
        // dbg!(&url);

        let resp = client
            .get(&url)
            .header("User-Agent", wikipedia_user_agent())
            .send()
            .await?;
        let content: Value = resp.json().await?;

        let v = content
            .get("query")
            .and_then(|v| v.get("pages").and_then(|pages| pages[0].get(&prop)));

        if let Some(v) = v {
            let linkshere: Vec<T> = serde_json::from_value(v.clone())?;

            results.extend(linkshere);

            if let Some(cont) = content.get("continue") {
                if let Some(continue_key) = continue_key {
                    let next_pageid: String =
                        String::deserialize(cont.get(continue_key).ok_or(MissingContinue)?)?;
                    continue_param = Some(format!("&{continue_key}={next_pageid}"));
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            return Ok(vec![]);
        }
    }

    Ok(results)
}

pub async fn get_latest_revision_id_before_date(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
    date: &DateTime<Utc>,
) -> Result<Option<u64>, WikipediaApiError> {
    let dt_string = date.to_rfc3339_opts(SecondsFormat::Secs, true); // ISO 8601 recommended: https://www.mediawiki.org/w/api.php?action=help&modules=main#main/datatype/timestamp

    #[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
    pub struct Revision {
        pub revid: u64,
        pub parentid: u64,
        pub timestamp: String, // ISO 8601
    }

    let results = query_wikipedia_api::<Revision>(
        wiki_prefix,
        "revisions",
        &format!("&titles={page_name}&rvslots=&rvlimit=1&rvstart={dt_string}&rvdir=older"),
        None,
    )
    .await?;

    for result in &results {
        if &DateTime::parse_from_rfc3339(&result.timestamp).unwrap() >= date {
            panic!("Revisions should not be after specified date, {:?}", result);
        }
    }

    Ok(results.first().map(|r| r.revid))
}

pub async fn get_creation_date(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
) -> Result<Option<DateTime<FixedOffset>>, WikipediaApiError> {
    #[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
    pub struct Revision {
        pub timestamp: String, // ISO 8601
    }

    let results: Vec<Revision> = query_wikipedia_api::<Revision>(
        wiki_prefix,
        "revisions",
        &format!("&titles={page_name}&rvlimit=1&rvdir=newer&rvprop=timestamp|user|comment"),
        None,
    )
    .await?;

    return Ok(results
        .first()
        .map(|r| DateTime::parse_from_rfc3339(&r.timestamp).unwrap()));
}

pub async fn get_added_diff_to_current(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
    rev_id: u64,
) -> Result<Vec<String>, WikipediaApiError> {
    get_diff_to_current(page_name, wiki_prefix, rev_id, true).await
}

pub async fn get_deleted_diff_to_current(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
    rev_id: u64,
) -> Result<Vec<String>, WikipediaApiError> {
    get_diff_to_current(page_name, wiki_prefix, rev_id, false).await
}

pub async fn get_diff_to_current(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
    rev_id: u64,
    added: bool,
) -> Result<Vec<String>, WikipediaApiError> {
    let wiki_prefix = wiki_prefix.as_ref();

    let url = format!(
        "https://{wiki_prefix}.wikipedia.org/w/api.php?action=compare\
        &fromrev={rev_id}\
        &totitle={page_name}\
        &difftype=inline\
        &format=json&formatversion=2"
    );
    info!("Requesting url: {}", &url);

    let res = get_wikipedia_async(&url).await?;
    let json = res.json::<Value>().await?;

    let body = json
        .get("compare")
        .and_then(|c| c.get("body"))
        .and_then(|d| d.as_str());

    let selector = Selector::parse(if added { "ins" } else { "del" }).unwrap();
    if let Some(body) = body {
        let document = Html::parse_document(body);
        let added = document
            .select(&selector)
            .filter_map(|node| node.text().next().map(|text| text.to_string()))
            .collect();
        return Ok(added);
    }

    Err(anyhow!("Failed to get diff").into())
}

pub async fn get_incoming_links(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
) -> Result<Vec<LinkHere>, WikipediaApiError> {
    query_wikipedia_api(
        wiki_prefix,
        "linkshere",
        &format!("&titles={page_name}&lhlimit=max&lhnamespace=0"),
        Some("lhcontinue"),
    )
    .await
}

pub async fn get_incoming_links_by_id(
    page_id: u32,
    wiki_prefix: impl AsRef<str>,
) -> Result<Vec<LinkHere>, WikipediaApiError> {
    query_wikipedia_api(
        wiki_prefix,
        "linkshere",
        &format!("&pageids={page_id}&lhlimit=max&lhnamespace=0"),
        Some("lhcontinue"),
    )
    .await
}

pub async fn get_outgoing_links(
    page_name: &str,
    wiki_prefix: impl AsRef<str>,
) -> Result<Vec<Link>, WikipediaApiError> {
    query_wikipedia_api(
        wiki_prefix,
        "links",
        &format!("&titles={page_name}&pllimit=max&plnamespace=0"),
        Some("plcontinue"),
    )
    .await
}

pub fn get_most_popular_pages(wiki_name: &str) -> Vec<(String, u32)> {
    let now = chrono::Utc::now();
    let last_month = chrono::Utc
        .with_ymd_and_hms(now.year(), now.month() - 1, 1, 0, 0, 0)
        .unwrap();
    let project = format!("{}.wikipedia", &wiki_name[0..=1]);
    let url = format!(
        "https://wikimedia.org/api/rest_v1/metrics/pageviews/top/{project}/all-access/{}/{}/all-days",
        last_month.year(),
        last_month.format("%m")
    );
    dbg!(&url);

    let resp = get_wikipedia_blocking(&url).unwrap();
    let json: Value = resp.json().unwrap();
    // dbg!(&json);
    #[derive(Deserialize, Debug)]
    struct Article {
        article: String,
        views: u32,
        rank: u32,
    }
    let a = &json["items"][0]["articles"];
    let articles: Vec<Article> = serde_json::from_value(a.clone()).unwrap();
    articles.into_iter().map(|a| (a.article, a.views)).collect()
}

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    pub pageid: u64,
    pub ns: u32,
    pub title: String,
    pub contentmodel: String,
    pub pagelanguage: String,
    pub pagelanguagehtmlcode: String,
    pub pagelanguagedir: String,
    pub touched: String,
    pub lastrevid: u64,
    pub length: u32,
}

pub async fn get_page_info_by_title(
    title: impl AsRef<str>,
    wiki_prefix: impl AsRef<str>,
) -> Result<Option<PageInfo>, WikipediaApiError> {
    get_page_info(
        &format!("&titles={}", urlencoding::encode(title.as_ref())),
        wiki_prefix,
    )
    .await
}

pub async fn get_page_info_by_id(
    pageid: u64,
    wiki_prefix: impl AsRef<str>,
) -> Result<Option<PageInfo>, WikipediaApiError> {
    get_page_info(&format!("&pageids={pageid}"), wiki_prefix).await
}

fn wikipedia_user_agent() -> String {
    env::var("WIKIPEDIA_REST_API_USER_AGENT")
        .expect("PROVIDE 'WIKIPEDIA_REST_API_USER_AGENT' as env var")
}

pub async fn get_wikipedia_async(url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get(url.as_ref())
        .header("User-Agent", wikipedia_user_agent())
        .send()
        .await
}

pub fn get_wikipedia_blocking(
    url: impl AsRef<str>,
) -> reqwest::Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    client
        .get(url.as_ref())
        .header("User-Agent", wikipedia_user_agent())
        .send()
}

/// Returns Ok(None) if entry is missing
pub async fn get_page_info(
    extra_params: &str,
    wiki_prefix: impl AsRef<str>,
) -> Result<Option<PageInfo>, WikipediaApiError> {
    let url = format!(
        "https://{}.wikipedia.org/w/api.php?action=query{extra_params}&prop=info&format=json&formatversion=2",
        wiki_prefix.as_ref(),
    );

    // Fetch the page ID for the given title
    let resp = get_wikipedia_async(&url).await?;
    let json: Value = resp.json().await?;

    let value = json
        .get("query")
        .and_then(|v| v.get("pages"))
        .and_then(|v| v.get(0));

    if value.and_then(|v| v.get("missing")).is_some() {
        return Ok(None);
    }

    Ok(value.map(PageInfo::deserialize).transpose()?)
}

fn parse_size_to_bytes(input: &str) -> Option<u64> {
    let input = input.trim();
    let mut parts = input.split_whitespace();

    let number_str = parts.next()?;
    let unit = parts.next()?;

    // Parse the number
    let number: f64 = f64::from_str(number_str).ok()?;

    let bytes = match unit {
        "B" | "Bytes" | "b" | "bytes" => number,
        "KB" => number * 1000.0,
        "MB" => number * 1000.0 * 1000.0,
        "GB" => number * 1000.0 * 1000.0 * 1000.0,
        "TB" => number * 1000.0 * 1000.0 * 1000.0 * 1000.0,
        _ => return None, // Invalid unit
    };

    Some(bytes as u64)
}

// parses file size of https://dumps.wikimedia.org/${wikiname}/${dump_date}/
async fn parse_file_sizes(body: &str) -> Vec<(String, u64)> {
    let mut file_sizes = Vec::new();

    let document = Html::parse_document(body);
    let selector = Selector::parse("body ul li.done").unwrap();
    let selector_filename = Selector::parse("li.file a").unwrap();
    let selector_filesize = Selector::parse("li.file").unwrap();

    for dump in document.select(&selector) {
        if let Some(filesize) = dump.select(&selector_filesize).next() {
            let filesize = &filesize.text().nth(1).unwrap().trim();
            let filename: String = dump.select(&selector_filename).next().unwrap().inner_html();

            file_sizes.push((filename, parse_size_to_bytes(filesize).unwrap()));
        }
    }

    file_sizes
}

//  (
//         "ngwikimedia",
//         2485,
//     ),
//  ("punjabiwikimedia", 1818)
// ("pihwiki", 41000),

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebWikiSize {
    pub name: String,
    pub total_size: Option<u64>,
    pub selected_tables_size: Option<u64>,
}

/// Returns wikis sorted by size (ASCENDING)
pub async fn find_smallest_wikis(
    dump_date: Option<String>,
    tables: &[impl AsRef<str>],
) -> Result<Vec<WebWikiSize>, reqwest::Error> {
    let tables: Vec<String> = tables
        .iter()
        .map(|item| item.as_ref().to_string())
        .collect();

    let base_path = "https://dumps.wikimedia.org";
    let all_dumps_url = format!("{base_path}/backup-index.html");
    let body = reqwest::get(all_dumps_url).await?.text().await?;

    let document = Html::parse_document(&body);

    // all links
    let selector = Selector::parse("a").unwrap();

    let wiki_sizes: Arc<Mutex<Vec<WebWikiSize>>> = Arc::new(Mutex::new(Vec::new()));

    // wikilinks include their dumpdate
    let dump_date_regex = Regex::new(r"\d{8}").unwrap();

    let all_links: Vec<String> = document
        .select(&selector)
        .filter_map(|e| {
            e.value().attr("href").and_then(|link| {
                if dump_date_regex.is_match(link) {
                    let mut link = link.to_string();
                    // link is by default the latest. If dump_date is given, replace it with it
                    if let Some(ref dump_date) = dump_date {
                        link = dump_date_regex
                            .replace(&link, &dump_date.clone())
                            .to_string();
                    }
                    Some(link.to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    // let all_links: Vec<String> = all_links.into_iter().take(20).collect();

    debug!("{} links found", all_links.len());
    let num_threads = 1;
    let worksloads = split_workload(&all_links, num_threads).await;
    debug!(
        "{} Threads with Workload per thread: {:?}",
        num_threads,
        worksloads.iter().map(|w| w.len()).collect::<Vec<usize>>()
    );

    let bar = Arc::new(Mutex::new(
        ProgressBarBuilder::new()
            .with_length(all_links.len() as u64)
            .with_name("Fetching wiki sizes")
            .build(),
    ));

    let mut tasks = vec![];

    for (tid, links) in worksloads.into_iter().enumerate() {
        let wiki_sizes = wiki_sizes.clone();
        let tables = tables.clone();
        let bar = bar.clone();

        tasks.push(tokio::spawn(async move {
            for link in links {
                trace!("[{tid}]: {:?}", &link);

                let wiki_size = calc_wiki_size(base_path, &tables, link).await?;

                wiki_sizes.lock().unwrap().push(wiki_size);
                bar.lock().unwrap().inc(1);
            }
            Ok(())
        }));
    }

    for task in tasks {
        task.await.unwrap()?;
    }
    bar.lock().unwrap().finish_and_clear();

    let mut wiki_sizes = wiki_sizes.lock().unwrap();
    wiki_sizes.sort_by(|a, b| a.total_size.cmp(&b.total_size)); // smaller are first
    Ok(wiki_sizes.clone())
}

async fn calc_wiki_size(
    base_path: &'static str,
    tables: &[impl AsRef<str>],
    link: String,
) -> Result<WebWikiSize, reqwest::Error> {
    let re: Regex = Regex::new(r"\d{8}-(.+?)\.sql").unwrap();
    let tables = tables
        .into_iter()
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>();

    let wiki_name = link.split("/").next().unwrap();
    let resp = get_wikipedia_async(&format!("{base_path}/{link}")).await?;
    // let resp = resp.error_for_status()?;
    if !resp.status().is_success() {
        return Ok(WebWikiSize {
            name: wiki_name.to_string(),
            total_size: None,
            selected_tables_size: None,
        });
    }

    let body = resp.text().await?;

    let file_sizes = parse_file_sizes(&body).await;
    let mut wiki_size = WebWikiSize {
        name: wiki_name.to_string(),
        total_size: Some(0),
        selected_tables_size: Some(0),
    };
    for (filename, filesize_bytes) in file_sizes {
        if let Some(table_name) = re.captures(&filename).and_then(|captures| captures.get(1)) {
            wiki_size.total_size = Some(wiki_size.total_size.unwrap_or(0) + filesize_bytes);
            if tables.contains(&table_name.as_str()) {
                wiki_size.selected_tables_size =
                    Some(wiki_size.selected_tables_size.unwrap_or(0) + filesize_bytes);
            }
        }
    }
    Ok(wiki_size)
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, TimeZone, Utc, format};

    use crate::{
        download::ALL_DB_TABLES,
        web::{
            calc_wiki_size, find_smallest_wikis, get_added_diff_to_current,
            get_deleted_diff_to_current, get_incoming_links, get_latest_revision_id_before_date,
            get_outgoing_links, get_page_info_by_id, get_page_info_by_title, parse_size_to_bytes,
        },
    };

    fn setup() {
        dotenv::dotenv().ok();
    }

    #[tokio::test]
    async fn test_calc_wiki_size() {
        setup();

        let now = chrono::Utc::now().with_day(1).unwrap();
        let dump_date = format!("{:04}{:02}01", now.year(), now.month() - 1);

        let ws = calc_wiki_size(
            "https://dumps.wikimedia.org",
            &ALL_DB_TABLES,
            format!("enwiki/{dump_date}/"),
        )
        .await;
        assert!(ws.unwrap().total_size.unwrap() > 0);

        let ws = calc_wiki_size(
            "https://dumps.wikimedia.org",
            &ALL_DB_TABLES,
            format!("hywiki/{dump_date}/"),
        )
        .await;
        assert!(ws.unwrap().total_size.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_incoming_links() {
        let res = get_incoming_links("Main Page", "en").await;
        assert!(!res.unwrap().is_empty());

        let res = get_incoming_links("Angela Merkel", "de").await;
        assert!(!res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_outgoing_links() {
        let res = get_outgoing_links("Albert Einstein", "en").await;
        assert!(!res.unwrap().is_empty());

        let res = get_outgoing_links("Angela_Merkel", "en").await;
        assert!(!res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_page_info() {
        let res = get_page_info_by_title("Angela Merkel", "de").await;
        assert_eq!(res.unwrap().unwrap().pageid, 145);

        let res = get_page_info_by_id(145, "de").await;
        assert_eq!(res.unwrap().unwrap().title, "Angela Merkel");

        assert!(
            get_page_info_by_title("045a3b28f5a0f08adc295a14", "es")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test()]
    #[ignore]
    async fn test_smallest_wiki() {
        let tables = ["page", "pagelinks", "linktarget"];
        let res = find_smallest_wikis(None, &tables).await.unwrap();
        dbg!(&res);
        assert!(!res.is_empty());
    }

    #[test]
    fn test_parse_size_to_bytes() {
        let test_cases = vec![
            ("26.7 GB", Some(26_700_000_000)),
            ("512 MB", Some(512_000_000)),
            ("1024 KB", Some(1_024_000)),
            ("2048 Bytes", Some(2048)),
            ("1.5 TB", Some(1_500_000_000_000)),
            ("1000 b", Some(1000)),
            ("5 KB", Some(5000)),
            ("0.5 MB", Some(500_000)),
            ("1.234 GB", Some(1_234_000_000)),
            ("Invalid Unit", None),
            ("1234", None),
            ("1000", None),
        ];

        for (input, expected) in test_cases {
            let result = parse_size_to_bytes(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[tokio::test]
    async fn test_revision_id() {
        let res = get_latest_revision_id_before_date(
            "Angela Merkel",
            "en",
            &Utc.with_ymd_and_hms(2024, 9, 1, 0, 0, 0).unwrap(),
        )
        .await;
        assert_eq!(res.unwrap().unwrap(), 1242954983);
    }

    #[tokio::test]
    async fn test_diff() {
        let res = get_added_diff_to_current(
            "Road to the Rolex Shanghai Masters Shanghai Challenger 2024",
            "de",
            247846542,
        )
        .await;
        assert!(!res.unwrap().is_empty());

        let res = get_deleted_diff_to_current("José_Jerí", "en", 1316431833).await;
        assert!(!res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_deleted_diff() {
        let res = get_deleted_diff_to_current("Nati nel 1900", "it", 145601453)
            .await
            .unwrap();
        assert!(!res.is_empty());
        // dbg!(&res);

        assert!(res.iter().any(|d| d.contains(&"Hans Rehmann".to_string())))
    }
}
