mod cli;
mod config;
mod deck_context;
mod domain;
mod handlers;
mod infrastructure;
mod multi_deck_context;

use anyhow::Result;
use clap::Parser;
use cli::commands::{CardCommands, Cli, Commands};
use config::get_config_dir;
use infrastructure::KeyringManager;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = get_config_dir()?;
    let keyring = KeyringManager::new(config_dir.clone());
    let deck_name = cli.hand.as_deref();

    match cli.command {
        Commands::Init => handlers::deck::handle_init(&keyring, &config_dir),
        Commands::Card { subcommand } => match subcommand {
            CardCommands::Add {
                name,
                field,
                file,
                generate,
                gen_length,
                gen_memorable,
                gen_words,
                gen_no_uppercase,
                gen_no_lowercase,
                gen_no_digits,
                gen_no_symbols,
            } => handlers::deck::handle_add(
                name,
                field,
                file,
                generate,
                gen_length,
                gen_memorable,
                gen_words,
                gen_no_uppercase,
                gen_no_lowercase,
                gen_no_digits,
                gen_no_symbols,
                deck_name,
                &keyring,
                &config_dir,
            ),
            CardCommands::Get {
                name,
                clip,
                totp,
                show,
            } => handlers::deck::handle_get(
                &name,
                clip,
                totp,
                show,
                deck_name,
                &keyring,
                &config_dir,
            ),
            CardCommands::List => handlers::deck::handle_list(deck_name, &keyring, &config_dir),
            CardCommands::Edit {
                name,
                interactive,
                field,
                file,
                rm_field,
            } => {
                if interactive {
                    handlers::deck::handle_edit_interactive(&name, deck_name, &keyring, &config_dir)
                } else {
                    handlers::deck::handle_edit(
                        &name,
                        field,
                        file,
                        rm_field,
                        deck_name,
                        &keyring,
                        &config_dir,
                    )
                }
            }
            CardCommands::Remove { name } => {
                handlers::deck::handle_rm(&name, deck_name, &keyring, &config_dir)
            }
        },
        Commands::Config { subcommand } => handlers::config::handle_config(subcommand, &config_dir),
        Commands::Read { uri } => {
            handlers::read::handle_read(&uri, deck_name, &keyring, &config_dir)
        }
        Commands::Inject {
            template,
            input,
            output,
        } => handlers::inject::handle_inject(
            template,
            input,
            output,
            deck_name,
            &keyring,
            &config_dir,
        ),
        Commands::Run { env, command } => {
            handlers::run::handle_run(env, &command, deck_name, &keyring, &config_dir)
        }
        Commands::Lock => handlers::session::handle_lock(&config_dir),
        Commands::Status => handlers::session::handle_status(&config_dir),
        Commands::Export { file } => {
            handlers::transfer::handle_export(&file, deck_name, &keyring, &config_dir)
        }
        Commands::Import { file, overwrite } => {
            handlers::transfer::handle_import(&file, overwrite, deck_name, &keyring, &config_dir)
        }
        Commands::Totp { subcommand } => {
            handlers::totp::handle_totp(subcommand, deck_name, &keyring, &config_dir)
        }
        Commands::Provider { subcommand } => {
            let ctx = multi_deck_context::MultiDeckContext::load(deck_name, &keyring, &config_dir)?;
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
        Commands::Hand { subcommand } => {
            handlers::deck_management::handle_deck(subcommand, deck_name, &keyring, &config_dir)
        }
        Commands::Ssh { subcommand } => {
            handlers::ssh::handle_ssh(subcommand, deck_name, &keyring, &config_dir)
        }
        Commands::Completion { shell } => handlers::completion::handle_completion(&shell),
        Commands::__CompleteCards { hand } => {
            handlers::completion::handle_complete_cards(hand.as_deref(), &config_dir)
        }
    }
}
