use std::fmt::Debug;
use std::path::Path;

use fxhash::{FxHashMap, FxHashSet};
use parse_mediawiki_sql::field_types::{PageId, PageTitle};
use parse_mediawiki_sql::iterate_sql_insertions;
use parse_mediawiki_sql::schemas::{Page};
use parse_mediawiki_sql::utils::Mmap;
use rusqlite::Connection;


pub fn page_id_to_title(id: &PageId, conn: &Connection) -> Option<PageTitle> {
    // let conn = Connection::open(DB_PAGE).unwrap();

    // let where_wiki = where_wiki_name.map(|wiki_name| format!("AND WikiPage.wiki_name = '{wiki_name}'")).unwrap_or_default();


    let mut stmt = conn.prepare(
        "SELECT page_title FROM WikiPage WHERE page_id = ?1"
    ).unwrap();
    let mut rows = stmt.query_map([id.0], |row| row.get(0)).unwrap();
    let title = rows.next();

    // return title.and_then(Result::ok);


    // match title {
    //     Some(title) => title.ok().and_then(PageTitle).and_then(),
    //     None => None
    // }

    // match title {
    //     Some(title) => match title {
    //         Ok(s) => Some(PageTitle(s)),
    //         Err(_) => None,
    //     },
    //     None => None
    // }

    title.and_then(|result| result.ok().map(PageTitle))
}

pub fn page_title_to_id(title: &PageTitle, conn: &Connection) -> Option<PageId> {
    let mut stmt = conn.prepare("SELECT page_id FROM WikiPage where page_title = ?1").unwrap();

    let mut rows = stmt.query_map([title.clone().0], |row| row.get(0)).unwrap();
    let title = rows.next();

    // return title.and_then(Result::ok);

    // match title {
    //     Some(title) => title.ok().and_then(PageTitle).and_then(),
    //     None => None
    // }
    // match title {
    //     Some(title) => match title {
    //         Ok(s) => Some(PageId(s)),
    //         Err(_) => None,
    //     },
    //     None => None
    // }
    title.map(|title| PageId(title.unwrap()))
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct WikiPage {
    pub id: u32,
    pub title: String,
    pub is_redirect: bool,
}

pub fn load_wiki_pages(page_db_path: impl AsRef<Path>) -> Vec<WikiPage> {
    // let path = "/home/gareth/dev/Rust/WikiGame/page_db.db";
    // let page_db_path = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/ja_page_db.sqlite";
    let conn = Connection::open(page_db_path).unwrap();
    // let mut stmt = conn.prepare(
    //     &format!("SELECT page_id, page_title FROM WikiPage where wiki_name = '{wiki_name}'")).unwrap();

    let mut stmt = conn.prepare(
        "SELECT page_id, page_title, is_redirect FROM WikiPage").unwrap();

    let rows = stmt.query_map([],
                              |row| {
                                  Ok(WikiPage {
                                      id: row.get(0).unwrap(),
                                      title: row.get(1).unwrap(),
                                      is_redirect: row.get::<usize, u32>(2).unwrap() == 1,
                                  })
                              }).unwrap();

    let mut row_vec: Vec<WikiPage> = vec![];
    for row in rows {
        let row = row.unwrap();
        row_vec.push(row);
    }

    row_vec
}


//TODO: return iterator
pub fn load_rows_from_page(page_db_path: impl AsRef<Path>) -> Vec<(PageId, PageTitle)> {
    // let path = "/home/gareth/dev/Rust/WikiGame/page_db.db";
    // let page_db_path = "/run/media/gareth/7FD71CF32A89EF6A/dev/wiki/sqlite/ja_page_db.sqlite";
    let conn = Connection::open(page_db_path).unwrap();
    // let mut stmt = conn.prepare(
    //     &format!("SELECT page_id, page_title FROM WikiPage where wiki_name = '{wiki_name}'")).unwrap();

    let mut stmt = conn.prepare(
        "SELECT page_id, page_title FROM WikiPage").unwrap();

    let rows = stmt.query_map([],
                              |row| {
                                  Ok(WikiPage {
                                      id: row.get(0).unwrap(),
                                      title: row.get(1).unwrap(),
                                      is_redirect: false, // not used, unknown here, does not matter
                                  })
                              }).unwrap();

    let mut row_vec: Vec<(PageId, PageTitle)> = vec![];
    for row in rows {
        let row = row.unwrap();
        row_vec.push((PageId(row.id), PageTitle(row.title)));
    }

    row_vec
}


pub type TitleIdMap = FxHashMap<PageTitle, PageId>;

pub fn load_title_id_map(path: impl AsRef<Path>) -> TitleIdMap {
    let mut pagetitle_to_map = FxHashMap::default();
    for (id, title) in load_rows_from_page(path) {
        pagetitle_to_map.entry(title).or_insert(id);
    }
    pagetitle_to_map
}

pub type IdTitleMap = FxHashMap<PageId, PageTitle>;

// TODO: add some info to the loading bar
pub fn load_id_title_map(path: impl AsRef<Path>) -> IdTitleMap {
    let mut pageid_title_map = FxHashMap::default();
    for (id, title) in load_rows_from_page(path) {
        pageid_title_map.entry(id).or_insert(title);
    }
    pageid_title_map
}

pub fn db_setup(conn: &Connection) {
    conn.execute(
        "CREATE TABLE if not exists WikiPage (
             page_id integer not null,
             page_title text not null,
             is_redirect integer
         )",
        (),
    ).expect("Failed creating table");
                 // is_redirect integer

}

pub fn create_indices_post_setup(conn: &Connection) {
    conn.execute(
        "CREATE INDEX if not exists idx_title_id ON WikiPage(page_title);",
        (),
    ).expect("Failed creating index");

    conn.execute(
        "CREATE INDEX if not exists idx_page_id ON WikiPage(page_id);",
        (),
    ).expect("Failed creating index");
}

pub fn create_unique_index(conn: &Connection) {
    conn.execute(
        "CREATE UNIQUE INDEX if not exists WikiPage_unique_index ON
           WikiPage(page_id, page_title)", (),
    ).expect("Failed creating unique index");
}

pub fn count_articles(mmap: &Mmap) {
    let count = iterate_sql_insertions::<Page>(&mmap).filter(|p| !p.is_redirect && p.namespace.0 == 0).count();
    dbg!(&count);
}

pub fn count_duplicates(db_path: &str) {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn.prepare("SELECT page_id, page_title, COUNT(*) FROM WikiPage \
            GROUP BY page_id, page_title HAVING COUNT(*) > 1").unwrap();

    let rows = stmt.query_map([], |row|
        Ok((row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap()))).unwrap();

    for row in rows {
        let res: (u32, u32, u32) = row.unwrap();
        dbg!(&res);
    }
}

pub fn get_random_page(db_path: &Path, num: u32) -> FxHashSet<WikiPage> {
    let stmt_str = "SELECT page_id, page_title, is_redirect FROM WikiPage ORDER BY RANDOM() LIMIT ?1";
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare(stmt_str).unwrap();
    // dbg!(&stmt);

    let res = stmt.query_map(
        [num], |row|
            Ok(WikiPage {
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                is_redirect: row.get::<usize, u32>(2).unwrap() == 1,
            })).unwrap();
    let r = res.map(|r| r.unwrap());
    r.collect()
}

#[cfg(test)]
mod page_test {
    use std::sync::{Arc, Mutex, OnceLock};

    use parse_mediawiki_sql::field_types::{PageId, PageTitle};
    use rusqlite::Connection;

    use crate::sqlite::db_wiki_path;
    use crate::sqlite::title_id_conv::{load_id_title_map, load_title_id_map, page_id_to_title, page_title_to_id};

    fn path() -> &'static str {
        dotenv::dotenv().unwrap();
        static DB_PATH: OnceLock<String> = OnceLock::new();
        DB_PATH.get_or_init(|| {
            db_wiki_path("dewiki")
        })
    }

    fn connection() -> &'static Arc<Mutex<Connection>> {
        static CONN: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();
        CONN.get_or_init(|| {
            Arc::new(Mutex::new(Connection::open(path()).unwrap()))
        })
    }

    #[test]
    fn test_page_id_to_title() {
        let pid = page_title_to_id(&PageTitle("Angela_Merkel".to_string()), &connection().lock().unwrap());
        assert_eq!(pid.unwrap(), PageId(145));
    }

    #[test]
    fn test_title_page_id() {
        let ptitle = page_id_to_title(&PageId(145), &connection().lock().unwrap());
        assert_eq!(ptitle.unwrap(), PageTitle("Angela_Merkel".to_string()));
    }

    #[test]
    fn test_load_maps() {
        load_title_id_map(path());
        load_id_title_map(path());
    }
}