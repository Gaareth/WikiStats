use std::hash::Hash;

use fxhash::{FxHashMap, FxHashSet};
use indicatif::ProgressStyle;
use parse_mediawiki_sql::field_types::{LinkTargetId, PageId, PageTitle};
use parse_mediawiki_sql::schemas::{LinkTarget, Page, PageLink};
use parse_mediawiki_sql::utils::Mmap;
use parse_mediawiki_sql::{FromSqlTuple, iterate_sql_insertions};

use crate::calc::MAX_SIZE;
use crate::utils::default_bar;

// pub fn load_pagelinks_map(pagelinks: Mmap) -> FxHashMap<LinkTargetId, PageTitle> {
//     load_map::<_, _, PageLink>(
//         &pagelinks,
//         |lt| (lt.id, lt.title),
//         |lt| lt.namespace.0 != 0,
//     )
// }

pub fn load_linktarget_map(linktarget_map: Mmap) -> FxHashMap<LinkTargetId, PageTitle> {
    load_map::<_, _, LinkTarget>(
        &linktarget_map,
        |lt| (lt.id, lt.title),
        |lt| lt.namespace.0 != 0,
    )
}

pub fn load_title_id_map(page_mmap: Mmap) -> FxHashMap<PageTitle, PageId> {
    load_map::<_, _, Page>(&page_mmap, |lt| (lt.title, lt.id), |lt| lt.namespace.0 != 0)
}

pub fn load_links_map<'a, K: Eq + Hash, V, I: FromSqlTuple<'a> + 'a, FInsert, FSkip>(
    linktarget_map: &'a Mmap,
    insert_fn: FInsert,
    skip_fn: FSkip,
) -> FxHashMap<K, Vec<V>>
where
    FInsert: Fn(I) -> (K, V),
    FSkip: Fn(&I) -> bool,
{
    let bar = indicatif::ProgressBar::new(MAX_SIZE as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7} {eta_precise}",
        )
        .unwrap(),
    );

    let mut map: FxHashMap<K, Vec<V>> = FxHashMap::default();

    for row in iterate_sql_insertions::<I>(&linktarget_map).into_iter() {
        bar.inc(1);

        if skip_fn(&row) {
            continue;
        }

        let (key, value) = insert_fn(row);
        map.entry(key).or_insert_with(Vec::new).push(value);
    }

    bar.finish();
    map
}

pub fn load_map<'a, K: Eq + Hash, V, I: FromSqlTuple<'a> + 'a>(
    linktarget_map: &'a Mmap,
    insert_fn: fn(I) -> (K, V),
    skip_fn: fn(&I) -> bool,
) -> FxHashMap<K, V> {
    let bar = indicatif::ProgressBar::new(MAX_SIZE as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7} {eta_precise}",
        )
            .unwrap(),
    );

    let mut map: FxHashMap<K, V> = FxHashMap::default();

    for row in iterate_sql_insertions::<I>(&linktarget_map).into_iter() {
        bar.inc(1);

        if skip_fn(&row) {
            continue;
        }

        let (key, value) = insert_fn(row);
        map.insert(key, value);
    }

    bar.finish();
    map
}

// pub fn load_sql_part_full(
//     pagelinks_sql: Mmap
// ) -> FxHashMap<PageId, Vec<PageId>> {
//     let bar = indicatif::ProgressBar::new(MAX_SIZE as u64);
//     bar.set_style(
//         ProgressStyle::with_template(
//             "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7} {eta_precise}",
//         )
//             .unwrap(),
//     );
//
//     let mut pageid_link_map: FxHashMap<PageId, Vec<PageId>> = FxHashMap::default();
//
//
//     for pagelink in iterate_sql_insertions::<PageLink>(&pagelinks_sql).into_iter() {
//         bar.inc(1);
//
//         if pagelink.from_namespace.0 != 0 {
//             continue;
//         }
//
//
//         pageid_link_map
//             .entry(pagelink.from)
//             .or_default()
//             .push(pagelink.target);
//     }
//
//     bar.finish();
//     pageid_link_map
// }

pub fn load_sql_part_map(
    pagelinks_sql: Mmap,
    part_size: u32,
    start_part: u32,
) -> FxHashMap<PageId, Vec<LinkTargetId>> {
    let bar = indicatif::ProgressBar::new(part_size as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7} {eta_precise}",
        )
            .unwrap(),
    );

    let mut pageid_link_map: FxHashMap<PageId, Vec<LinkTargetId>> = FxHashMap::default();

    let mut row_count = 0;
    let mut read_rows = 0;
    for pagelink in iterate_sql_insertions::<PageLink>(&pagelinks_sql).into_iter() {
        row_count += 1;

        let current_part: u32 = (row_count as f32 / part_size as f32).ceil() as u32;
        if current_part < start_part {
            continue;
        }

        if current_part > start_part {
            break;
        }

        bar.inc(1);

        // || pagelink.namespace.0 != 0
        // namespace zero are articles (what we want)
        if pagelink.from_namespace.0 != 0 {
            continue;
        }

        pageid_link_map
            .entry(pagelink.from)
            .or_default()
            .push(pagelink.target);

        read_rows += 1;
    }

    bar.finish();
    pageid_link_map
}

pub fn load_sql_full<'a, I: Hash + Eq + FromSqlTuple<'a> + 'a, R: Hash + Eq>(
    pagelinks_sql: &'a Mmap,
    insert_fn: fn(I) -> R,
    skip_fn: fn(&I) -> bool,
) -> FxHashSet<R> {
    let mut row_set: FxHashSet<R> = FxHashSet::default();
    let mut row_count = 0;

    for row in iterate_sql_insertions::<I>(pagelinks_sql).into_iter() {
        row_count += 1;

        if skip_fn(&row) {
            continue;
        }

        row_set.insert(insert_fn(row));
    }

    row_set
}

pub fn load_sql_part_set_generic<'a, I: Hash + Eq + FromSqlTuple<'a> + 'a, R: Hash + Eq>(
    pagelinks_sql: &'a Mmap,
    part_size: usize,
    start_part: usize,
    insert_fn: fn(I) -> R,
    skip_fn: fn(&I) -> bool,
) -> (FxHashSet<R>, u64) {
    println!("Loading sql file.. with {part_size} entries");
    let bar = default_bar(part_size as u64);

    // let mut pageid_link_map: FxHashSet<(PageId, PageTitle)> = FxHashSet::default();

    let mut row_set: FxHashSet<R> = FxHashSet::default();

    let mut row_count = 0;
    let mut read_rows = 0;

    for row in iterate_sql_insertions::<I>(pagelinks_sql).into_iter() {
        row_count += 1;

        let current_part: usize = (row_count as f32 / part_size as f32).ceil() as usize;
        if current_part < start_part {
            continue;
        }

        if current_part > start_part {
            break;
        }
        // namespace zero are articels (what we want)
        // if row.from_namespace.into_inner() != 0 {
        //     // dbg!(&link);
        //     // dbg!(&page_id);
        //     continue;
        // }

        bar.inc(1);

        if skip_fn(&row) {
            continue;
        }

        if read_rows > part_size {
            dbg!(current_part);
        }

        // pageid_link_map.entry(pagelink.from).or_insert_with(Vec::new).push(pagelink.title);

        row_set.insert(insert_fn(row));

        // pageid_link_map.insert((pagelink.from, pagelink.title));
        read_rows += 1;
    }

    bar.finish();
    (row_set, row_count)
}

pub fn load_sql_part_set<'a, T: FromSqlTuple<'a> + 'a>(
    pagelinks_sql: &'a Mmap,
    part_size: usize,
    start_part: usize,
    skip_fn: fn(&T) -> bool,
) -> (FxHashSet<T>, u64)
where
    T: Hash,
    T: Eq,
{
    println!("Loading sql file.. with {part_size} entries");
    let bar = default_bar(part_size as u64);

    // let mut pageid_link_map: FxHashSet<(PageId, PageTitle)> = FxHashSet::default();

    let mut row_set: FxHashSet<T> = FxHashSet::default();

    let mut row_count = 0;
    let mut read_rows = 0;

    for row in iterate_sql_insertions::<T>(pagelinks_sql).into_iter() {
        row_count += 1;

        let current_part: usize = (row_count as f32 / part_size as f32).ceil() as usize;
        if current_part < start_part {
            continue;
        }

        if current_part > start_part {
            break;
        }
        // namespace zero are articles (what we want)
        // if row.from_namespace.into_inner() != 0 {
        //     // dbg!(&link);
        //     // dbg!(&page_id);
        //     continue;
        // }

        if skip_fn(&row) {
            continue;
        }

        bar.inc(1);

        if read_rows > part_size {
            dbg!(current_part);
        }

        // pageid_link_map.entry(pagelink.from).or_insert_with(Vec::new).push(pagelink.title);

        row_set.insert(row);

        // pageid_link_map.insert((pagelink.from, pagelink.title));
        read_rows += 1;
    }

    bar.finish();
    (row_set, row_count)
}

#[cfg(test)]
mod tests {
    use fxhash::FxHashMap;
    use parse_mediawiki_sql::field_types::{LinkTargetId, PageTitle};
    use parse_mediawiki_sql::utils::memory_map;

    use crate::sqlite::load::load_linktarget_map;

    #[test]
    fn test_load_linktarget_map() {
        let mmap = unsafe { memory_map("tests/data/small/test-20240901-linktarget.sql").unwrap() };
        let map = load_linktarget_map(mmap);
        let mut expected = FxHashMap::default();
        expected.insert(LinkTargetId(3), PageTitle("Main_Page".to_string()));
        expected.insert(LinkTargetId(4), PageTitle("DUMMY".to_string()));

        assert_eq!(map, expected);
    }

    #[test]
    #[ignore]
    fn test_load_linktarget_map_big() {
        let mmap = unsafe { memory_map("tests/data/dewiki-20240901-linktarget.sql").unwrap() };
        let map = load_linktarget_map(mmap);
        assert!(!map.is_empty())
    }
}
