use crate::args::Commands;

pub mod db;
pub mod debug;
pub mod stats;
pub mod tasks;

pub async fn handle_command(command: Commands) {
    match command {
        Commands::ProcessDatabases { .. } => db::handle_process_databases(command).await,
        Commands::Stats { subcommands } => stats::handle_stats(subcommands).await,
        Commands::Debug { subcommands } => debug::handle_debug_commands(subcommands).await,
        Commands::Tasks { subcommands } => tasks::handle_tasks_commands(subcommands).await,
    }
}
