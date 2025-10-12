use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::path::Path;

use fxhash::FxHashSet;
use indicatif::MultiProgress;
use parse_mediawiki_sql::utils::Mmap;
use parse_mediawiki_sql::{FromSqlTuple, iterate_sql_insertions};
use rusqlite::{Connection, Row};

use crate::sqlite::load::load_sql_part_set_generic;
use crate::utils::default_bar;

pub fn select_all<T: Eq + Hash>(
    db_path: impl AsRef<Path>,
    table: &str,
    unwrap_row: fn(row: &Row) -> T,
) -> FxHashSet<T> {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn.prepare(&format!("SELECT * FROM {table}")).unwrap();

    let rows = stmt.query_map([], |row| Ok(unwrap_row(&row))).unwrap();
    // (row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap())

    let mut data = FxHashSet::default();
    for row in rows {
        let res: T = row.unwrap();
        data.insert(res);
    }
    data
}

pub fn diff(path1: impl AsRef<Path>, path2: impl AsRef<Path>) {
    let old_data = select_all::<(u32, u32)>(&path1, "WikiLink", |row| {
        (row.get(0).unwrap(), row.get(1).unwrap())
    });

    let new_data = select_all::<(u32, u32)>(&path1, "WikiLink", |row| {
        (row.get(0).unwrap(), row.get(1).unwrap())
    });

    // let symm_diff = old_data.symmetric_difference(&new_data);
    // dbg!(&symm_diff.count());

    let new = new_data.difference(&old_data);
    dbg!(&new.count());
    let deleted = old_data.difference(&new_data);
    dbg!(deleted.count());
}

pub fn diff_sqldump<'a, T: FromSqlTuple<'a> + 'a + Eq + Hash>(
    mmap1: &'a Mmap,
    mmap2: &'a Mmap,
    skip: fn(&T) -> bool,
    format: Box<dyn Fn(T) -> String>,
) {
    let old_data = load_sql_part_set_generic::<T, T>(mmap1, usize::MAX, 0, |p| p, skip);

    let mut new_count = 0;
    let mut diffs = vec![];

    let mbar = MultiProgress::new();
    let bar = mbar.add(default_bar(u32::MAX as u64));
    let bar2 = mbar.add(default_bar(u32::MAX as u64));

    let file = File::create("poem2.txt").unwrap();

    for new in iterate_sql_insertions::<T>(mmap2).into_iter() {
        bar.inc(1);

        if skip(&new) {
            continue;
        }

        if !old_data.0.contains(&new) {
            new_count += 1;
            bar2.inc(1);
            diffs.push(format(new));
            // writeln!(&file, "{}", format(new)).unwrap();
        }
    }

    // fs::write(path, json).unwrap();

    for diff in diffs {
        writeln!(&file, "{}", diff).unwrap();
    }
    bar.finish();
    dbg!(&new_count);
}
