use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::{Duration, Instant};
use std::{fs, io};

use anyhow::{Context, anyhow};
use chrono::{Datelike, Days, Months, Utc};
use colored::Colorize;
use futures::StreamExt;
use futures::future::join_all;
use indicatif::MultiProgress;
use log::{debug, error, info, warn};
use scraper::{Html, Selector};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use url::Url;

use crate::utils::{download_bar, spinner_bar};

pub async fn check_dump_complete(wiki: &str, tables: &[impl AsRef<str>], dt: &str) -> bool {
    let mut all_tables_complete = true;
    for table in tables {
        let table = table.as_ref();

        let url = format!("https://dumps.wikimedia.org/{wiki}/{dt}/{wiki}-{dt}-{table}.sql.gz");
        debug!("Checking {url}");
        let resp = reqwest::get(url).await.unwrap();
        let status = resp.status();
        if !status.is_success() {
            error!("[{dt}] Table [{table}] for {wiki} incomplete");
            all_tables_complete = false;
        }
    }

    all_tables_complete
}

pub async fn check_dump_complete_all(
    wiki_names: &[impl AsRef<str>],
    tables: &[impl AsRef<str>],
    dump_date: &str,
) -> bool {
    let mut all_complete = true;
    for wiki_name in wiki_names {
        let wiki_name = wiki_name.as_ref();
        let complete = check_dump_complete(wiki_name, tables, dump_date).await;
        if !complete {
            all_complete = false;
            println!("[{dump_date}] Dump for {wiki_name} is incomplete");
        }
    }
    all_complete
}

// Returns all dump_dates for that all wikis have finished tables
pub async fn get_all_available_dump_dates_for_all_wikis(
    wiki_names: &[impl AsRef<str>],
    tables: &[impl AsRef<str> + Debug],
) -> Vec<String> {
    let mut all_dump_dates: Option<HashSet<String>> = None;

    for wiki in wiki_names {
        let wiki = wiki.as_ref();
        let wiki_dump_dates: HashSet<String> = get_all_available_dump_dates(wiki, tables)
            .await
            .into_iter()
            .collect();

        all_dump_dates = match all_dump_dates {
            Some(ref current) => Some(current.intersection(&wiki_dump_dates).cloned().collect()),
            None => Some(wiki_dump_dates),
        };
    }

    let available_dump_dates: Vec<String> = all_dump_dates
        .map(|s| s.into_iter().collect())
        .unwrap_or_default();

    available_dump_dates
}

pub async fn get_all_available_dump_dates(
    wiki: impl AsRef<str>,
    tables: &[impl AsRef<str> + Debug],
) -> Vec<String> {
    let url = format!("https://dumps.wikimedia.org/{}", wiki.as_ref());

    let resp = reqwest::get(url).await.unwrap();
    let status = resp.status();
    if !status.is_success() {
        return Vec::new();
    }

    let doc = Html::parse_document(&resp.text().await.unwrap());
    let selector = Selector::parse("a").unwrap();

    let mut dump_dates: Vec<String> = Vec::new();
    for element in doc.select(&selector) {
        if let Some(dump_date) = element.value().attr("href") {
            if dump_date
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                // Remove the last character which is a /
                let dump_date = &dump_date[..dump_date.len() - 1];

                if check_dump_complete(wiki.as_ref(), tables, dump_date).await {
                    dump_dates.push(dump_date.to_string());
                }
            }
        }
    }

    dump_dates
}

/// Gets the latest dump_date for that all supplied wiki dumps are complete
/// Returns the subdir of the wiki where all files are located
/// Only complete if all tables are done
/// # Args:
/// - wikis_to_support: the wiki names to check for dump completeness
/// - no_fallback: Don't return earlier date if latest dump is incomplete
///
/// # Return
/// - Some(date_string) where date_string is like 20240820
/// - None if the current month has no complete dump and check_multiple is false
/// - None if there was no complete dump found in the last n months
pub async fn latest_dump_date(
    wikis_to_support: &[impl AsRef<str> + Debug],
    tables: &[impl AsRef<str> + Debug],
    check_multiple: bool,
    check_all_days: bool,
) -> Option<String> {
    let mut date = Utc::now();

    if date.day() >= 20 {
        date = date.with_day(20).unwrap();
    } else {
        date = date.with_day(1).unwrap();
    };

    let months_to_check: u32 = 12;
    let mut checked_months: u32 = 0;

    loop {
        if checked_months >= months_to_check {
            break;
        }

        let dt_s = format!("{}{:02}{:02}", date.year(), date.month(), date.day());
        info!("Checking dumpdate: {dt_s}");

        let all_complete = check_dump_complete_all(wikis_to_support, tables, &dt_s).await;
        if all_complete {
            return Some(dt_s);
        } else {
            warn!("Not all dumps complete for {dt_s}");

            if !check_multiple {
                warn!("No fallback allowed. Wait until latest dump is complete");
                return None;
            }

            warn!("Checking if fallback is ready");
        }

        if check_all_days {
            date = date.checked_sub_days(Days::new(1)).unwrap();
        } else {
            date = match date.day() {
                20 => date.with_day(1).unwrap(),
                1 => {
                    checked_months += 1;
                    let prev_month = date.checked_sub_months(Months::new(1)).unwrap();
                    prev_month.with_day(20).unwrap()
                }
                _ => panic!("Invalid state {}", date.day()),
            }
        }
    }

    warn!(
        "Not dump date found for all wikis [{:?}], and all tables: [{:?}]. Checked {checked_months}",
        wikis_to_support, tables
    );
    None
}

pub static ALL_DB_TABLES: [&str; 3] = ["page", "pagelinks", "linktarget"];

static MIRROR_URLS: [&str; 5] = [
    "https://mirror.accum.se/mirror/wikimedia.org/dumps", // sweden 12MiB/s
    "https://wikimedia.mirror.clarkson.edu",              // new york 10MiB/s
    "https://wikimedia.bringyour.com",                    // california 9MiB/s
    "http://wikipedia.c3sl.ufpr.br",                      // brazil 7MiB/s
    "https://dumps.wikimedia.org",                        // only allows 2 concurrent connections
];

async fn download_file_with_retries<F, Fut, T, E>(download_fn: F, retries: u32) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Debug,
{
    let base_delay: u64 = 2;
    let max_delay: u64 = 60 * 60; // 1h

    let mut attempts = 0;
    loop {
        match download_fn().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                attempts += 1;

                if attempts >= retries {
                    eprintln!("All {retries} attempts failed: {e:?}");
                    return Err(e);
                }

                // Exponential backoff: base delay doubles each time
                let base_delay_secs = base_delay.pow(attempts).min(max_delay);

                let delay = Duration::from_secs(base_delay_secs);

                eprintln!(
                    "Retry {attempts}/{retries} failed: {e:?}. Retrying in {:?}...",
                    delay
                );

                sleep(delay).await;
            }
        }
    }
}

async fn download_file_bar(
    url: String,
    dest_dir: PathBuf,
    multi_bar: MultiProgress,
    md5sums: HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let t1 = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60 * 2 * 60))
        .build()
        .unwrap();

    let response = client.get(&url).send().await.unwrap();
    let response = response.error_for_status()?;

    let dest = {
        let fname = response.url().path_segments().unwrap().last().unwrap();
        dest_dir.join(fname)
    };

    // if !response.status().is_success() {
    //     eprintln!(
    //         "{}",
    //         format!("Error downloading: {}: {}", url, response.status()).red()
    //     );
    //     return;
    // }

    let total_size = response
        .content_length()
        .ok_or(anyhow!("Can't get content length"))?;
    let mut downloaded = 0;
    let filename = dest.file_name().and_then(OsStr::to_str).unwrap();
    let url = Url::parse(&url)?;
    let domain_name = url.host_str().unwrap();
    let bar = multi_bar.add(download_bar(
        total_size,
        &format!("{filename} ({domain_name})"),
    ));
    // bar.set_message(format!("Downloading {}", url));

    let mut stream = response.bytes_stream();

    if Path::exists(&dest) {
        let file = OpenOptions::new()
            .create(false)
            .write(true)
            .open(dest.clone())
            .await
            .with_context(|| format!("Failed opening file {filename}"))?;

        let filesize = file.metadata().await.unwrap().len();

        let is_not_corrupted = verify_download(&md5sums, filename, &dest).await;

        if filesize == total_size && is_not_corrupted {
            multi_bar
                .println(
                    format!("File {filename} already downloaded: skipping")
                        .magenta()
                        .to_string(),
                )
                .unwrap();
            return Ok(());
        }

        if !is_not_corrupted {
            multi_bar
                .println(
                    format!("Download {filename} is corrupt: redownloading")
                        .red()
                        .to_string(),
                )
                .unwrap();
        }
    }

    // truncates file to redownload from the start
    let mut file = File::create(&dest).await.unwrap();

    while let Some(item) = stream.next().await {
        let chunk = item.with_context(|| "Network error during download")?;
        file.write_all(&chunk).await.unwrap();
        downloaded = min(downloaded + (chunk.len() as u64), total_size);
        bar.set_position(downloaded);
    }

    bar.finish_and_clear();

    let is_not_corrupted = verify_download(&md5sums, filename, &dest).await;
    if is_not_corrupted {
        multi_bar
            .println(
                format!("Downloaded file {filename} in {:?}", t1.elapsed())
                    .green()
                    .to_string(),
            )
            .unwrap();
    } else {
        multi_bar
            .println(
                format!("Failed downloading {filename}: corrupt?. Please try again")
                    .red()
                    .to_string(),
            )
            .unwrap();
    }

    Ok(())
}

pub fn wikis(wiki_prefixes: &[impl AsRef<str> + Display]) -> Vec<String> {
    wiki_prefixes
        .iter()
        .map(|prefix| format!("{prefix}wiki"))
        .collect()
}

pub async fn download_md5sums(
    wiki: impl Into<String> + Display,
    latest: &str,
) -> HashMap<String, String> {
    let url = format!("https://dumps.wikimedia.org/{wiki}/{latest}/{wiki}-{latest}-md5sums.txt");
    debug!("Download url: {url}");
    let resp = reqwest::get(&url).await.unwrap();

    if !resp.status().is_success() {
        eprintln!(
            "{}",
            format!("Error downloading md5sums: {}: {}", url, resp.status()).red()
        );
        exit(-1);
    }

    let mut sums = HashMap::new();
    for line in resp.text().await.unwrap().lines() {
        let split = line.split_once(' ').unwrap();
        sums.insert(split.1.trim().to_string(), split.0.trim().to_string());
    }

    sums
}

pub async fn verify_download(
    md5sums: &HashMap<String, String>,
    file_name: &str,
    location: impl AsRef<Path>,
) -> bool {
    let expected = md5sums.get(file_name).unwrap();
    let received = format!("{:x}", md5::compute(fs::read(location).unwrap()));
    expected == &received
}

pub fn unpack_gz_pb(
    path: impl AsRef<Path>,
    multi_pb: &MultiProgress,
    always_unpack: bool,
    try_remove: bool,
) -> Result<PathBuf, io::Error> {
    let path = path.as_ref();
    let mut out_path: PathBuf = PathBuf::from(path);
    out_path.set_extension("");

    let filename = &out_path.file_name().and_then(OsStr::to_str).unwrap();

    if always_unpack
        || !out_path.exists()
        || (out_path.exists() && out_path.metadata().unwrap().len() == 0)
    {
        if out_path.exists() && out_path.metadata().unwrap().len() == 0 {
            println!(
                "{}",
                format!("WARN: Unpacking because existing file seems to be empty").yellow()
            );
        }

        let t1 = Instant::now();
        let spinner = multi_pb.add(spinner_bar(&format!(" Unpacking file {filename}")));

        // Retry if unpacking fails
        let max_attempts = 10;
        let mut attempts = 0;
        loop {
            match unpack_gz(&out_path, fs::File::open(path).unwrap()) {
                Ok(_) => break,
                Err(e) => {
                    attempts += 1;
                    eprintln!(
                        "{}",
                        format!("Error unpacking file {filename}: {e}. Retrying ({attempts}/{max_attempts})")
                            .red()
                    );
                    // sleep for a bit before retrying
                    std::thread::sleep(Duration::from_secs(2));
                    if attempts >= max_attempts {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Failed to unpack file {filename} after {max_attempts} attempts: {e}"
                            ),
                        ));
                    }
                }
            }
        }

        spinner.finish_and_clear();
        multi_pb
            .println(
                format!("Unpacked file {filename} in {:?}", t1.elapsed())
                    .cyan()
                    .to_string(),
            )
            .unwrap();
    }

    if try_remove {
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed removing gzip file {path:?} {e}");
        }
    }

    Ok(out_path)
}

fn unpack_gz(out_path: &PathBuf, input_gz: fs::File) -> Result<(), Error> {
    let mut output = fs::File::create(&out_path)?;
    // let mut decoder: bufread::GzDecoder<BufReader<fs::File>> = bufread::GzDecoder::new(input_gz);
    let mut decoder = flate2::read::GzDecoder::new(input_gz);
    io::copy(&mut decoder, &mut output)?;
    Ok(())
}

// https://wikipedia.mirror.pdapps.org // russia 9-10 MiB/s BUT 2 Months behind
// https://wikidata.aerotechnet.com/ // US 2MiB/s BUT 2 Months behind

pub async fn test_get_url(rest: String) -> String {
    for mirror_base_url in MIRROR_URLS {
        let url = format!("{mirror_base_url}/{rest}");
        let r = reqwest::get(&url).await.unwrap();
        if r.status().is_success() {
            return url;
        }
    }
    // return String::new();
    eprintln!(
        "{}",
        format!(
            "Error downloading: {}. No mirror server provides this file.",
            rest
        )
        .red()
    );
    exit(-1);
}

/// Downloads and extracts sql files for {wiki_names} to {path} dir
///
pub async fn download_wikis(
    wiki_names: &[impl AsRef<str>],
    tables: &[impl AsRef<str>],
    path: impl Into<PathBuf>,
    dump_date: impl AsRef<str>,
    multi_pb: &MultiProgress,
) {
    // let latest = latest_wiki_subdir();
    let dump_date = dump_date.as_ref();

    let mut tasks = vec![];
    // let path = path.into().join(&latest).join("downloads");
    let path = path.into();
    fs::create_dir_all(&path).unwrap_or_else(|e| panic!("Failed creating dir: {path:?} {e}"));

    let wikimedia_url = "https://dumps.wikimedia.org"; // only allows 2 concurrent connections

    let base_url = "https://mirror.accum.se/mirror/wikimedia.org/dumps";

    // let mirror_urls = [
    //     "https://mirror.accum.se/mirror/wikimedia.org/dumps", // sweden 12MiB/s
    //     // "https://wikimedia.mirror.clarkson.edu", // new york 10MiB/s
    //     // "https://wikimedia.bringyour.com", // california 9MiB/s
    //     // "http://wikipedia.c3sl.ufpr.br", // brazil 7MiB/s
    //     // "https://dumps.wikimedia.org" // only allows 2 concurrent connections
    // ];
    //
    // let mut mirrors_used: HashMap<String, u32> = mirror_urls.iter().map(|s| (s.to_string(), 0)).collect();
    //
    // let mut mirror_queue: VecDeque<&str> = VecDeque::from(mirror_urls);

    println!("Download directory: {}", path.to_str().unwrap());
    for wiki in wiki_names {
        let wiki = wiki.as_ref();
        let wiki_hashes = download_md5sums(wiki, &dump_date).await;
        // dbg!(&wiki_hashes);
        for db in tables.iter() {
            let db = db.as_ref();
            // let base_url = mirror_queue.pop_front().unwrap();
            // *mirrors_used.get_mut(base_url).unwrap() += 1;
            //
            // if !(base_url == wikimedia_url && mirrors_used[base_url] >= 2) {
            //     mirror_queue.push_back(base_url);
            // }

            let url =
                test_get_url(format!("{wiki}/{dump_date}/{wiki}-{dump_date}-{db}.sql.gz")).await;
            let filename = format!("{wiki}-{dump_date}-{db}.sql.gz");

            // download_table(
            //                 filename,
            //                 url,
            //                 path.clone(),
            //                 multi_pb.clone(),
            //                 wiki_hashes.clone(),
            //                 false,
            //             )

            let multi_pb_clone = multi_pb.clone();
            let url_c = url.clone();
            let path_c = path.clone();
            let wiki_hashes_c = wiki_hashes.clone();

            let task = tokio::spawn(download_file_with_retries(
                move || {
                    let url = url_c.clone();
                    let path = path_c.clone();
                    let multi_pb = multi_pb_clone.clone();
                    let wiki_hashes = wiki_hashes_c.clone();

                    async move { download_file_bar(url, path, multi_pb, wiki_hashes).await }
                },
                3,
            ));

            tasks.push(task);
        }
    }

    let results = join_all(tasks).await;
    for (i, res) in results.into_iter().enumerate() {
        match res {
            Ok(_) => { /* task completed successfully */ }
            Err(e) => {
                panic!("Task {i} panicked or failed: {e}");
            }
        }
    }
}

pub fn clean_downloads(download_path: &Path, wiki_names: &[String]) {
    match fs::read_dir(download_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if let Some(wiki_name) = filename.split("-").next()
                        && wiki_names.contains(&wiki_name.to_string())
                    {
                        if let Err(e) = fs::remove_file(&path) {
                            eprintln!(
                                "{}",
                                format!("Failed to remove file {}: {}", filename, e).red()
                            );
                        }
                    }
                }
            }
        }
        _ => {
            eprintln!(
                "{}",
                format!("Failed to read directory {:?}", download_path).red()
            );
        }
    }
}
