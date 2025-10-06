// use std::process::exit;

// use colored::Colorize;

// pub fn print_error_and_exit(msg: impl AsRef<str>) -> ! {
//     eprintln!("{} {}", "Error:".red(), msg.as_ref().red());
//     exit(-1);
// }

#[macro_export]
macro_rules! print_error_and_exit {
    ($($arg:tt)*) => {{
        use std::process::exit;
        use colored::Colorize;

        eprintln!("{} {}", "Error:".red(), format!($($arg)*).red());
        exit(-1);
    }};
}
