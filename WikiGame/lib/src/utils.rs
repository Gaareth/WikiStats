use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn bar_color(color: &str, length: u64) -> ProgressBar {
    let bar = indicatif::ProgressBar::new(length);
    bar.set_style(
        ProgressStyle::with_template(
            &("{spinner:.blue} {bar:40.".to_owned() + "white/" + color + "}  [{elapsed_precise}] {pos:>7}/{len:7} [{percent}%] {eta_precise} {per_sec}"),
        ).unwrap()
    );
    bar
}

pub fn default_barstyle() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7} [{percent}%] {eta_precise} {per_sec}",
    )
        .unwrap()
}


pub fn default_secondary_barstyle() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.blue} {bar:40.green/green} [{elapsed_precise}] {pos:>7}/{len:7} {eta_precise}",
    )
        .unwrap()
}

pub fn default_barstyle_unknown() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.cyan} [{elapsed_precise}] [{per_sec}] {pos:>7}/?",
    )
        .unwrap()
}

pub fn default_bar(length: u64) -> ProgressBar {
    let bar = indicatif::ProgressBar::new(length);
    bar.set_style(default_barstyle());
    bar
}

pub fn default_secondary_bar(length: u64) -> ProgressBar {
    let bar = indicatif::ProgressBar::new(length);
    bar.set_style(default_secondary_barstyle());
    bar
}

pub fn default_bar_unknown() -> ProgressBar {
    let bar = indicatif::ProgressBar::new(0);
    bar.set_style(default_barstyle_unknown());
    bar
}

pub fn download_barstyle(name: &str) -> ProgressStyle {
    let mut tempate_str = "{spinner:.blue} [{elapsed_precise}] [ETA {eta_precise}] \
    {bar:40.green/green} [{percent}%] [{total_bytes}] ({bytes_per_sec}): ".to_string();
    tempate_str.push_str(name);
    // tempate_str.push_str(" {msg}");
    ProgressStyle::with_template(&tempate_str).unwrap()
}

pub fn write_barstyle(name: &str) -> ProgressStyle {
    let mut tempate_str = "{spinner:.green} {bar:40.cyan/blue} [{elapsed_precise}] {pos:>7}/{len:7} [{percent}%] {eta_precise} {per_sec}".to_string();
    tempate_str.push_str(name);
    // tempate_str.push_str(" {msg}");
    ProgressStyle::with_template(&tempate_str).unwrap()
}


pub fn download_bar(length: u64, name: &str) -> ProgressBar {
    let bar = indicatif::ProgressBar::new(length);
    bar.set_style(download_barstyle(name));
    bar
}

pub fn spinner_bar(msg: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    let mut template = "{spinner:.cyan} {elapsed_precise}".to_string();
    template.push_str(msg);
    bar.set_style(ProgressStyle::with_template(&template).unwrap());
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

pub async fn fake_bar() {
    let b = default_bar(100);
    for i in 1..=100 {
        b.inc(1);
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}

pub async fn multi_fake_bar_async(m: &MultiProgress) {
    let b = m.add(default_bar(100));
    
    for i in 1..=100 {
        b.inc(1);
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}
