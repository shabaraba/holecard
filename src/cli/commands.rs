use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hc")]
#[command(about = "Secure password manager CLI", long_about = None)]
pub struct Cli {
    #[arg(long, global = true, help = "Vault name to use")]
    pub vault: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new vault")]
    Init,

    #[command(about = "Manage vault entries")]
    Entry {
        #[command(subcommand)]
        subcommand: EntryCommands,
    },

    #[command(about = "Manage configuration")]
    Config {
        #[command(subcommand)]
        subcommand: Option<ConfigCommands>,
    },

    #[command(about = "Read a secret value from URI")]
    Read {
        #[arg(help = "Secret URI (hc://[vault/]item/field or op://[vault/]item/field)")]
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

    #[command(about = "Lock the vault (clear session)")]
    Lock,

    #[command(about = "Show session status")]
    Status,

    #[command(about = "Export vault to JSON file")]
    Export {
        #[arg(help = "Output file path")]
        file: String,
    },

    #[command(about = "Import entries from JSON file")]
    Import {
        #[arg(help = "Input file path")]
        file: String,

        #[arg(long, help = "Overwrite existing entries")]
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

    #[command(about = "Manage vaults")]
    Vault {
        #[command(subcommand)]
        subcommand: VaultCommands,
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

    #[command(name = "__complete-entries", hide = true)]
    __CompleteEntries {
        #[arg(long)]
        vault: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EntryCommands {
    #[command(about = "Add a new entry")]
    Add {
        #[arg(help = "Entry name")]
        name: Option<String>,

        #[arg(short, long, value_parser = parse_field, help = "Custom field (key=value)")]
        field: Vec<(String, String)>,

        #[arg(long, value_parser = parse_file_field, help = "Add field from file (key=path)")]
        file: Vec<(String, String)>,

        #[arg(short, long, help = "Generate random password for 'password' field")]
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

    #[command(about = "Get an entry")]
    Get {
        #[arg(help = "Entry name")]
        name: String,

        #[arg(
            short,
            long,
            value_name = "FIELD",
            help = "Copy field to clipboard (optional field name, defaults to 'password' or first field)"
        )]
        clip: Option<Option<String>>,

        #[arg(long, help = "Show TOTP code")]
        totp: bool,

        #[arg(long, help = "Show field values (requires password re-entry)")]
        show: bool,
    },

    #[command(about = "List all entries")]
    List,

    #[command(about = "Edit an entry")]
    Edit {
        #[arg(help = "Entry name")]
        name: String,

        #[arg(short, long, help = "Interactive mode")]
        interactive: bool,

        #[arg(short, long, value_parser = parse_field, help = "Add or update field (key=value)")]
        field: Vec<(String, String)>,

        #[arg(long, value_parser = parse_file_field, help = "Add or update field from file (key=path)")]
        file: Vec<(String, String)>,

        #[arg(short = 'd', long = "rm-field", help = "Remove field by key")]
        rm_field: Vec<String>,
    },

    #[command(about = "Remove an entry")]
    Remove {
        #[arg(help = "Entry name")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Set vault file path")]
    VaultPath {
        #[arg(help = "New vault file path")]
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

        #[arg(help = "Entry name or entry.field (e.g., myapp.db_url)")]
        entry_field: String,

        #[arg(long, help = "Override secret name in provider")]
        as_name: Option<String>,

        #[arg(long, help = "Push all fields from entry as separate secrets")]
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
    #[command(about = "Add TOTP secret to an entry")]
    Add {
        #[arg(help = "Entry name")]
        entry: String,

        #[arg(help = "TOTP secret (base32 encoded)")]
        secret: String,
    },

    #[command(about = "Get TOTP code (displays and copies to clipboard)")]
    Get {
        #[arg(help = "Entry name")]
        entry: String,
    },

    #[command(about = "Remove TOTP secret from an entry")]
    Rm {
        #[arg(help = "Entry name")]
        entry: String,
    },
}

#[derive(Subcommand)]
pub enum VaultCommands {
    #[command(about = "List all vaults")]
    List,

    #[command(about = "Create a new vault")]
    Create {
        #[arg(help = "Vault name")]
        name: String,
    },

    #[command(about = "Delete a vault")]
    Delete {
        #[arg(help = "Vault name")]
        name: String,

        #[arg(long, help = "Skip confirmation")]
        force: bool,
    },

    #[command(about = "Set active vault")]
    Use {
        #[arg(help = "Vault name")]
        name: String,
    },

    #[command(about = "Move entry to another vault")]
    Move {
        #[arg(help = "Entry name")]
        entry: String,

        #[arg(help = "Target vault name")]
        to_vault: String,
    },

    #[command(about = "Copy entry to another vault")]
    Copy {
        #[arg(help = "Entry name")]
        entry: String,

        #[arg(help = "Target vault name")]
        to_vault: String,
    },

    #[command(about = "Change master password")]
    Passwd,
}

#[derive(Subcommand)]
pub enum SshCommands {
    #[command(about = "Add SSH connection entry")]
    Add {
        #[arg(help = "Entry name")]
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
        #[arg(help = "Entry name containing SSH key")]
        name: String,

        #[arg(long, help = "Lifetime in seconds (0 = forever)")]
        lifetime: Option<u32>,
    },

    #[command(about = "Remove SSH key from ssh-agent")]
    Unload {
        #[arg(help = "Entry name or public key fingerprint")]
        name: String,
    },

    #[command(about = "List loaded SSH keys in ssh-agent")]
    List,

    #[command(about = "Connect to SSH host (auto-loads key from entry)")]
    Connect {
        #[arg(help = "Entry name or alias (e.g., git@github.com)")]
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
