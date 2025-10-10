use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Instant;

use colored::Colorize;
use fxhash::FxHashMap;
use indicatif::{MultiProgress, ProgressBar};
use num_format::{Locale, ToFormattedString};
use parse_mediawiki_sql::field_types::{LinkTargetId, PageId, PageTitle};
use parse_mediawiki_sql::schemas::{CategoryLink, Page, PageLink};
use parse_mediawiki_sql::utils::{memory_map, Mmap};
use parse_mediawiki_sql::{iterate_sql_insertions, FromSqlTuple};
use rusqlite::types::Null;
use rusqlite::{CachedStatement, Connection};

use crate::calc::MAX_SIZE;
use crate::sqlite::category_links::pagetype_to_string;
use crate::sqlite::title_id_conv::TitleIdMap;
use crate::sqlite::{category_links, load, page_links, title_id_conv, wiki};
use crate::utils::{default_bar, default_bar_unknown, spinner_bar, write_barstyle};

//
type InsertFn<T, P> = fn(T) -> P;
type SkipFn<T> = fn(&T) -> bool;

type UniqueIndexFn = fn(conn: &mut Connection);

const MAX_ESTIMATED_SIZE: usize = 230_712_457;

pub struct InsertOptions<T, R> {
    insert_stmt: String,
    insert_fn: fn(&mut CachedStatement, R, &TitleIdMap, &LinkTargetTitleMap),
    from_fn: fn(T) -> R,
}

pub struct DuplicateOptions {
    mem_size_divisor: Option<f32>,
    unique_index_fn: Option<UniqueIndexFn>,
    skip_duplicates: bool,
}

impl DuplicateOptions {
    pub fn allow_duplicates() -> Self {
        Self {
            skip_duplicates: false,
            unique_index_fn: None,
            mem_size_divisor: None,
        }
    }

    pub fn skip_duplicates(unique_index_fn: UniqueIndexFn, mem_size_divisor: f32) -> Self {
        Self {
            skip_duplicates: true,
            unique_index_fn: Some(unique_index_fn),
            mem_size_divisor: Some(mem_size_divisor),
        }
    }
}

// lazy_static! {
//     static ref TITLE_ID_MAP: FxHashMap<PageTitle, PageId> = sqlite::title_id_conv::load_title_id_map();
// }

// pub fn setup(db_path: &str) -> Connection {
//     println!("Inserting into {db_path}");
//
//     let mut conn = Connection::open(db_path)
//         .unwrap_or_else(|_| panic!("Failed creating database connection with {db_path}"));
//
//     conn.execute("PRAGMA synchronous = OFF", ()).unwrap();
//     conn
// }

pub type LinkTargetTitleMap = FxHashMap<LinkTargetId, PageTitle>;

pub struct ToSqlite<'a> {
    pub wiki_name: String,
    dump_date: String,
    multi_pb: &'a MultiProgress,
    base_path: PathBuf,
}

impl<'a> ToSqlite<'a> {
    // pub fn new(wiki_name: impl Into<String>, dump_date: impl Into<String>, base_path: impl Into<PathBuf>) -> Self {
    //     let m_pb = MultiProgress::new();
    //     ToSqlite::new_bar(wiki_name, dump_date, m_pb, base_path)
    // }

    /// base_path: Path that contains the dump_date subdirectories (e.g: 20240501)
    pub fn new_bar(
        wiki_name: impl Into<String>,
        dump_date: impl Into<String>,
        multi_pb: &'a MultiProgress,
        base_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            wiki_name: wiki_name.into(),
            dump_date: dump_date.into(),
            multi_pb,
            base_path: base_path.into(),
        }
    }

    pub fn create_db(
        &self,
        db_path: &Path,
        pagelinks_sql_path: impl AsRef<Path>,
        page_sql_path: impl AsRef<Path>,
        linktarget_sql_path: impl AsRef<Path>,
    ) {
        // println!("-#--#- {wiki_name} -#--#-");
        // println!("[{wiki_name}] Inserting into database at: {db_path:?}");
        // println!("[{wiki_name}] Using {pagelinks_sql_path:?} and \n{page_sql_path:?}");

        let t1 = Instant::now();

        if Path::new(db_path).exists() {
            println!(
                "{}",
                format!("sqlite file {db_path:?} already exists").green()
            );
            exit(0);
        }

        let mut conn = Connection::open(db_path).unwrap_or_else(|_| {
            panic!(
                "[{}]  Failed creating database connection with {db_path:?}",
                &self.wiki_name
            )
        });

        conn.execute(
            "CREATE TABLE if not exists Info (
            is_done INTEGER,
            insertion_time_s INTEGER,
            index_creation_time_s INTEGER
        )",
            (),
        )
        .unwrap();

        conn.execute("INSERT INTO Info Values (?, ?, ?)", (0, Null, Null))
            .unwrap();

        conn.execute("PRAGMA synchronous = OFF", ()).unwrap();
        // conn.execute("PRAGMA journal_mode = OFF", ()).unwrap();

        wiki::create_db(&conn, &self.wiki_name);

        //  167_176_646
        // 6_566_564_65

        self.create_title_id_conv_db(&page_sql_path, &mut conn);

        // let map = title_id_conv::load_title_id_map(db_path);
        let lt_mmap: Mmap = unsafe { memory_map(&linktarget_sql_path).unwrap() };
        let lt_pt_map = load::load_linktarget_map(lt_mmap);

        let title_id_map = title_id_conv::load_title_id_map(db_path);
        self.create_pagelinks_db(
            &pagelinks_sql_path,
            &mut conn,
            &title_id_map,
            &lt_pt_map,
            false,
        );

        conn.execute(
            "INSERT INTO Info(insertion_time_s) Values (?)",
            (t1.elapsed().as_secs(),),
        )
        .unwrap();

        // post_insert(db_path);
        self.multi_pb
            .println(
                format!(
                    "[{}] Done. Total time elapsed: {:?}",
                    &self.wiki_name,
                    t1.elapsed()
                )
                .green()
                .to_string(),
            )
            .unwrap()
    }

    pub fn post_insert(&self, db_path: &Path) {
        let wiki_name = &self.wiki_name;
        self.multi_pb
            .println(format!(
                "[{wiki_name}] Creating indices for database at: {db_path:?}"
            ))
            .unwrap();

        let t1 = Instant::now();
        let conn = Connection::open(db_path).unwrap_or_else(|_| {
            panic!("[{wiki_name}]  Failed creating database connection with {db_path:?}")
        });

        let bar = spinner_bar("Creating [WikiPage] index");
        title_id_conv::create_indices_post_setup(&conn);
        title_id_conv::create_unique_index(&conn);
        bar.finish_with_message(format!("{:?}", t1.elapsed()));

        let t2 = Instant::now();
        let bar2 = spinner_bar("Creating [WikiLink] index");
        page_links::create_indices_post_setup(&conn);
        page_links::create_unique_index(&conn);
        bar2.finish_with_message(format!("{:?}", t2.elapsed()));

        conn.execute(
            "INSERT INTO Info(is_done, index_creation_time_s) Values (?, ?)",
            (1, t1.elapsed().as_secs()),
        )
        .unwrap();

        conn.execute("PRAGMA synchronous = ON", ()).unwrap();
        println!(
            "{}",
            format!(
                "[{wiki_name}] Done POSTINSERT. Total time elapsed: {:?}",
                t1.elapsed()
            )
            .green()
        );
    }

    pub fn create_pagelinks_db_custom(
        &self,
        in_sql_file_path: impl AsRef<Path>,
        in_linktarget_file_path: impl AsRef<Path>,
        page_db_path: impl AsRef<Path>,
        out_db_path: impl AsRef<Path>,
    ) {
        let mut conn = Connection::open(&out_db_path).unwrap_or_else(|_| {
            panic!(
                "[{}] Failed creating database connection with {}",
                self.wiki_name,
                out_db_path.as_ref().display()
            )
        });

        let lt_mmap: Mmap = unsafe { memory_map(&in_linktarget_file_path).unwrap() };
        let lt_pt_map = load::load_linktarget_map(lt_mmap);

        let title_id_map = title_id_conv::load_title_id_map(page_db_path);

        self.create_pagelinks_db(
            &in_sql_file_path,
            &mut conn,
            &title_id_map,
            &lt_pt_map,
            false,
        );
    }

    pub fn create_pagelinks_db_default(&self) {
        let (mmap_path, db_path) = self.prepare("pagelinks");
        let (lt_mmap_path, _) = self.prepare("linktarget");

        self.create_pagelinks_db_custom(mmap_path, lt_mmap_path, &db_path, &db_path);
    }

    pub fn prepare(&self, table_name: &str) -> (PathBuf, PathBuf) {
        let base_path_str = self.base_path.to_str().unwrap();
        let dump_date = &self.dump_date;
        let wiki_name = &self.wiki_name;

        let db_dir_path = PathBuf::from(base_path_str).join(dump_date).join("sqlite");

        fs::create_dir_all(&db_dir_path).unwrap();

        let base = format!("{base_path_str}/{dump_date}/downloads/{wiki_name}-{dump_date}");

        let db_path = db_dir_path.join(format!("{wiki_name}_{table_name}_database.sqlite"));

        let mmap_path = PathBuf::from(format!("{base}-{table_name}.sql"));

        (mmap_path, db_path)
    }

    pub fn create_pagelinks_db(
        &self,
        sql_file_path: impl AsRef<Path>,
        conn: &mut Connection,
        map: &TitleIdMap,
        lt_map: &LinkTargetTitleMap,
        count: bool,
    ) {
        self.multi_pb
            .println(format!(
                "[{}] {}",
                self.wiki_name,
                "--- WikiLink ---".purple()
            ))
            .unwrap();
        let mmap: Mmap = unsafe { memory_map(&sql_file_path).unwrap() };

        if map.is_empty() {
            panic!("titleid map cant be empty");
        }

        if lt_map.is_empty() {
            panic!("LinkTargetTitleMap map cant be empty");
        }

        page_links::db_setup(conn);

        let opts = DuplicateOptions::skip_duplicates(
            |conn| {
                page_links::create_unique_index(conn);
            },
            1.0,
        );

        // let opts = DuplicateOptions::allow_duplicates();
        type InsertType = (PageId, LinkTargetId);

        fn insert_pagelink(
            stmt: &mut CachedStatement,
            link: InsertType,
            map: &TitleIdMap,
            lt_pt_map: &LinkTargetTitleMap,
        ) {
            let from_id = link.0;
            let target_id = link.1;

            if let Some(pid) = lt_pt_map.get(&target_id).and_then(|pt| map.get(pt)) {
                let link = (from_id.0, pid.0);
                stmt.execute(link).unwrap();
            }
        }

        fn from_pagelink(link: PageLink) -> InsertType {
            (link.from, link.target)
        }

        let insrt_opts = InsertOptions {
            insert_stmt: "INSERT INTO WikiLink(page_id, page_link) VALUES (?, ?)".to_string(),
            insert_fn: insert_pagelink,
            from_fn: from_pagelink,
        };

        // self.count_progress_bar::<PageLink>(&mmap);
        let num_entries = if count {
            self.count_progress_bar::<PageLink>(&mmap)
        } else {
            MAX_ESTIMATED_SIZE
        };
        // let num_entries = MAX_SIZE as usize;

        // target namespace not in sql dump anymore since 1.43
        // let skip_fn = |pl: &PageLink| -> bool {
        //     pl.namespace.0 != 0 || pl.from_namespace.0 != 0
        // };

        let skip_fn = |pl: &PageLink| -> bool { pl.from_namespace.0 != 0 };

        // let data = load_sql_part_set::<PageLink>(mmap, (MAX_SIZE / 2) as usize, 1, skip_fn);
        // let data = load_sql_part_map(mmap, (MAX_SIZE / 10), 1);
        self.insert_directly::<PageLink, InsertType>(
            &mmap,
            conn,
            num_entries,
            &insrt_opts,
            skip_fn,
            opts,
            map,
            lt_map,
            "pagelinks",
        );

        // remove_duplicates(conn, "WikiLink", false);
        // page_links::create_indices_post_setup(conn);
    }

    pub fn create_title_id_conv_db_default(&self) {
        let (mmap, out_db_path) = self.prepare("page");
        let mut conn = Connection::open(&out_db_path).unwrap_or_else(|_| {
            panic!(
                "[{}] Failed creating database connection with {}",
                self.wiki_name,
                out_db_path.display()
            )
        });

        self.create_title_id_conv_db(&mmap, &mut conn);
    }

    pub fn create_title_id_conv_db(&self, sql_file_path: impl AsRef<Path>, conn: &mut Connection) {
        self.multi_pb
            .println(format!(
                "[{}] {}",
                self.wiki_name,
                "--- WikiPage ---".purple()
            ))
            .unwrap();
        conn.execute("PRAGMA synchronous = OFF", ()).unwrap();

        let mmap: Mmap = unsafe { memory_map(sql_file_path).unwrap() };

        title_id_conv::db_setup(conn);

        let opts = DuplicateOptions::skip_duplicates(
            |conn| {
                title_id_conv::create_unique_index(conn);
            },
            1.0,
        );

        // let opts = DuplicateOptions::allow_duplicates();
        type InsertType = (u32, String, u8);
        fn from_page(page: Page) -> InsertType {
            (page.id.0, page.title.0, page.is_redirect as u8)
        }

        //   type InsertType = (u32, String);
        // fn from_page(page: Page) -> InsertType {
        //     (page.id.0, page.title.0)
        // }

        fn insert_page(
            stmt: &mut CachedStatement,
            insert: InsertType,
            _: &TitleIdMap,
            _: &LinkTargetTitleMap,
        ) {
            // assert!(link.from_namespace.into_inner() == 0);
            let res = stmt.execute(insert).unwrap();
            // if res.is_err() {
            //     duplicates += 1;
            // }
            //
            //
            // count += 1;
        }

        let insrt_opts = InsertOptions {
            insert_stmt: "INSERT INTO WikiPage(page_id, page_title, is_redirect) VALUES (?, ?, ?)"
                .to_string(),
            insert_fn: insert_page,
            from_fn: from_page,
        };

        // let mut spinner = Spinner::new(Spinners::Dots3, "counting rows".to_string());

        // let num_entries = count_rows_sqlfile::<Page>(mmap);
        let num_entries_hint = 7_984_938; //7_984_938 // 7_549_140

        // spinner.stop();
        let skip_fn = |p: &Page| -> bool {
            p.namespace.0 != 0
            // false
            // p.is_redirect
        };

        self.insert_directly::<Page, InsertType>(
            &mmap,
            conn,
            num_entries_hint,
            &insrt_opts,
            skip_fn,
            opts,
            &FxHashMap::default(),
            &FxHashMap::default(),
            "page",
        );

        // title_id_conv::create_indices_post_setup(conn);
    }

    pub fn create_category_links_db(&self, sql_file_path: impl AsRef<Path>, conn: &mut Connection) {
        self.multi_pb
            .println(format!(
                "[{}] {}",
                self.wiki_name,
                "--- WikiCategoryLinks ---".purple()
            ))
            .unwrap();
        let mmap: Mmap = unsafe { memory_map(sql_file_path).unwrap() };

        category_links::db_setup(conn);

        let opts = DuplicateOptions::skip_duplicates(
            |conn| {
                category_links::create_unique_index(conn);
            },
            1.0,
        );

        // let opts = DuplicateOptions::allow_duplicates();
        type InsertType = (u32, String, String);

        fn from_cl(cl: CategoryLink) -> InsertType {
            (cl.from.0, cl.to.0, pagetype_to_string(cl.r#type))
        }

        fn insert_cl(
            stmt: &mut CachedStatement,
            insert: InsertType,
            _: &TitleIdMap,
            _: &LinkTargetTitleMap,
        ) {
            stmt.execute(insert).unwrap();
        }

        let insrt_opts = InsertOptions {
            insert_stmt: "INSERT INTO WikiCategoryLinks(page_id_from, category_name, category_type) VALUES (?, ?, ?)".to_string(),
            insert_fn: insert_cl,
            from_fn: from_cl,
        };

        // let num_entries = count_rows_sqlfile::<CategoryLink>(mmap);
        let num_entries = 17_973_536;

        let skip_fn = |p: &CategoryLink| -> bool {
            // p.r#type != PageType::Page
            false
        };

        self.insert_directly::<CategoryLink, InsertType>(
            &mmap,
            conn,
            num_entries,
            &insrt_opts,
            skip_fn,
            opts,
            &FxHashMap::default(),
            &FxHashMap::default(),
            "categorylinks",
        );

        // title_id_conv::create_indices_post_setup(conn);
    }

    pub fn insert_directly<'b, WikiType, InsertType>(
        &self,
        mmap: &'b Mmap,
        conn: &mut Connection,
        num_entries_hint: usize,
        insert_options: &InsertOptions<WikiType, InsertType>,
        skip_fn: SkipFn<WikiType>,
        duplicate_options: DuplicateOptions,
        map: &TitleIdMap,
        lt_map: &LinkTargetTitleMap,
        table_name: &str,
    ) where
        WikiType: Hash + Eq + FromSqlTuple<'b> + 'b,
        InsertType: Hash + Eq,
    {
        let t1 = Instant::now();

        let mut data = iterate_sql_insertions::<WikiType>(mmap);
        let data_iterator = data
            .into_iter()
            .filter(|wiki| !skip_fn(wiki)) // remove unwanted sql entries
            .map(|wiki| (insert_options.from_fn)(wiki)); // map from Wikipedia SQL to wanted type

        let total_inserted = self.insert_transaction(
            conn,
            num_entries_hint,
            insert_options,
            data_iterator,
            map,
            lt_map,
            table_name,
        );
        // dbg!(&total_inserted);
        // assert!(total_inserted > 0, "SQL File should contain at least one row");

        let t2 = Instant::now();
        // let mut sp = Spinner::new(Spinners::Dots, "Creating unique index".into());
        let sp = self.multi_pb.add(spinner_bar(" Creating unique index"));
        duplicate_options.unique_index_fn.unwrap()(conn);
        // sp.stop();
        sp.finish();

        self.multi_pb
            .println(format!("\nCreating unique index took: {:?}", t2.elapsed()))
            .unwrap();

        self.multi_pb.println("\n-- Total stats: --").unwrap();
        self.print_stats(t1, total_inserted);
    }

    fn insert_transaction<WikiType, InsertType, I: Iterator<Item = InsertType>>(
        &self,
        conn: &mut Connection,
        length_hint: usize,
        insert_options: &InsertOptions<WikiType, InsertType>,
        iterator: I,
        map: &TitleIdMap,
        lt_map: &LinkTargetTitleMap,
        table_name: &str,
    ) -> u64 {
        let t1 = Instant::now();
        let tx = conn.transaction().unwrap();

        let bar = ProgressBar::new(length_hint as u64);
        bar.set_style(write_barstyle(&format!(
            "[{}] {}",
            &self.wiki_name, table_name
        )));
        let bar = self.multi_pb.add(bar);

        self.multi_pb.println("Writing to database..").unwrap();

        let mut num_inserted = 0;
        {
            let mut stmt = tx.prepare_cached(&insert_options.insert_stmt).unwrap();
            for row in iterator {
                // stmt.execute((insert_options.insert_fn)(row)).unwrap();
                (insert_options.insert_fn)(&mut stmt, row, map, lt_map);
                bar.inc(1);
                num_inserted += 1;
            }

            bar.finish();
        }

        tx.commit().expect("Failed committing transaction :(");

        self.multi_pb
            .println("Successfully inserted data into db".green().to_string())
            .unwrap();

        self.print_stats(t1, num_inserted);

        num_inserted
    }

    fn print_stats(&self, t1: Instant, num_inserted: u64) {
        let elapsed = t1.elapsed();
        let total_secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
        let msg = format!(
            "Elapsed: {:#?} \nSpeed: {} rows/s",
            elapsed,
            ((num_inserted as f64 / total_secs) as u64)
                .to_formatted_string(&Locale::de)
                .bold()
        );
        self.multi_pb.println(msg).unwrap();
    }

    pub fn count_progress_bar<'b, T: FromSqlTuple<'b> + 'b>(&self, mmap: &'b Mmap) -> usize {
        self.multi_pb.println("Counting rows..").unwrap();
        let bar = self.multi_pb.add(default_bar_unknown());

        let mut count = 0;
        for _ in iterate_sql_insertions::<T>(mmap).into_iter() {
            bar.inc(1);
            count += 1;
        }

        bar.finish();
        self.multi_pb
            .println(format!("Found: {count} rows"))
            .unwrap();
        count
    }
}

//
// pub fn insert_iter<'a, WikiType, InsertType>(mmap: &'a Mmap, conn: &mut Connection,
//                                              num_entries: usize,
//                                              insert_options: &InsertOptions<WikiType, InsertType>,
//                                              skip_fn: SkipFn<WikiType>,
//                                              duplicate_options: DuplicateOptions,
//                                              map: &TitleIdMap,
// ) where WikiType: Hash + Eq + FromSqlTuple<'a> + 'a, InsertType: Hash + Eq {
//     {
//         if num_entries == 0 {
//             eprintln!("{}", "sql file should contain more than 0 entries".on_red().black());
//             exit(-1);
//         }
//
//         let t1 = Instant::now();
//
//         let mut data = iterate_sql_insertions::<WikiType>(mmap);
//         let data_iterator = data.into_iter()
//             .filter(|wiki| !skip_fn(wiki)) // remove unwanted sql entries
//             .map(|wiki| (insert_options.from_fn)(wiki));                     // map from Wikipedia SQL to wanted type
//
//         let total_inserted = insert_transaction(conn, num_entries, insert_options, data_iterator, map);
//
//         let t2 = Instant::now();
//         let mut sp = Spinner::new(Spinners::Dots, "Creating unique index".into());
//         duplicate_options.unique_index_fn.unwrap()(conn); // ()(_) lol
//         sp.stop();
//
//         println!();
//         println!("Creating unique index took: {:?}", t2.elapsed());
//
//         println!("\n-- Total stats: --");
//         print_stats(t1, total_inserted);
//     }
// }
//
//
// /// mem_size_divisor: a value of 2 means half of the rows will be loaded into memory for duplicate elimination
// pub fn insert_gradually<'a, WikiType, InsertType>
// (mmap: &'a Mmap, conn: &mut Connection,
//  num_entries: usize,
//  insert_options: &InsertOptions<WikiType, InsertType>,
//  skip_fn: SkipFn<WikiType>,
//  duplicate_options: DuplicateOptions,
//  map: &TitleIdMap,
// )
//     where WikiType: Hash + Eq + FromSqlTuple<'a> + 'a, InsertType: Hash + Eq {
//     if num_entries == 0 {
//         eprintln!("{}", "sql file should contain more than 0 entries".on_red().black());
//         exit(-1);
//     }
//
//     let t1 = Instant::now();
//     // let num_parts: f32 = 3.5;
//
//     // if num_parts > 1 {
//     //     conn.execute(
//     //         "CREATE UNIQUE INDEX if not exists WikiLinks_page_id_page_links_key ON
//     //        WikiLinks(page_id, page_links)", (),
//     //     ).expect("Failed creating unique index");
//     // }
//
//     let mut already_inserted = 0;
//     let mut total_inserted = 0;
//
//     // if duplicate_options.skip_duplicates {
//     //     let part_size: usize = (num_entries as f32 / duplicate_options.mem_size_divisor.unwrap()).round() as usize;
//     //
//     //     println!("Loading {part_size} entries into memory..");
//     //     let (data, row_count) =
//     //         load_sql_part_set_generic::<WikiType, InsertType>(mmap, part_size, 1, insert_options.from_fn, skip_fn);
//     //
//     //     // let (data, row_count) =
//     //     //     load_sql_part_set_generic_atleast::<WikiType, InsertType>(mmap, 2000, insert_options.from_fn, skip_fn);
//     //
//     //     // dbg!(&data.len());
//     //     // let data: FxHashSet<InsertType> = data.into_iter().take(1000).collect();
//     //     already_inserted = row_count;
//     //
//     //     // total_inserted is not necessarily equal to already_inserted
//     //     // total_inserted counts how many rows were actually inserted
//     //     // already_inserted counts how many rows were skipped
//     //     total_inserted += insert_transaction(conn, already_inserted as usize, insert_options, data.into_iter(), map);
//     //
//     //     if part_size < num_entries {
//     //         println!("Skipped {already_inserted} [~{}%] entries", (already_inserted as usize / num_entries) * 100);
//     //     }
//     // }
//
//     dbg!(&num_entries);
//     dbg!(&already_inserted);
//     let rest_entries = num_entries - already_inserted as usize;
//     // let rest_entries = 0;
//
//     if rest_entries > 0 {
//         println!("Rest rows: {rest_entries}");
//         let t1 = Instant::now();
//         let mut sp = Spinner::new(Spinners::Dots, "Creating unique index".into());
//         duplicate_options.unique_index_fn.unwrap()(conn); // ()(_) lol
//         sp.stop();
//
//         println!();
//         println!("Creating unique index took: {:?}", t1.elapsed());
//
//         let mut data = iterate_sql_insertions::<WikiType>(mmap);
//         let data_iterator = data.into_iter()
//             .skip(already_inserted as usize)                                 //skip already looked at rows
//             .filter(|wiki| !skip_fn(wiki))  // remove unwanted sql entries
//             .map(|wiki| (insert_options.from_fn)(wiki));                            // map from Wikipedia SQL to wanted type
//
//         total_inserted += insert_transaction(conn, rest_entries, insert_options, data_iterator, map);
//     }
//
//
//     println!("\n-- Total stats: --");
//     print_stats(t1, total_inserted);
// }

pub fn count_rows_sqlfile<'a, T: FromSqlTuple<'a> + 'a>(mmap: &'a Mmap) -> usize {
    iterate_sql_insertions::<T>(mmap).count()
}

// pub fn count_rows_sqlfile_path<'a, T: FromSqlTuple<'a> + 'a>(path: impl AsRef<Path>) {
//     let mmap = unsafe { memory_map(path).unwrap() };
//     let rows = iterate_sql_insertions::<T>(&mmap).count();
// }
//

pub fn count_duplicates(conn: &Connection, table_name: &str) {
    // let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn
        .prepare(&format!("Select DISTINCT * from {table_name};"))
        .unwrap();

    let rows = stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap();

    // dbg!(rows.count());
    let bar = default_bar(MAX_SIZE as u64);
    let mut c = 0;
    for row in rows {
        let res: u32 = row.unwrap();
        // dbg!(&res);
        c += 1;
        bar.inc(1);
    }
    bar.finish();
    dbg!(&c);
}

pub fn remove_duplicates(conn: &mut Connection, table_name: &str, count: bool) {
    let t1 = Instant::now();

    if count {
        count_duplicates(conn, table_name);
    }

    conn.execute_batch(&format!(
        "CREATE TABLE temp_table as SELECT DISTINCT * FROM {table_name};
            DROP TABLE {table_name};
            ALTER TABLE temp_table RENAME TO {table_name}"
    ))
    .unwrap();

    dbg!(&t1.elapsed());
    println!("Removed duplicates");

    if count {
        count_duplicates(conn, table_name);
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlite::load::{load_linktarget_map, load_title_id_map};

    use super::*;

    #[test]
    fn parse_pagelink_format() {
        let mmap = unsafe { memory_map("tests/data/small/test-20240901-pagelinks.sql").unwrap() };
        let rows = iterate_sql_insertions::<PageLink>(&mmap).take(1).next();
        assert!(rows.is_some());
        dbg!(&rows);
    }

    #[test]
    fn create_pagelinks() {
        let pl_path = "tests/data/small/test-20240901-pagelinks.sql";

        let pt_path = "tests/data/small/test-20240901-page.sql";
        let lt_path = "tests/data/small/test-20240901-linktarget.sql";

        let pt_mmap = unsafe { memory_map(pt_path).unwrap() };
        let pt_map = load_title_id_map(pt_mmap);

        let lt_mmap = unsafe { memory_map(lt_path).unwrap() };
        let lt_map = load_linktarget_map(lt_mmap);

        // can't get tempdir to work here?
        let out_db_path = "tests/data/temp/data.sqlite";
        fs::remove_file(out_db_path).expect("Failed removing file");
        let mut conn = Connection::open(out_db_path).unwrap();

        let dump_date = "20240901";
        let multi_pb = MultiProgress::new();
        let base_directory = "";

        let tosqlite = ToSqlite::new_bar("test", dump_date, &multi_pb, base_directory);
        tosqlite.create_pagelinks_db(pl_path, &mut conn, &pt_map, &lt_map, false);

        assert!(
            fs::metadata(out_db_path)
                .expect("Failed to get metadata")
                .len()
                > 0
        );

        fs::remove_file(out_db_path).expect("Failed removing file");
    }
}
