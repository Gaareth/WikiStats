use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::{fs, thread};
use std::borrow::Borrow;
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use nipper::{Document, Selection};
use reqwest::Url;

use bzip2::Compression;
use bzip2::read::{BzEncoder, BzDecoder};

// type PageID = String;

use parse_mediawiki_sql::{
    iterate_sql_insertions,
    schemas::Page,
    schemas::PageLink,
    field_types::{PageNamespace, PageTitle, PageId},
    utils::memory_map,
};
use parse_mediawiki_sql::utils::Mmap;


fn validate_domain() {
    todo!();
}

fn parse_url(url: &str) -> reqwest::Url {
    reqwest::Url::parse(url).unwrap()
}


// fn parse_sql_tuple(string: &str) -> (PageID, String) {
//     let mut value_split = string.split(',');
//     let page_id = value_split.next().unwrap().to_string();
//     value_split.next();
//
//     let mut page_title = string.split(&format!("{}{},", page_id, 0)).last().unwrap();
//     let end_index = page_title.rfind(',').unwrap();
//     page_title = page_title.split_at(end_index).0;
//
//     (page_id, page_title.to_string())
// }

fn load_sql_part(pagelinks_sql: Mmap, part_size: i32, start_part: i32) -> HashMap<PageId, Vec<PageTitle>> {
    let bar = indicatif::ProgressBar::new(part_size as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7}").unwrap());

    let mut pageid_link_map: HashMap<PageId, Vec<PageTitle>> = HashMap::new();

    let mut row_count = 0;
    let mut read_rows = 0;
    for pagelink in iterate_sql_insertions::<PageLink>(&pagelinks_sql).into_iter() {
        row_count += 1;

        let current_part: i32 = (row_count as f32 / part_size as f32).ceil() as i32;
        if current_part < start_part {
            continue
        }

        if current_part > start_part {
            break
        }

        bar.inc(1);

        if read_rows > part_size {
            dbg!(current_part);
        }

        pageid_link_map.entry(pagelink.from).or_insert_with(Vec::new).push(pagelink.title);
        read_rows += 1;
    }

    bar.finish();
    pageid_link_map
}

fn load_pagelinks_map(path: &str) -> HashMap<PageId, Vec<PageTitle>> {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn combine() {
    // What do do:
    let mut current_page_links: (PageId, Vec<PageTitle>);
    // get first file
    for (id, titles) in load_pagelinks_map("src/pageid_links1.json") {
        // initialize current_page_links
        current_page_links = (id, titles);

        let map_other = load_pagelinks_map("src/pageid_links2.json");
        dbg!(map_other.contains_key(&id));
        drop(map_other);
    }



    // get pageID from all other links
    // combine
    // save to file
    // loop
}

fn main() {
    // let path = "/run/media/gareth/7FD71CF32A89EF6A/dev/enwiki-20221001-pages-meta-current.xml.bz2";
    let path = "src/dewiki-20221001-pagelinks.sql";
    // let mut file = File::open(path).unwrap();
    let pagelinks_sql = unsafe { memory_map(path).unwrap() };

    let mut pageid_link_map = load_sql_part(pagelinks_sql
                                            , 167081462, 1);
    //
    // dbg!(&pageid_link_map.len());
    fs::write("src/pageid_all.json",
              serde_json::to_string(&pageid_link_map).unwrap()).unwrap();
    // for (from_id, to_title) in &titles {
    //     pageid_link_map.entry(from_id).or_insert_with(Vec::new).push(to_title);
    // }
    // dbg!(pageid_link_map.get(&PageId(655087)).unwrap().len());

    // let mut content = String::new();
    // file.read_to_string(&mut content).unwrap();
    // print!("Loaded bitches");
    //
    // let pat = "INSERT INTO `pagelinks` VALUES ";
    // let start = content.find(pat).unwrap() + pat.len();
    //
    // let main_content = content.split_at(start).1;
    // let inserts = main_content.split("),(");
    // // dbg!(inserts.count());
    //
    // let bar = indicatif::ProgressBar::new(0);
    // bar.set_style(
    //     ProgressStyle::with_template(
    //         "{spinner:.green} [{elapsed_precise}] {pos}").unwrap());
    //
    // let mut c = 0;
    //
    // let mut pageid_link_map: HashMap<PageID, Vec<String>> = HashMap::new();
    //
    // for insert in inserts {
    //     c += 1;
    //     let insert = if insert.starts_with('(') {
    //         insert.chars().skip(1).collect()
    //     } else {
    //         insert.to_string()
    //     };
    //
    //     let tuple = parse_sql_tuple(&insert);
    //     pageid_link_map.entry(tuple.0).or_insert_with(Vec::new).push(tuple.1);
    //     bar.inc(1);
    // }
    // // bar.finish();
    // println!("{}", c);
    // dbg!(pageid_link_map.len());

    // let reader = BufReader::new(BzDecoder::new(file));
    // let reader = Reader::from_reader(reader);
    // let mut reader = BufReader::new(file);


    // const BUFF_SIZE: usize = usize::pow(2, 12);
    // dbg!(&BUFF_SIZE);
    // let mut buffer = [0; BUFF_SIZE];
    // let mut total_read_kb = 0;
    //
    // loop {
    //     let read = reader.read(&mut buffer).expect("Failed reading to buffer");
    //     // dbg!(&read);
    //     total_read_kb += (read / (1000));
    //
    //     if total_read_kb % 1_000_000 == 0 {
    //         dbg!(total_read_kb / 1_000_000);
    //     }
    //
    //     let mut decompressor = BzDecoder::new(buffer.as_slice());
    //     dbg!(dec)
    //
    //     if read == 0 {
    //         break;
    //     }
    // }

    // let mut contents = String::new();
    // decompressor.read_to_string(&mut contents).unwrap();
    // dbg!(contents.len());

}
