use rusqlite::Connection;

pub fn create_db(conn: &Connection, wiki_name: &str) {
    db_setup(conn);
    conn.execute("INSERT OR IGNORE INTO Wiki(name) VALUES (?1)", (wiki_name, )).unwrap();
}

fn db_setup(conn: &Connection) {
    conn.execute(
        "CREATE TABLE if not exists Wiki (
            name TEXT PRIMARY KEY
        )",
        (),
    ).expect("Failed creating table");
}