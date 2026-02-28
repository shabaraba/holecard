use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, shells};
use std::io;
use std::path::Path;

use crate::cli::commands::Cli;
use crate::config::Config;
use crate::infrastructure::{DeckRegistry, SessionManager};

pub fn handle_completion(shell: &str) -> Result<()> {
    let mut cmd = Cli::command();

    match shell.to_lowercase().as_str() {
        "bash" => {
            generate(shells::Bash, &mut cmd, "hc", &mut io::stdout());
            print_bash_dynamic_completion();
            println!();
            println!("# To enable completion, add the following to your shell config:");
            println!("#   eval \"$(hc completion bash)\"");
            println!("# Or add to ~/.bashrc:");
            println!("#   echo 'eval \"$(hc completion bash)\"' >> ~/.bashrc");
        }
        "zsh" => {
            let mut buf = Vec::new();
            generate(shells::Zsh, &mut cmd, "hc", &mut buf);
            let mut completion = String::from_utf8(buf).unwrap();
            completion = patch_zsh_completion(completion);

            print!("{}", completion);
            println!();
            println!("# To enable completion, add the following to your shell config:");
            println!("#   eval \"$(hc completion zsh)\"");
            println!("# Or add to ~/.zshrc:");
            println!("#   echo 'eval \"$(hc completion zsh)\"' >> ~/.zshrc");
        }
        "fish" => {
            generate(shells::Fish, &mut cmd, "hc", &mut io::stdout());
            print_fish_dynamic_completion();
            println!();
            println!("# To enable completion, add the following to your shell config:");
            println!("#   hc completion fish | source");
            println!("# Or save to ~/.config/fish/completions/hc.fish:");
            println!("#   hc completion fish > ~/.config/fish/completions/hc.fish");
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported shell: {}. Supported shells: bash, zsh, fish",
                shell
            ));
        }
    }

    Ok(())
}

fn print_bash_dynamic_completion() {
    println!(
        r#"
# Dynamic card name completion
_hc_complete_card_names() {{
    local entries
    entries=$(hc __complete-cards 2>/dev/null)
    COMPREPLY+=($(compgen -W "$entries" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

# Override completion for commands that take card names
_hc_card_get() {{
    case "${{prev}}" in
        get|edit|remove|rm)
            _hc_complete_card_names
            return 0
            ;;
    esac
}}

complete -F _hc_card_get hc
"#
    );
}

fn patch_zsh_completion(completion: String) -> String {
    let helper_function = r#"
# Dynamic card name completion helper
_hc_card_names() {
    local -a entries
    entries=(${(f)"$(hc __complete-cards 2>/dev/null)"})
    _describe 'card names' entries
}

"#;

    let completion = completion
        .replace(
            "':name -- Card name:_default'",
            "':name -- Card name: _hc_card_names'",
        )
        .replace(
            "':card -- Card name:_default'",
            "':card -- Card name: _hc_card_names'",
        );

    let completion = completion.replace(
        "autoload -U is-at-least\n",
        &format!("autoload -U is-at-least\n{}", helper_function),
    );

    completion
}

fn print_fish_dynamic_completion() {
    println!(
        r#"
# Dynamic card name completion for fish
function __hc_card_names
    hc __complete-cards 2>/dev/null
end

complete -c hc -n "__fish_seen_subcommand_from card; and __fish_seen_subcommand_from get" -a "(__hc_card_names)"
complete -c hc -n "__fish_seen_subcommand_from card; and __fish_seen_subcommand_from edit" -a "(__hc_card_names)"
complete -c hc -n "__fish_seen_subcommand_from card; and __fish_seen_subcommand_from remove" -a "(__hc_card_names)"
"#
    );
}

pub fn handle_complete_cards(deck_name: Option<&str>, config_dir: &Path) -> Result<()> {
    let deck_name = match deck_name {
        Some(name) => name.to_string(),
        None => {
            let registry = DeckRegistry::load(config_dir)?;
            registry.get_active_deck()?.name
        }
    };

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &deck_name, config.session_timeout_minutes);

    let card_names = session.load_card_names().unwrap_or_default();

    for name in card_names {
        println!("{}", name);
    }

    Ok(())
}
