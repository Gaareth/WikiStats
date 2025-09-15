use std::time::Duration;

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct ProgressBarBuilder {
    spinner_color: String,
    bar_color_fg: String,
    bar_color_bg: String,
    name: String,
    length: u64,
}

impl ProgressBarBuilder {
    pub fn new() -> Self {
        Self {
            spinner_color: "cyan".to_string(),
            bar_color_fg: "cyan".to_string(),
            bar_color_bg: "blue".to_string(),
            name: "".to_string(),
            length: 100,
        }
    }

    pub fn with_spinner_color(mut self, color: &str) -> Self {
        self.spinner_color = color.to_string();
        self
    }

    pub fn with_bar_color_fg(mut self, color: &str) -> Self {
        self.bar_color_fg = color.to_string();
        self
    }

    pub fn with_bar_color_bg(mut self, color: &str) -> Self {
        self.bar_color_bg = color.to_string();
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_length(mut self, length: u64) -> Self {
        self.length = length;
        self
    }

    pub fn build(self) -> ProgressBar {
        let template = format!(
            "{{spinner:.{}}} {{prefix:.bold.{}}} [{{elapsed_precise}}] {{bar:40.{}/{}}} {{pos:>7}}/{{len:7}} [{{percent}}%] | ETA: {{eta_precise}} {{per_sec}}",
            self.spinner_color, self.spinner_color, self.bar_color_fg, self.bar_color_bg
        );
        let bar = ProgressBar::new(self.length);
        bar.set_style(ProgressStyle::with_template(&template).unwrap());
        bar.set_prefix(self.name);
        bar
    }
}

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
