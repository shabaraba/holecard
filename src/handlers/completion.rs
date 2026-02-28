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
# Dynamic hand name completion
_hc_complete_hand_names() {{
    local entries
    entries=$(hc __complete-hands 2>/dev/null)
    COMPREPLY+=($(compgen -W "$entries" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

# Override completion for commands that take hand names
_hc_hand_get() {{
    case "${{prev}}" in
        get|edit|remove|rm)
            _hc_complete_hand_names
            return 0
            ;;
    esac
}}

complete -F _hc_hand_get hc
"#
    );
}

fn patch_zsh_completion(completion: String) -> String {
    let helper_function = r#"
# Dynamic hand name completion helper
_hc_hand_names() {
    local -a entries
    entries=(${(f)"$(hc __complete-hands 2>/dev/null)"})
    _describe 'hand names' entries
}

"#;

    let completion = completion
        .replace(
            "':name -- Hand name:_default'",
            "':name -- Hand name: _hc_hand_names'",
        )
        .replace(
            "':card -- Hand name:_default'",
            "':card -- Hand name: _hc_hand_names'",
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
# Dynamic hand name completion for fish
function __hc_hand_names
    hc __complete-hands 2>/dev/null
end

complete -c hc -n "__fish_seen_subcommand_from hand; and __fish_seen_subcommand_from get" -a "(__hc_hand_names)"
complete -c hc -n "__fish_seen_subcommand_from hand; and __fish_seen_subcommand_from edit" -a "(__hc_hand_names)"
complete -c hc -n "__fish_seen_subcommand_from hand; and __fish_seen_subcommand_from remove" -a "(__hc_hand_names)"
"#
    );
}

pub fn handle_complete_hands(deck_name: Option<&str>, config_dir: &Path) -> Result<()> {
    let deck_name = match deck_name {
        Some(name) => name.to_string(),
        None => {
            let registry = DeckRegistry::load(config_dir)?;
            registry.get_active_deck()?.name
        }
    };

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &deck_name, config.session_timeout_minutes);

    let hand_names = session.load_card_names().unwrap_or_default();

    for name in hand_names {
        println!("{}", name);
    }

    Ok(())
}
