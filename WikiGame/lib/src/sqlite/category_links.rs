use parse_mediawiki_sql::field_types::PageType;
use rusqlite::Connection;

pub fn pagetype_to_string(s: PageType) -> String {
    use PageType::*;
    match s {
        Page => "page".to_string(),
        Subcat => "subcat".to_string(),
        File => "file".to_string(),
    }
}

pub fn db_setup(conn: &Connection) {
    conn.execute(
        "CREATE TABLE if not exists WikiCategoryLinks (
            page_id_from INTEGER,
            category_name TEXT,
            category_type TEXT /* enum('page','subcat','file') */
        )",
        (),
    ).expect("Failed creating table");
//            UNIQUE (page_id, page_link)
    // conn.execute(
    //         "CREATE UNIQUE INDEX WikiLinks_page_id_page_links_key ON
    //         WikiLinks(page_id, page_links)", ()
    // );

    // conn.execute("PRAGMA journal_mode = MEMORY", ()).unwrap();
}

pub fn create_unique_index(conn: &Connection) {
    conn.execute(
        "CREATE UNIQUE INDEX if not exists WikiCategoryLinks_unique_index ON
           WikiCategoryLinks(page_id_from, category_name, category_type)", (),
    ).expect("Failed creating unique index");
}

pub fn create_indices_post_setup(conn: &Connection) {
    // println!("Creating index..");
    // conn.execute(
    //     "CREATE INDEX if not exists idx_link_id ON WikiCategoryLinks(page_id);",
    //     (),
    // ).expect("Failed creating index");
    // 
    // conn.execute(
    //     "CREATE INDEX if not exists idx_link_page ON WikiCategoryLinks(page_link);",
    //     (),
    // ).expect("Failed creating index");
    // 
}
