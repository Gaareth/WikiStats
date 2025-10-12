use std::{collections::HashSet, fs};

use wiki_stats::{download, stats::Stats};

use crate::{
    args::{TasksCommands, WikisArgs},
    print_error_and_exit,
    validation::validate_wiki_names,
};

pub async fn handle_tasks_commands(subcommands: TasksCommands) {
    match subcommands {
        TasksCommands::GetCompleteDumpdates { args } => {
            let WikisArgs { wikis, tables } = args;
            validate_wiki_names(&wikis)
                .await
                .unwrap_or_else(|e| print_error_and_exit!("Failed validating wiki names: {e}"));
            println!(
                "> Searching all dump dates for {wikis:?} where the tables {tables:?} are available"
            );

            for wiki in wikis {
                let dump_dates = download::get_all_available_dump_dates(&wiki, &tables).await;
                println!("Available '{wiki}' dump dates: {dump_dates:?}");
            }
        }

        TasksCommands::GetTasks { args, stats_path } => {
            let WikisArgs { wikis, tables } = args;
            validate_wiki_names(&wikis)
                .await
                .unwrap_or_else(|e| print_error_and_exit!("Failed validating wiki names: {e}"));
            if !stats_path.exists() {
                if let Err(e) = fs::create_dir_all(&stats_path) {
                    print_error_and_exit!("Failed to create stats_path directory: {e}");
                }
            }

            let mut finished_dump_dates: Vec<String> = fs::read_dir(&stats_path)
                .unwrap()
                .flatten()
                .filter_map(|entry| {
                    let file_name = entry.file_name();
                    let file_name = file_name.to_str()?;

                    if file_name.ends_with(".json") {
                        let stats: Stats = serde_json::from_str(
                            &fs::read_to_string(entry.path()).unwrap_or_else(|_| {
                                print_error_and_exit!("Failed loading stats file: {file_name:?}")
                            }),
                        )
                        .unwrap_or_else(|_| {
                            print_error_and_exit!("Failed deserializing stats file: {file_name:?}")
                        });

                        // println!()
                        // if one requested wiki is not done here, it is not finished and will need to be redone
                        if wikis.iter().any(|w| !stats.wikis.contains(w)) {
                            return None;
                        }

                        let dump_date = stats.dump_date;
                        Some(dump_date)
                    } else {
                        None
                    }
                })
                .collect();

            finished_dump_dates.sort();
            println!("Finished dumpdates: {finished_dump_dates:?}");

            let available_dump_dates =
                download::get_all_available_dump_dates_for_all_wikis(&wikis, &tables).await;

            // remove dups
            let mut available_dump_dates: Vec<_> = available_dump_dates
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            available_dump_dates.sort();
            println!("Available dumpdates: {available_dump_dates:?}");

            let mut dump_dates_todo: Vec<String> = available_dump_dates
                .into_iter()
                .filter(|x| !finished_dump_dates.contains(x))
                .collect();
            dump_dates_todo.sort();

            println!("Todo dumpdates: {dump_dates_todo:?}")
        }
    }
}
