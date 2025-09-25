use crate::args::Commands;
use crate::validation::validate_wiki_names;
use colored::Colorize;
use std::process::exit;
use wiki_stats::process::process_wikis_seq;

pub async fn handle_process_databases(command: Commands) {
    if let Commands::ProcessDatabases {
        path,
        wikis,
        remove_downloads,
        dump_date,
        overwrite_sql,
    } = command {
        validate_wiki_names(&wikis)
            .await
            .unwrap_or_else(|e| panic!("{}: {e}", "Failed validating wiki names".red()));
        
        let basepath = &path;
        
        if !basepath.exists() {
            eprintln!(
                "{}: The specified path does not exist: {}",
                "Error".red(),
                basepath.display().to_string().underline()
            );
            exit(-1);
        }
        
        process_wikis_seq(
            &wikis,
            basepath,
            dump_date,
            remove_downloads,
            overwrite_sql,
        )
        .await;
    } else {
        unreachable!("This function should only be called with the ProcessDatabases command");
    }
}