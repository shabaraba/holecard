use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hc")]
#[command(about = "Secure password manager CLI", long_about = None)]
pub struct Cli {
    #[arg(long, global = true, help = "Deck name to use")]
    pub deck: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new deck")]
    Init,

    #[command(about = "Manage hands")]
    Hand {
        #[command(subcommand)]
        subcommand: HandCommands,
    },

    #[command(about = "Manage configuration")]
    Config {
        #[command(subcommand)]
        subcommand: Option<ConfigCommands>,
    },

    #[command(about = "Read a secret value from URI")]
    Read {
        #[arg(help = "Secret URI (hc://[deck/]hand/card or op://[deck/]hand/card)")]
        uri: String,
    },

    #[command(about = "Inject secrets from template with URI references")]
    Inject {
        #[arg(help = "Template string with hc://... or op://... URIs (if --input not specified)")]
        template: Option<String>,

        #[arg(short = 'i', long, help = "Input template file (use '-' for stdin)")]
        input: Option<String>,

        #[arg(short = 'o', long, help = "Output file (default: stdout)")]
        output: Option<String>,
    },

    #[command(about = "Run command with environment variables from URIs")]
    Run {
        #[arg(long, value_parser = parse_env_var, help = "Environment variable (KEY=hc://... or KEY=op://...)")]
        env: Vec<(String, String)>,

        #[arg(last = true, help = "Command and arguments to execute")]
        command: Vec<String>,
    },

    #[command(about = "Lock the deck (clear session)")]
    Lock,

    #[command(about = "Show session status")]
    Status,

    #[command(about = "Export deck to JSON file")]
    Export {
        #[arg(help = "Output file path")]
        file: String,
    },

    #[command(about = "Import hands from JSON file")]
    Import {
        #[arg(help = "Input file path")]
        file: String,

        #[arg(long, help = "Overwrite existing hands")]
        overwrite: bool,
    },

    #[command(about = "Manage TOTP (Time-based One-Time Password)")]
    Totp {
        #[command(subcommand)]
        subcommand: TotpCommands,
    },

    #[command(about = "Manage secret providers (GitHub, Cloudflare, etc.)")]
    Provider {
        #[command(subcommand)]
        subcommand: ProviderCommands,
    },

    #[command(about = "Generate a secure password")]
    Generate {
        #[arg(short, long, help = "Password length (default: 20)")]
        length: Option<usize>,

        #[arg(short, long, help = "Generate memorable passphrase")]
        memorable: bool,

        #[arg(short, long, help = "Number of words in passphrase (default: 4)")]
        words: Option<usize>,

        #[arg(long, help = "Exclude uppercase letters")]
        no_uppercase: bool,

        #[arg(long, help = "Exclude lowercase letters")]
        no_lowercase: bool,

        #[arg(long, help = "Exclude digits")]
        no_digits: bool,

        #[arg(long, help = "Exclude symbols")]
        no_symbols: bool,

        #[arg(short, long, help = "Copy to clipboard")]
        clip: bool,
    },

    #[command(about = "Manage decks")]
    Deck {
        #[command(subcommand)]
        subcommand: DeckCommands,
    },

    #[command(about = "Manage SSH keys")]
    Ssh {
        #[command(subcommand)]
        subcommand: SshCommands,
    },

    #[command(about = "Generate shell completion script")]
    Completion {
        #[arg(help = "Shell type (bash, zsh, fish)")]
        shell: String,
    },

    #[command(name = "__complete-hands", hide = true)]
    __CompleteHands {
        #[arg(long)]
        deck: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HandCommands {
    #[command(about = "Add a new hand")]
    Add {
        #[arg(help = "Hand name")]
        name: Option<String>,

        #[arg(short, long, value_parser = parse_field, help = "Card (key=value)")]
        field: Vec<(String, String)>,

        #[arg(long, value_parser = parse_file_field, help = "Add card from file (key=path)")]
        file: Vec<(String, String)>,

        #[arg(short, long, help = "Generate random password for 'password' card")]
        generate: bool,

        #[arg(long, help = "Generated password length (default: 20)")]
        gen_length: Option<usize>,

        #[arg(short = 'm', long, help = "Generate memorable passphrase")]
        gen_memorable: bool,

        #[arg(short = 'w', long, help = "Number of words in passphrase (default: 4)")]
        gen_words: Option<usize>,

        #[arg(long, help = "Exclude uppercase from generated password")]
        gen_no_uppercase: bool,

        #[arg(long, help = "Exclude lowercase from generated password")]
        gen_no_lowercase: bool,

        #[arg(long, help = "Exclude digits from generated password")]
        gen_no_digits: bool,

        #[arg(long, help = "Exclude symbols from generated password")]
        gen_no_symbols: bool,
    },

    #[command(about = "Get a hand")]
    Get {
        #[arg(help = "Hand name")]
        name: String,

        #[arg(
            short,
            long,
            value_name = "CARD",
            help = "Copy card to clipboard (optional card name, defaults to 'password' or first card)"
        )]
        clip: Option<Option<String>>,

        #[arg(long, help = "Show TOTP code")]
        totp: bool,

        #[arg(long, help = "Show card values (requires password re-entry)")]
        show: bool,
    },

    #[command(about = "List all hands")]
    List,

    #[command(about = "Edit a hand")]
    Edit {
        #[arg(help = "Hand name")]
        name: String,

        #[arg(short, long, help = "Interactive mode")]
        interactive: bool,

        #[arg(short, long, value_parser = parse_field, help = "Add or update card (key=value)")]
        field: Vec<(String, String)>,

        #[arg(long, value_parser = parse_file_field, help = "Add or update card from file (key=path)")]
        file: Vec<(String, String)>,

        #[arg(short = 'd', long = "rm-card", help = "Remove card by key")]
        rm_card: Vec<String>,
    },

    #[command(about = "Remove a hand")]
    Remove {
        #[arg(help = "Hand name")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Set deck file path")]
    DeckPath {
        #[arg(help = "New deck file path")]
        path: String,
    },

    #[command(about = "Set session timeout (minutes)")]
    SessionTimeout {
        #[arg(help = "Timeout in minutes")]
        minutes: u64,
    },
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    #[command(about = "List all configured providers")]
    List,

    #[command(about = "Add a new provider configuration")]
    Add {
        #[command(subcommand)]
        provider: ProviderAddCommands,
    },

    #[command(about = "Edit provider authentication credentials")]
    Edit {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,

        #[command(subcommand)]
        provider: ProviderAddCommands,
    },

    #[command(about = "Remove a provider configuration")]
    Remove {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,
    },

    #[command(about = "Manage secrets in provider")]
    Secrets {
        #[command(subcommand)]
        subcommand: ProviderSecretsCommands,
    },
}

#[derive(Subcommand)]
pub enum ProviderSecretsCommands {
    #[command(about = "List secrets in a provider")]
    List {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,
    },

    #[command(about = "Add secret(s) to provider")]
    Add {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,

        #[arg(help = "Hand name or hand.card (e.g., myapp.db_url)")]
        card_field: String,

        #[arg(long, help = "Override secret name in provider")]
        as_name: Option<String>,

        #[arg(long, help = "Push all cards from hand as separate secrets")]
        expand: bool,
    },

    #[command(about = "Remove a secret from provider")]
    Remove {
        #[arg(help = "Provider type")]
        provider_type: String,

        #[arg(help = "Provider ID")]
        provider_id: String,

        #[arg(help = "Secret name to delete")]
        secret_name: String,
    },
}

#[derive(Subcommand)]
pub enum ProviderAddCommands {
    #[command(about = "Add GitHub Actions Secrets provider")]
    Github {
        #[arg(help = "Provider ID (e.g., my-repo)")]
        provider_id: String,

        #[arg(long, help = "GitHub repository (owner/repo)")]
        repo: String,

        #[arg(long, help = "GitHub Personal Access Token")]
        token: String,
    },

    #[command(about = "Add Cloudflare Workers Secrets provider")]
    Cloudflare {
        #[arg(help = "Provider ID (e.g., my-worker)")]
        provider_id: String,

        #[arg(long, help = "Cloudflare Account ID")]
        account_id: String,

        #[arg(long, help = "Worker name")]
        worker_name: String,

        #[arg(long, help = "Cloudflare API Token")]
        token: String,
    },
}

#[derive(Subcommand)]
pub enum TotpCommands {
    #[command(about = "Add TOTP secret to a hand")]
    Add {
        #[arg(help = "Hand name")]
        card: String,

        #[arg(help = "TOTP secret (base32 encoded)")]
        secret: String,
    },

    #[command(about = "Get TOTP code (displays and copies to clipboard)")]
    Get {
        #[arg(help = "Hand name")]
        card: String,
    },

    #[command(about = "Remove TOTP secret from a hand")]
    Rm {
        #[arg(help = "Hand name")]
        card: String,
    },
}

#[derive(Subcommand)]
pub enum DeckCommands {
    #[command(about = "List all decks")]
    List,

    #[command(about = "Create a new deck")]
    Create {
        #[arg(help = "Deck name")]
        name: String,
    },

    #[command(about = "Delete a deck")]
    Delete {
        #[arg(help = "Deck name")]
        name: String,

        #[arg(long, help = "Skip confirmation")]
        force: bool,
    },

    #[command(about = "Set active deck")]
    Use {
        #[arg(help = "Deck name")]
        name: String,
    },

    #[command(about = "Move hand to another deck")]
    Move {
        #[arg(help = "Hand name")]
        card: String,

        #[arg(help = "Target deck name")]
        to_hand: String,
    },

    #[command(about = "Copy hand to another deck")]
    Copy {
        #[arg(help = "Hand name")]
        card: String,

        #[arg(help = "Target deck name")]
        to_hand: String,
    },

    #[command(about = "Change master password")]
    Passwd,
}

#[derive(Subcommand)]
pub enum SshCommands {
    #[command(about = "Add SSH connection hand")]
    Add {
        #[arg(help = "Hand name")]
        name: String,

        #[arg(
            long,
            help = "SSH config alias (mutually exclusive with username/hostname)"
        )]
        alias: Option<String>,

        #[arg(long, help = "SSH username")]
        username: Option<String>,

        #[arg(long, help = "SSH hostname")]
        hostname: Option<String>,

        #[arg(long, help = "SSH password (mutually exclusive with private-key)")]
        password: Option<String>,

        #[arg(
            long,
            help = "Path to private key file (mutually exclusive with password)"
        )]
        private_key: Option<String>,

        #[arg(long, help = "Path to public key file (optional)")]
        public_key: Option<String>,

        #[arg(long, help = "Passphrase for private key (optional)")]
        passphrase: Option<String>,
    },

    #[command(about = "Load SSH key into ssh-agent")]
    Load {
        #[arg(help = "Hand name containing SSH key")]
        name: String,

        #[arg(long, help = "Lifetime in seconds (0 = forever)")]
        lifetime: Option<u32>,
    },

    #[command(about = "Remove SSH key from ssh-agent")]
    Unload {
        #[arg(help = "Hand name or public key fingerprint")]
        name: String,
    },

    #[command(about = "List loaded SSH keys in ssh-agent")]
    List,

    #[command(about = "Connect to SSH host (auto-loads key from hand)")]
    Connect {
        #[arg(help = "Hand name or alias (e.g., git@github.com)")]
        target: String,

        #[arg(last = true, help = "Additional SSH arguments")]
        ssh_args: Vec<String>,
    },
}

fn parse_field(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid field format: '{}'. Expected key=value", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn parse_file_field(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid field format: '{}'. Expected key=path", s));
    }

    let key = parts[0].to_string();
    let path = parts[1];

    let expanded_path = if path.starts_with('~') {
        path.replacen(
            '~',
            &std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
            1,
        )
    } else {
        path.to_string()
    };

    let content = std::fs::read_to_string(&expanded_path)
        .map_err(|e| format!("Failed to read file '{}': {}", expanded_path, e))?;

    Ok((key, content))
}

fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid env var format: '{}'. Expected KEY=value",
            s
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
