use clap::Parser;

mod testdata;
mod logging;
mod args;
mod commands;
mod validation;
mod utils;

use args::Cli;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed reading env variables");
    let cli = Cli::parse();

    logging::setup_logging(cli.verbose).await;
    commands::handle_command(cli.command).await;
}

#[cfg(test)]
mod cli_test {
    use crate::{validation::validate_wiki_names, Cli};

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[tokio::test]
    async fn test_validate_wikis() {
        assert!(validate_wiki_names(&["enwiki"]).await.is_ok());
        assert!(validate_wiki_names(&["enwiki", "jawiki"]).await.is_ok());

        assert!(validate_wiki_names(&["DOESNOTEXIST"]).await.is_err());
        assert!(validate_wiki_names(&["enwiki", "DOESNOTEXIST"])
            .await
            .is_err());
        assert!(validate_wiki_names(Vec::<String>::new().as_slice())
            .await
            .is_err());
    }

    // #[tokio::test]
    // async fn test_validate_sqlite_files() {
    //     assert!(validate_sqlite_files(&["jawiki"]).await.is_ok());
    //     assert!(validate_wiki_names(&["jawiki"]).await.is_err());
    // }
}


