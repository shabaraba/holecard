use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, shells};
use std::io;
use std::path::Path;

use crate::cli::commands::Cli;
use crate::config::Config;
use crate::infrastructure::{SessionManager, VaultRegistry};

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
# Dynamic entry name completion
_hc_complete_entry_names() {{
    local entries
    entries=$(hc __complete-entries 2>/dev/null)
    COMPREPLY+=($(compgen -W "$entries" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

# Override completion for commands that take entry names
_hc_entry_get() {{
    case "${{prev}}" in
        get|edit|remove|rm)
            _hc_complete_entry_names
            return 0
            ;;
    esac
}}

complete -F _hc_entry_get hc
"#
    );
}

fn patch_zsh_completion(completion: String) -> String {
    let helper_function = r#"
# Dynamic entry name completion helper
_hc_entry_names() {
    local -a entries
    entries=(${(f)"$(hc __complete-entries 2>/dev/null)"})
    _describe 'entry names' entries
}

"#;

    let completion = completion
        .replace(
            "':name -- Entry name:_default'",
            "':name -- Entry name: _hc_entry_names'",
        )
        .replace(
            "':entry -- Entry name:_default'",
            "':entry -- Entry name: _hc_entry_names'",
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
# Dynamic entry name completion for fish
function __hc_entry_names
    hc __complete-entries 2>/dev/null
end

complete -c hc -n "__fish_seen_subcommand_from entry; and __fish_seen_subcommand_from get" -a "(__hc_entry_names)"
complete -c hc -n "__fish_seen_subcommand_from entry; and __fish_seen_subcommand_from edit" -a "(__hc_entry_names)"
complete -c hc -n "__fish_seen_subcommand_from entry; and __fish_seen_subcommand_from remove" -a "(__hc_entry_names)"
"#
    );
}

pub fn handle_complete_entries(vault_name: Option<&str>, config_dir: &Path) -> Result<()> {
    let vault_name = match vault_name {
        Some(name) => name.to_string(),
        None => {
            let registry = VaultRegistry::load(config_dir)?;
            registry.get_active_vault()?.name
        }
    };

    let config = Config::load(config_dir)?;
    let session = SessionManager::new(config_dir, &vault_name, config.session_timeout_minutes);

    let entry_names = session.load_entry_names().unwrap_or_default();

    for name in entry_names {
        println!("{}", name);
    }

    Ok(())
}
