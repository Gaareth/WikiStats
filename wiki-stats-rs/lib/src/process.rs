use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::{fs, vec};

use colored::Colorize;
use crossbeam::queue::ArrayQueue;
use futures_util::future::join_all;
use indicatif::MultiProgress;
use log::info;
use parse_mediawiki_sql::utils::memory_map;
use rusqlite::Connection;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::download::{self, clean_downloads};
use crate::download::{ALL_DB_TABLES, unpack_gz_pb};
use crate::sqlite::load::load_linktarget_map;
use crate::sqlite::title_id_conv::TitleIdMap;
use crate::sqlite::to_sqlite::{LinkTargetTitleMap, ToSqlite};
use crate::sqlite::{join_db_wiki_path, page_links, title_id_conv, wiki};

// 1_591_804_203 20240401 pagelinks
// 785_164_001 20240301 pagelinks
pub async fn process_wiki_to_db(
    wiki_name: impl AsRef<str>,
    base_directory: impl Into<PathBuf>,
    dump_date: impl AsRef<str>,
) {
    let wiki_name = wiki_name.as_ref();
    let dump_date = dump_date.as_ref();
    let base_directory = base_directory.into();

    let db_dir_path = base_directory.join("sqlite");
    fs::create_dir_all(&db_dir_path).unwrap();

    let base_sql = base_directory.join(format!("downloads/{wiki_name}-{dump_date}"));
    let base_sql_str = base_sql.to_str().unwrap();

    let db_path = db_dir_path.join(format!("{wiki_name}_database.sqlite"));

    let mb = MultiProgress::new();
    let tosqlite = ToSqlite::new_bar(wiki_name, dump_date, &mb, base_directory);
    tosqlite.create_db(
        &db_path,
        &format!("{base_sql_str}-pagelinks.sql"),
        &format!("{base_sql_str}-page.sql"),
        &format!("{base_sql_str}-linktarget.sql"),
    );

    tosqlite.post_insert(&db_path);
}

fn prepare(
    wiki_name: &str,
    multi_pb: &MultiProgress,
    base_directory: &Path,
    delete: bool,
) -> Result<(PathBuf, Connection), ()> {
    let db_dir_path = base_directory.join("sqlite");
    fs::create_dir_all(&db_dir_path).unwrap();
    let out_db_path = db_dir_path.join(format!("{wiki_name}_database.sqlite"));

    // if out_db_path.exists() {
    //     multi_pb.println(format!("sqlite file {out_db_path:?} already exists").red().to_string()).unwrap();
    //     if delete {
    //         let res = fs::remove_file(&out_db_path);
    //         if let Err(err) = res {
    //             println!("Failed removing file: {out_db_path:?}. Error: {err}");
    //         }
    //         info!("Removed sqlite file {out_db_path:?}");
    //     } else {
    //         return Err(());
    //     }
    // }

    let conn = Connection::open(&out_db_path).unwrap_or_else(|_| {
        panic!(
            "Failed creating database connection with {}",
            out_db_path.display()
        )
    });

    return Ok((out_db_path, conn));
}

struct DownloadOptions {
    always_download: bool,
    always_unpack: bool,
    try_remove: bool,
}

pub async fn process_wikis_seq(
    wiki_names: &[impl AsRef<str> + Debug],
    base_directory: impl Into<PathBuf>,
    dump_date_option: Option<String>,
    overwrite_sql: bool,
) -> String {
    let tables = ALL_DB_TABLES;

    let dump_date = if let Some(dump_date) = dump_date_option {
        if download::check_dump_complete_all(&wiki_names, &tables, &dump_date).await {
            dump_date
        } else {
            panic!("No new complete dump ready");
        }
    } else {
        download::latest_dump_date(&wiki_names, &tables, false, false)
            .await
            .expect("No new complete dump ready")
    };

    let base_directory = base_directory.into().join(&dump_date);
    let download_path = base_directory.join("downloads");

    // remove wikis that were done already
    let done_wiki_names = check_existing_sqlite_files(overwrite_sql, &wiki_names, &base_directory);
    let mut wiki_names: Vec<String> = wiki_names.iter().map(|s| s.as_ref().to_string()).collect();
    wiki_names.retain(|x| !done_wiki_names.contains(x));

    let multi_pb = MultiProgress::new();

    for wiki_name in &wiki_names {
        for table_name in tables {
            download::download_wikis(
                &[wiki_name],
                &[table_name],
                &download_path,
                &dump_date,
                &multi_pb,
            )
            .await;

            let gz_file_path =
                download_path.join(format!("{wiki_name}-{dump_date}-{table_name}.sql.gz"));
            unpack_gz_pb(&gz_file_path, &multi_pb, false, false).unwrap();
        }

        process_wiki_to_db(wiki_name, &base_directory, &dump_date).await;
    }

    dump_date
}

struct Options {
    num_sql_threads: u32,
    num_download_thread: u32,
    dump_date_no_fallback: bool,
    unpack_always_unpack: bool,
    unpack_try_remove: bool,
    sql_count_rows: bool,
    overwrite_sql: bool,
}

pub async fn process_threaded(
    wiki_names: &[impl AsRef<str>],
    base_directory: impl Into<PathBuf>,
    dump_date_option: Option<String>,
    overwrite_sql: bool,
) -> String {
    let t1 = Instant::now();
    let processed_tables = ALL_DB_TABLES;
    let wiki_names: Vec<String> = wiki_names.iter().map(|s| s.as_ref().to_string()).collect();

    let dump_date = if let Some(dump_date) = dump_date_option {
        if download::check_dump_complete_all(&wiki_names, &processed_tables, &dump_date).await {
            dump_date
        } else {
            panic!("No new complete dump ready");
        }
    } else {
        download::latest_dump_date(&wiki_names, &processed_tables, false, false)
            .await
            .expect("No new complete dump ready")
    };

    let base_directory = base_directory.into().join(&dump_date);
    let download_path = base_directory.join("downloads");
    let download_path = Arc::new(download_path);

    // remove wikis that were done already
    let done_wiki_names = check_existing_sqlite_files(overwrite_sql, &wiki_names, &base_directory);
    let mut wiki_names: Vec<String> = wiki_names.into_iter().map(|s| s.to_string()).collect();
    wiki_names.retain(|x| !done_wiki_names.contains(x));

    let num_jobs = wiki_names.len() * processed_tables.len();
    let num_sql_threads = 2;
    let num_download_threads = 2;

    let mut unpack_txs = vec![];
    let mut unpack_rxs = vec![];

    for _ in 0..num_jobs {
        let (tx, rx) = mpsc::channel::<(String, String, PathBuf)>(1);
        unpack_txs.push(tx);
        unpack_rxs.push(rx);
    }

    let mut tasks = vec![];

    let multi_pb = Arc::new(MultiProgress::new());

    let job_queue: Arc<ArrayQueue<(String, String, usize)>> = Arc::new(ArrayQueue::new(num_jobs));
    let mut job_counter = 0;
    for wiki_name in &wiki_names {
        for table_name in processed_tables {
            job_queue
                .push((wiki_name.clone(), table_name.to_string(), job_counter))
                .unwrap();
            job_counter += 1;
        }
    }

    let sql_queue = Arc::new(ArrayQueue::new(num_jobs));
    let (sql_tx, _) = broadcast::channel::<u8>(num_jobs);

    // Spawn download threads
    for tid in 0..num_download_threads {
        let job_queue = job_queue.clone();
        let multi_pb = multi_pb.clone();
        let unpack_txs = unpack_txs.clone();
        let download_path = download_path.clone();
        let dump_date = dump_date.clone();

        tasks.push(tokio::spawn(async move {
            while let Some((wiki_name, table_name, job_counter)) = job_queue.pop() {
                // multi_pb.println(format!("{tid}, {wiki_name}, {table_name} => {job_counter}")).unwrap();

                download::download_wikis(
                    &[&wiki_name],
                    &[&table_name],
                    &download_path.deref(),
                    &dump_date,
                    &multi_pb,
                )
                .await;

                let gz_file_path = download_path
                    .clone()
                    .join(format!("{wiki_name}-{dump_date}-{table_name}.sql.gz"));
                unpack_txs[job_counter]
                    .send((wiki_name.to_string(), table_name, gz_file_path))
                    .await
                    .unwrap();
            }
        }));
    }

    // drop original unpack senders
    drop(unpack_txs);

    // Spawn unpacker threads
    for mut unpack_rx in unpack_rxs.into_iter() {
        let multi_pb = multi_pb.clone();
        let sql_queue = sql_queue.clone();
        let sql_tx = sql_tx.clone();

        tasks.push(tokio::spawn(async move {
            while let Some((wiki_name, table_name, mut gz_file_path)) = unpack_rx.recv().await {
                // multi_pb.println(&format!("Unpacking: {wiki_name}, {table_name}")).unwrap();

                unpack_gz_pb(&gz_file_path, &multi_pb, false, false).unwrap();

                // *.sql.gz -> *.sql
                gz_file_path.set_extension("");
                let sql_file_path = gz_file_path;
                sql_queue
                    .push((wiki_name, table_name, sql_file_path))
                    .unwrap();

                // notify that there is data in the queue
                sql_tx.send(1).unwrap();
            }
        }));
    }

    let mut sql_wiki_queues = HashMap::new();
    for wiki_name in &wiki_names {
        sql_wiki_queues.insert(
            wiki_name.clone(),
            ArrayQueue::<String>::new(processed_tables.len()),
        );
    }

    // Spawn threads to create sqlite database from .sql files
    tasks.extend(tosqlite_threaded(
        &dump_date,
        base_directory,
        num_jobs,
        num_sql_threads,
        multi_pb,
        sql_queue,
        sql_tx,
        &wiki_names,
    ));

    join_all(tasks).await;
    println!("Total time elapsed: {:?}", t1.elapsed());
    dump_date
}

fn check_existing_sqlite_files(
    overwrite_sql: bool,
    wiki_names: &[impl AsRef<str>],
    base_directory: &PathBuf,
) -> Vec<String> {
    let mut done_wikis = vec![];

    for wiki_name in wiki_names {
        let wiki_name = wiki_name.as_ref();
        let output_file = join_db_wiki_path(base_directory.join("sqlite"), wiki_name);
        dbg!(&output_file);
        // when calling the command twice, it will error upon inserting if there already is data (Unique error).
        // that might be a bit late however, so check here first, that if a sqlite file exists it has to be empty
        if !overwrite_sql
            && fs::exists(&output_file).unwrap()
            && fs::metadata(&output_file).unwrap().len() > 0
        {
            println!(
                "{}: {:?} already exists and is not empty. Use --overwrite-sql",
                "Success".green(),
                output_file
            );
            done_wikis.push(wiki_name.to_string());
        } else if overwrite_sql {
            info!("Removing {output_file:?} as per --overwrite-sql");
            fs::remove_file(&output_file).expect("Failed removing file {output_file}");
        }
    }

    return done_wikis;
}

#[derive(Clone)]
struct PageLinksData {
    title_id_map: Option<TitleIdMap>,
    linktarget_title_map: Option<LinkTargetTitleMap>,
    pagelinks_sql_path: Option<PathBuf>,
}

impl PageLinksData {
    pub fn none() -> Self {
        PageLinksData {
            title_id_map: None,
            linktarget_title_map: None,
            pagelinks_sql_path: None,
        }
    }
}

fn tosqlite_threaded(
    dump_date: &str,
    base_directory: PathBuf,
    num_jobs: usize,
    num_sql_threads: i32,
    multi_pb: Arc<MultiProgress>,
    sql_queue: Arc<ArrayQueue<(String, String, PathBuf)>>,
    sql_tx: Sender<u8>,
    wiki_names: &[impl AsRef<str>],
) -> Vec<JoinHandle<()>> {
    let jobs_done_counter = Arc::new(AtomicUsize::new(0));

    let wiki_settings_map: HashMap<String, PageLinksData> = wiki_names
        .iter()
        .map(|s| s.as_ref().to_string())
        .zip(vec![PageLinksData::none(); wiki_names.len()])
        .collect();
    let wiki_settings_map = Arc::new(Mutex::new(wiki_settings_map));

    let mut tasks = vec![];
    // let mut seen_output_paths = Mutex::new(HashSet::new());

    for tid in 0..num_sql_threads {
        let base_directory = base_directory.clone();
        let dump_date = dump_date.to_string();
        let multi_pb = multi_pb.clone();
        let jobs_done_counter = jobs_done_counter.clone();
        let sql_queue = sql_queue.clone();
        let mut sql_rx = sql_tx.subscribe();
        let wiki_settings_map = wiki_settings_map.clone();

        tasks.push(tokio::task::spawn(async move {
            let mut tosqlite = ToSqlite::new_bar(
                "",
                dump_date,
                multi_pb.as_ref(),
                base_directory.parent().unwrap(),
            );

            sql_rx.recv().await.unwrap(); // don't spin lock while the input thread has not produced any results

            // actively wait for new data. I don't know anymore why I did it like this.
            while jobs_done_counter.load(Ordering::Relaxed) < num_jobs {
                // multi_pb.println(format!("{tid} waiting")).unwrap();
                if let Some((wiki_name, table_name, sql_file_path)) = sql_queue.pop() {
                    // multi_pb.println(format!("[{tid}] {wiki_name} {table_name}")).unwrap();

                    // todo:
                    let (out_db_path, mut conn) =
                        prepare(&wiki_name, &multi_pb, base_directory.as_path(), false).unwrap();
                    // seen_output_paths.lock().await.insert(out_db_path);

                    tosqlite.wiki_name.clone_from(&wiki_name);

                    match table_name.as_str() {
                        "pagelinks" => {
                            let mut w_mutex = wiki_settings_map.lock().await;
                            let pld = w_mutex.get_mut(&wiki_name).unwrap();

                            pld.pagelinks_sql_path = Some(sql_file_path.clone());
                            try_execute_pagelinks(&tosqlite, &mut conn, &out_db_path, pld).await;
                        }
                        "page" => {
                            tosqlite.create_title_id_conv_db(&sql_file_path, &mut conn);
                            title_id_conv::create_indices_post_setup(&conn);

                            // what(&multi_pb);

                            let map = title_id_conv::load_title_id_map(&out_db_path);

                            let mut w_mutex = wiki_settings_map.lock().await;
                            let pld = w_mutex.get_mut(&wiki_name).unwrap();
                            pld.title_id_map = Some(map);

                            try_execute_pagelinks(&tosqlite, &mut conn, &out_db_path, pld).await;
                        }
                        "linktarget" => {
                            let mmap = unsafe { memory_map(sql_file_path).unwrap() };

                            let map = load_linktarget_map(mmap);
                            let mut w_mutex = wiki_settings_map.lock().await;
                            let pld = w_mutex.get_mut(&wiki_name).unwrap();
                            pld.linktarget_title_map = Some(map);

                            try_execute_pagelinks(&tosqlite, &mut conn, &out_db_path, pld).await;
                        }
                        "categorylinks" => {
                            tosqlite.create_category_links_db(&sql_file_path, &mut conn)
                        }

                        _ => unimplemented!(),
                    }

                    jobs_done_counter.fetch_add(1, Ordering::Relaxed);
                }
                sleep(Duration::from_millis(1000)).await;
            }
        }))
    }

    return tasks;
}

async fn try_execute_pagelinks(
    tosqlite: &ToSqlite<'_>,
    conn: &mut Connection,
    out_db_path: &Path,
    page_links_data: &PageLinksData,
) {
    // pagelinks needs the title_id_map, so it knows which pageids are articles.
    if let Some(title_id_map) = &page_links_data.title_id_map {
        if let Some(linktarget_title_map) = &page_links_data.linktarget_title_map {
            // pagelinks needs the linktarget_title_map, so it can convert from linktarget to pagetitle. then to pageid using title_id_map.
            if let Some(pagelinks_sql_path) = &page_links_data.pagelinks_sql_path {
                tosqlite.create_pagelinks_db(
                    pagelinks_sql_path,
                    conn,
                    title_id_map,
                    linktarget_title_map,
                    false,
                );
                page_links::create_indices_post_setup(conn);

                // let prefix: String = tosqlite.wiki_name.chars().take(2).collect();
                // let valid = post_validation(&out_db_path, prefix, 1).await;
                // if !valid {
                //     panic!("Failed pagelinks validation. Database or input sql are wrong")
                // }
            }
        }
    }
}

// Parallel download and disk write is heavily bottlenecked?
pub async fn split_workload<T: Clone>(workload: &[T], num_threads: usize) -> Vec<Vec<T>> {
    assert!(num_threads > 0, "Number of thread should not be zero");

    let chunk_size = workload.len() / num_threads;
    let remaining = workload.len() % num_threads;
    let mut splitted_workload = vec![vec![]; num_threads];

    dbg!(&chunk_size);
    dbg!(&remaining);

    let mut iter = workload.iter();
    for i in 0..num_threads {
        let mut num_elements = chunk_size;

        // first ${remaining} threads get one more, to distribute the "leftover" elements.
        if i < remaining {
            num_elements += 1;
        }

        for _ in 0..num_elements {
            let v = iter.next().unwrap().clone();
            splitted_workload[i].push(v);
        }
    }

    splitted_workload
}

// I could not get the test env to run with multiple tokio threads
pub async fn test_bench_threaded() {
    let b = PathBuf::from("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/");

    for entry in fs::read_dir(b.join("sqlite")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("test-wiki")
        {
            fs::remove_file(&path).unwrap();
            println!("Deleted: {:?}", path);
        }
    }

    let t1 = Instant::now();
    let mb = Arc::new(MultiProgress::new());

    let mut tasks = vec![];

    // let test_page = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/test-page-dump.sql";
    let test_page =
        "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/small-page.sql";

    let test_pagelinks =
        "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/small-pagelinks.sql";

    let wikis = 4;
    let num_jobs = wikis * 1;
    let threads = 4;
    let queue = Arc::new(ArrayQueue::new(num_jobs));

    let mut wiki_names = vec![];

    for i in 0..wikis {
        wiki_names.push(format!("test-wiki-{i}"));

        queue
            .push((
                format!("test-wiki-{i}"),
                "page".to_string(),
                PathBuf::from(test_page),
            ))
            .unwrap();
        // queue.push((format!("test-wiki-{i}"), "pagelinks".to_string(),
        //             PathBuf::from(test_pagelinks))).unwrap();
    }

    let (sender, _) = broadcast::channel::<u8>(1);

    let sender_clone = sender.clone();
    tasks.push(tokio::spawn(async move {
        sender_clone.send(1).unwrap();
    }));

    tasks.extend(tosqlite_threaded(
        "20240501",
        b.clone(),
        num_jobs,
        threads,
        mb,
        queue,
        sender,
        &wiki_names,
    ));
    join_all(tasks).await;

    dbg!(&t1.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    // test for Unexpected EOF in copy of unpack gzipped file
    #[test]
    fn test_unexpected_eof() {}

    #[test]
    fn bench_insert() {
        // let mb = MultiProgress::new();
        // let b = PathBuf::from("/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/");
        //
        // let tosqlite = ToSqlite::new_bar("test-wiki", "20240501", &mb, &b);
        //
        // let test_page = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/small-page.sql";
        //
        // let test_pagelinks = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/small-pagelinks.sql";
        //
        // let test_linktarget = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki2/20240501/downloads/small-pagelinks.sql";
        //
        //
        // let test_db_path = b.join("sqlite").join("test.db");
        // let mut conn = Connection::open(&test_db_path).unwrap();
        // conn.execute("PRAGMA synchronous = OFF", ()).unwrap();
        // conn.execute("PRAGMA journal_mode = MEMORY", ());
        //
        //
        // tosqlite.create_title_id_conv_db(test_page, &mut conn);
        // tosqlite.create_pagelinks_db(test_pagelinks, &mut conn,
        //                              &title_id_conv::load_id_title_map(&test_db_path), false);
        //
        // // tosqlite.create_pagelinks_db(test_pagelinks, &mut conn, false);
        // fs::remove_file(test_db_path).unwrap();
    }

    #[tokio::test]
    async fn bench_threaded() {
        test_bench_threaded().await;
    }
}
