

fn recursive_check(link: Url, end_url: &Url, _max_tries: i32, rec_count: i32) {
    // println!("{link}");

    if rec_count + 1 > _max_tries {
        return;
    }

    if &link == end_url {
        println!("Found end url in {}", rec_count + 1);
        return;
    }

    let links = get_links(link.clone(), link);


    for link in links {
        recursive_check(link, end_url, _max_tries, rec_count + 1);
    }
}

fn scrape(start_url: reqwest::Url, end_url: reqwest::Url, max_tries: i32) {
    let links = get_links(start_url.clone(), start_url);

    dbg!(links.len());

    let bar = ProgressBar::new(links.len() as u64);
    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}").unwrap());
    // bar.tick();

    for link in links {
        // println!("{link}");
        thread::spawn(move || {
            recursive_check(link, &end_url.clone(), max_tries, 1);
            // bar.inc(1);
        });
    }

    bar.finish();
}

fn get_link(domain: &str, link_part: &str) -> Url {
    match Url::parse(link_part) {
        Ok(url) => url,
        Err(_) => Url::parse(&format!("https://{domain}{link_part}")).unwrap(),
    }
}

fn filter_link_selection(start_url: Url, sel: Selection) -> Option<Url> {
    if let Some(href) =  sel.attr("href") {
        if href.starts_with('#') {
            return None
        }

        let link = get_link(start_url.host_str().unwrap(), &href);
        let links_string = link.to_string();

        if !links_string.contains("wikipedia.org/wiki/") ||
            links_string.contains("Wikipedia:") {
            return None
        }

        // dbg!(&links_string);
        return Some(link)
    }
    None
}

fn get_links(start_url: Url, url: Url) -> Vec<Url> {
    let request = reqwest::blocking::get(url).unwrap();
    let content = request.text().unwrap();
    let document = Document::from(&content);
    let main_content = document.select("#mw-content-text").first();

    main_content.select("a").iter()
        .filter_map(|sel| filter_link_selection(start_url.clone(), sel)).collect()
}
