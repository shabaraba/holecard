mod cli;
mod config;
mod context;
mod domain;
mod handlers;
mod infrastructure;
mod multi_vault_context;

use anyhow::Result;
use clap::Parser;
use cli::commands::{Cli, Commands, EntryCommands};
use config::get_config_dir;
use infrastructure::KeyringManager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = get_config_dir()?;
    let keyring = KeyringManager::new(config_dir.clone());
    let vault_name = cli.vault.as_deref();

    match cli.command {
        Commands::Init => handlers::vault::handle_init(&keyring, &config_dir),
        Commands::Entry { subcommand } => match subcommand {
            EntryCommands::Add {
                name,
                field,
                generate,
                gen_length,
                gen_memorable,
                gen_words,
                gen_no_uppercase,
                gen_no_lowercase,
                gen_no_digits,
                gen_no_symbols,
            } => handlers::vault::handle_add(
                name,
                field,
                generate,
                gen_length,
                gen_memorable,
                gen_words,
                gen_no_uppercase,
                gen_no_lowercase,
                gen_no_digits,
                gen_no_symbols,
                vault_name,
                &keyring,
                &config_dir,
            ),
            EntryCommands::Get {
                name,
                clip,
                totp,
                show,
            } => handlers::vault::handle_get(
                &name,
                clip,
                totp,
                show,
                vault_name,
                &keyring,
                &config_dir,
            ),
            EntryCommands::List => handlers::vault::handle_list(vault_name, &keyring, &config_dir),
            EntryCommands::Edit {
                name,
                interactive,
                field,
                rm_field,
            } => {
                if interactive {
                    handlers::vault::handle_edit_interactive(
                        &name,
                        vault_name,
                        &keyring,
                        &config_dir,
                    )
                } else {
                    handlers::vault::handle_edit(
                        &name,
                        field,
                        rm_field,
                        vault_name,
                        &keyring,
                        &config_dir,
                    )
                }
            }
            EntryCommands::Remove { name } => {
                handlers::vault::handle_rm(&name, vault_name, &keyring, &config_dir)
            }
        },
        Commands::Config { subcommand } => handlers::config::handle_config(subcommand, &config_dir),
        Commands::Inject { entry, template } => {
            handlers::template::handle_inject(&entry, &template, vault_name, &keyring, &config_dir)
        }
        Commands::Run { entry, command } => {
            handlers::template::handle_run(&entry, &command, vault_name, &keyring, &config_dir)
        }
        Commands::Lock => handlers::session::handle_lock(&config_dir),
        Commands::Status => handlers::session::handle_status(&config_dir),
        Commands::Export { file } => {
            handlers::transfer::handle_export(&file, vault_name, &keyring, &config_dir)
        }
        Commands::Import { file, overwrite } => {
            handlers::transfer::handle_import(&file, overwrite, vault_name, &keyring, &config_dir)
        }
        Commands::Totp { subcommand } => {
            handlers::totp::handle_totp(subcommand, vault_name, &keyring, &config_dir)
        }
        Commands::Provider { subcommand } => {
            let ctx =
                multi_vault_context::MultiVaultContext::load(vault_name, &keyring, &config_dir)?;
            handlers::provider::handle_provider(&ctx.inner, &subcommand)
        }
        Commands::Generate {
            length,
            memorable,
            words,
            no_uppercase,
            no_lowercase,
            no_digits,
            no_symbols,
            clip,
        } => handlers::password::handle_generate(
            length,
            memorable,
            words,
            no_uppercase,
            no_lowercase,
            no_digits,
            no_symbols,
            clip,
        ),
        Commands::Vault { subcommand } => {
            handlers::vault_management::handle_vault(subcommand, &keyring, &config_dir)
        }
        Commands::Ssh { subcommand } => {
            handlers::ssh::handle_ssh(subcommand, vault_name, &keyring, &config_dir)
        }
    }
}
