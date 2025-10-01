use dirs::home_dir;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;

pub async fn setup_logging(verbose: u8) {
    let logfile_path = home_dir()
        .expect("Failed retrieving home dir of OS")
        .join("wikiStats-cli.log");

    let logfile = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(&logfile_path)
        .unwrap_or_else(|_| panic!("Failed creating? logfile at {logfile_path:?}"));

    let term_loglevel = match verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    println!("LogLevel: {term_loglevel}");

    CombinedLogger::init(vec![
        TermLogger::new(
            term_loglevel,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, Config::default(), logfile),
    ])
    .unwrap();
}