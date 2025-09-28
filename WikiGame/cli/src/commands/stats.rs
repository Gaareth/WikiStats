use wiki_stats::sqlite::get_all_database_files;

use crate::{
    args::{Commands, SampleOptions, StatsArgs, StatsCommands, WikiSizesArgs},
    utils::print_error_and_exit,
    validation::{validate_sqlite_files, validate_wiki_names},
};

async fn handle_add_sample_stats(args: StatsArgs, sample_args: SampleOptions) {
    let StatsArgs {
        output_path,
        db_path,
        wikis,
        all_wikis,
    } = args;

    let wikis = if all_wikis {
        &get_all_database_files(&db_path).unwrap_or_else(|e| {
            print_error_and_exit(format!("Failed fetching all wikis from db path: {e}"));
        })
    } else {
        &wikis
    };

    println!("Wikis: {wikis:?}");

    validate_wiki_names(wikis)
        .await
        .unwrap_or_else(|e| print_error_and_exit(format!("Failed validating wiki names: {e}")));
    validate_sqlite_files(&db_path, wikis)
        .await
        .unwrap_or_else(|e| {
            print_error_and_exit(format!("Failed validating wiki sqlite files: {e}"))
        });

    let SampleOptions {
        sample_size,
        threads,
        cache_size,
        overwrite,
    } = sample_args;
    println!("> Creating sample bfs stats..");

    wiki_stats::stats::add_sample_bfs_stats(
        &output_path,
        db_path,
        wikis.clone(),
        sample_size,
        threads,
        cache_size,
        overwrite,
    )
    .await;
}

async fn handle_generate_stats(
    args: StatsArgs,
    add_sample: bool,
    add_web_wiki_sizes: bool,
    sample_args: SampleOptions,
) {
    let StatsArgs {
        output_path,
        db_path,
        wikis,
        all_wikis,
    } = args;

    let wikis = if all_wikis {
        &get_all_database_files(&db_path).unwrap_or_else(|e| {
            print_error_and_exit(format!("Failed fetching all wikis from db path: {e}"));
        })
    } else {
        &wikis
    };
    println!("Wikis: {wikis:?}");

    validate_wiki_names(wikis)
        .await
        .unwrap_or_else(|e| print_error_and_exit(format!("Failed validating wiki names: {e}")));
    validate_sqlite_files(&db_path, wikis)
        .await
        .unwrap_or_else(|e| {
            print_error_and_exit(format!("Failed validating wiki sqlite files: {e}"))
        });

    let base_path = db_path
        .clone()
        .parent()
        .unwrap_or_else(|| {
            print_error_and_exit("Failed extracting base path from db path");
        })
        .to_path_buf();

    let dump_date = base_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| {
            print_error_and_exit(
                "Failed extracting dumpdate from path. Please provide using --dump-date",
            )
        });
    println!("Assuming dump_date (from path): {dump_date}");

    println!(
        "Creating stats at {:?} using db files from: {:?}",
        &output_path, &db_path
    );
    wiki_stats::stats::create_stats(&output_path, wikis.clone(), &db_path, dump_date).await;

    if add_sample {
        let SampleOptions {
            sample_size,
            threads,
            cache_size,
            overwrite,
        } = sample_args;
        println!("Creating sample bfs stats..");
        wiki_stats::stats::add_sample_bfs_stats(
            &output_path,
            db_path,
            wikis.clone(),
            sample_size,
            threads,
            cache_size,
            overwrite,
        )
        .await;
    }

    if add_web_wiki_sizes {
        println!("Assuming basepath (from path): {base_path:?}");
        println!("Adding web wiki sizes to stats at {output_path:?} using db files from: {base_path:?} for dump date {dump_date:?}");
        wiki_stats::stats::add_web_wiki_sizes(&output_path, Some(dump_date.to_string())).await;
    }
}

pub async fn handle_stats(subcommands: StatsCommands) {
    match subcommands {
        StatsCommands::AddSampleStats { args, sample_args } => {
            handle_add_sample_stats(args, sample_args).await;
        }

        StatsCommands::AddWebWikiSizes { args, output_path } => {
            let WikiSizesArgs {
                base_path,
                dump_date,
            } = args;
            println!("Adding web wiki sizes to stats at {output_path:?} using db files from: {base_path:?} for dump date {dump_date:?}");
            wiki_stats::stats::add_web_wiki_sizes(&output_path, dump_date).await;
        }

        StatsCommands::Generate {
            args,
            add_sample,
            sample_args,
            add_web_wiki_sizes,
        } => {
            handle_generate_stats(args, add_sample, add_web_wiki_sizes, sample_args).await;
        }
    }
}
