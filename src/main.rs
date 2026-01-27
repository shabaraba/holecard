mod cli;
mod config;
mod context;
mod domain;
mod handlers;
mod infrastructure;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::get_config_dir;
use infrastructure::KeyringManager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = get_config_dir()?;
    let keyring = KeyringManager::new(config_dir.clone());

    match cli.command {
        Commands::Init => handlers::vault::handle_init(&keyring, &config_dir),
        Commands::Add { name, field } => {
            handlers::vault::handle_add(name, field, &keyring, &config_dir)
        }
        Commands::Get { name, clip, totp } => {
            handlers::vault::handle_get(&name, clip, totp, &keyring, &config_dir)
        }
        Commands::List => handlers::vault::handle_list(&keyring, &config_dir),
        Commands::Edit { name } => handlers::vault::handle_edit(&name, &keyring, &config_dir),
        Commands::Rm { name } => handlers::vault::handle_rm(&name, &keyring, &config_dir),
        Commands::Config { subcommand } => handlers::config::handle_config(subcommand, &config_dir),
        Commands::Inject { entry, template } => {
            handlers::template::handle_inject(&entry, &template, &keyring, &config_dir)
        }
        Commands::Run { entry, command } => {
            handlers::template::handle_run(&entry, &command, &keyring, &config_dir)
        }
        Commands::Lock => handlers::session::handle_lock(&config_dir),
        Commands::Status => handlers::session::handle_status(&config_dir),
        Commands::Export { file } => {
            handlers::transfer::handle_export(&file, &keyring, &config_dir)
        }
        Commands::Import { file, overwrite } => {
            handlers::transfer::handle_import(&file, overwrite, &keyring, &config_dir)
        }
        Commands::Totp { subcommand } => {
            handlers::totp::handle_totp(subcommand, &keyring, &config_dir)
        }
    }
}
