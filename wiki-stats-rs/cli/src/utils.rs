use std::process::exit;

use colored::Colorize;

pub fn print_error_and_exit(msg: impl AsRef<str>) -> ! {
    eprintln!("{} {}", "Error:".red(), msg.as_ref().red());
    exit(-1);
}