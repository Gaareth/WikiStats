use crate::args::Commands;
use crate::print_error_and_exit;
use crate::validation::validate_wiki_names;
use colored::Colorize;
use parse_mediawiki_sql::field_types::PageTitle;
use std::process::exit;
use wiki_stats::process::process_wikis_seq;
use wiki_stats::sqlite::join_db_wiki_path;
use wiki_stats::validate::post_validation;
use wiki_stats::web;

pub async fn handle_process_databases(command: Commands) {
    if let Commands::ProcessDatabases {
        path,
        wikis,
        remove_downloads,
        dump_date,
        overwrite_sql,
        validate,
        num_pages,
    } = command
    {
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

        let dump_date =
            process_wikis_seq(&wikis, basepath, dump_date, remove_downloads, overwrite_sql).await;

        if validate {
            print!("> Validating");
            for wiki in wikis {
                let wiki_prefix = &wiki[..2];
                let random_pages: Vec<PageTitle> =
                    web::get_random_wikipedia_pages(num_pages, &wiki_prefix)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|p| PageTitle(p.title))
                        .collect();
                let db_file = join_db_wiki_path(basepath.join(&dump_date).join("sqlite"), &wiki);
                let valid = post_validation(&db_file, &dump_date, &wiki_prefix, &random_pages).await;
                if !valid {
                    print_error_and_exit!("[{wiki}] Failed post validation for {db_file:?}")
                } else {
                    print!("{}", format!("Validation was successful").green())
                }
            }
        }
    } else {
        unreachable!("This function should only be called with the ProcessDatabases command");
    }
}
