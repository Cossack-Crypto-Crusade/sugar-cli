use clap::{Parser, Subcommand};

use crate::{
    config::TokenStandard,
    constants::{
        DEFAULT_AIRDROP_LIST, DEFAULT_AIRDROP_LIST_HELP, DEFAULT_ASSETS, DEFAULT_CACHE,
        DEFAULT_CONFIG, DEFAULT_PRIORITY_FEE,
    },
};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Log level: trace, debug, info, warn, error, off
    #[clap(short, long, global = true)]
    pub log_level: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interact with the bundlr network
    Bundlr {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        #[clap(subcommand)]
        action: BundlrAction,
    },

    /// Manage the collection on the candy machine
    Collection {
        #[clap(subcommand)]
        command: CollectionSubcommands,
    },

    /// Manage candy machine configuration
    Config {
        #[clap(subcommand)]
        command: ConfigSubcommands,
    },

    /// Deploy cache items into candy machine config on-chain
    Deploy {
        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// The optional collection address where the candymachine will mint the tokens to
        #[clap(long)]
        collection_mint: Option<String>,
    },

    /// Manage freeze guard actions
    Freeze {
        #[clap(subcommand)]
        command: FreezeCommand,
    },

    /// Manage guards on the candy machine
    Guard {
        #[clap(subcommand)]
        command: GuardCommand,
    },

    /// Generate hash of cache file for hidden settings.
    Hash {
        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Compare a provided hash with a cache file to check integrity.
        #[clap(long)]
        compare: Option<String>,
    },

    /// Create a candy machine deployment from assets
    Launch {
        /// Path to the directory with the assets to upload
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,

        /// Path to the keypair file [default: solana config or "~/.config/solana/id.json"]
        #[clap(short, long)]
        keypair: Option<String>,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Path to the cache file
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Strict mode: validate against JSON metadata standard exactly
        #[clap(long)]
        strict: bool,

        /// Skip collection validate prompt
        #[clap(long)]
        skip_collection_prompt: bool,
    },

    /// Import existing NFTs metadata links into a Sugar cache
    Import {
        /// Path to the text file containing Arweave metadata URLs.
        #[clap(short, long, value_name = "FILE")]
        import: std::path::PathBuf,

        /// Path to the output cache file (e.g. ./cache.json)
        #[clap(short, long, default_value = "cache.json", value_name = "CACHE")]
        output: std::path::PathBuf,
    },

    /// Mint one NFT from candy machine
    Mint {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Amount of NFTs to be minted in bulk
        #[clap(short, long)]
        number: Option<u64>,

        /// Public key of the receiver of the minted NFT, defaults to keypair
        #[clap(long)]
        receiver: Option<String>,

        /// Address of candy machine to mint from.
        #[clap(long)]
        candy_machine: Option<String>,
    },
    /// Airdrop NFTs from candy machine
    Airdrop {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Address of candy machine to mint from.
        #[clap(long)]
        candy_machine: Option<String>,

        /// List of airdrop targets.
        #[clap(long, default_value = DEFAULT_AIRDROP_LIST, help = DEFAULT_AIRDROP_LIST_HELP)]
        airdrop_list: String,
    },

    /// Reveal the NFTs from a hidden settings candy machine
    Reveal {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// RPC timeout to retrieve the mint list (in seconds).
        #[clap(short, long)]
        timeout: Option<u64>,

        /// Address to transfer the update authority to
        #[clap(short, long)]
        new_update_authority: Option<String>,
    },

    /// Show the on-chain config of an existing candy machine
    Show {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Address of candy machine
        candy_machine: Option<String>,

        /// Display a list of unminted indices
        #[clap(long)]
        unminted: bool,
    },

    /// Sign one or all NFTs from candy machine
    Sign {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Mint id for single NFT to be signed
        #[clap(short, long)]
        mint: Option<String>,

        /// Candy machine id.
        #[clap(long)]
        candy_machine_id: Option<String>,
    },

    /// Upload assets to storage and creates the cache config
    Upload {
        /// Path to the directory with the assets to upload
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the keypair file [default: solana config or "~/.config/solana/id.json"]
        #[clap(short, long)]
        keypair: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,
    },

    /// Interact with ArDrive storage service
    Ardrive {
        #[clap(subcommand)]
        command: ArdriveCommand,
    },

    /// Validate JSON metadata files
    Validate {
        /// Assets directory to upload, defaults to "assets"
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,

        /// Strict mode: validate against JSON metadata standard exactly
        #[clap(long)]
        strict: bool,

        /// Skip collection prompt
        #[clap(long)]
        skip_collection_prompt: bool,
    },

    /// Verify uploaded data
    Verify {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,
    },

    /// Withdraw funds a from candy machine account closing it
    Withdraw {
        /// Address of candy machine to withdraw funds from.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// List available candy machines, no withdraw performed
        #[clap(long)]
        list: bool,

        /// Address of authority to find candy machines for.
        /// If authority != keypair.pubkey then force --list.
        /// Defaults to keypair.pubkey.
        #[clap(long)]
        authority: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum BundlrAction {
    /// Retrieve the balance on bundlr
    Balance,
    /// Withdraw funds from bundlr
    Withdraw,
}

#[derive(Subcommand)]
pub enum ArdriveCommand {
    /// Upload a file to ArDrive
    Upload {
        /// Path to the file to upload
        file: std::path::PathBuf,

        /// Optional bucket name
        #[clap(short, long)]
        bucket: Option<String>,
    },

    /// List contents of a bucket
    List {
        /// Optional bucket name
        #[clap(short, long)]
        bucket: Option<String>,
    },

    /// Show info about an item
    Info {
        /// Item id
        id: String,
    },

    /// Delete an item
    Delete {
        /// Item id
        id: String,
    },
    /// Export (set) an ArDrive wallet file for CLI usage
    #[clap(alias = "export")]
    SetWallet {
        /// Path to the ardrive wallet JSON file
        wallet: std::path::PathBuf,
    },
    /// List contents of a specific ArDrive drive
    ListDrives {
        /// Optional path to the ardrive wallet JSON file (overrides stored wallet)
        #[clap(short, long, value_name = "WALLET")]
        wallet: Option<std::path::PathBuf>,

        /// ID of the drive to list (required)
        #[clap(short, long)]
        drive_id: String,
    },
    /// List files in a specific ArDrive drive
    ListDriveFiles {
        /// Optional path to the ardrive wallet JSON file (overrides stored wallet)
        #[clap(short, long, value_name = "WALLET")]
        wallet: Option<std::path::PathBuf>,

        /// ID of the drive to list (required)
        #[clap(short, long)]
        drive_id: String,

        /// Optional JSON output file path to save the file list
        #[clap(short, long, value_name = "OUTPUT")]
        output: Option<std::path::PathBuf>,

        /// Optional file extension filter (e.g. json)
        #[clap(short = 'e', long, value_name = "EXT")]
        filter: Option<String>,
    },
    /// List all drives (detailed) accessible by the wallet
    ListAllDrives {
        /// Optional path to the ardrive wallet JSON file (overrides stored wallet)
        #[clap(short, long, value_name = "WALLET")]
        wallet: Option<std::path::PathBuf>,

        /// Optional JSON output file path to save the drive list
        #[clap(short, long, value_name = "OUTPUT")]
        output: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ConfigSubcommands {
    /// Interactive process to create a config file
    Create {
        /// Path to the config file
        #[clap(short, long)]
        config: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the keypair file [default: solana config or "~/.config/solana/id.json"]
        #[clap(short, long)]
        keypair: Option<String>,

        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS)]
        assets_dir: String,
    },
    /// Update the candy machine config on-chain
    Update {
        /// Path to the config file, defaults to "config.json"
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Pubkey for the new authority
        #[clap(short, long)]
        new_authority: Option<String>,

        /// Address of candy machine to update.
        #[clap(long)]
        candy_machine: Option<String>,
    },
    /// Set specific candy machine config values
    Set {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Token Standard to set.
        #[clap(short, long)]
        token_standard: Option<TokenStandard>,

        /// Address of candy machine to update.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of the rule set to use.
        #[clap(long)]
        rule_set: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CollectionSubcommands {
    /// Set the collection mint on the candy machine
    Set {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Address of candy machine to update.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of collection mint to set the candy machine to.
        collection_mint: String,
    },
}

#[derive(Subcommand)]
pub enum GuardCommand {
    /// Add a candy guard on a candy machine
    Add {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Address of the candy machine.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of the candy guard.
        #[clap(long)]
        candy_guard: Option<String>,
    },
    /// Remove a candy guard from a candy machine
    Remove {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Address of the candy machine.
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of the candy guard.
        #[clap(long)]
        candy_guard: Option<String>,
    },
    /// Show the on-chain config of an existing candy guard
    Show {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Address of the candy guard.
        #[clap(long)]
        candy_guard: Option<String>,
    },
    /// Update the configuration of a candy guard
    Update {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Address of the candy guard.
        #[clap(long)]
        candy_guard: Option<String>,
    },
    /// Withdraw funds from a candy guard account closing it
    Withdraw {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Address of the candy guard.
        #[clap(long)]
        candy_guard: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum FreezeCommand {
    /// Initialize the freeze escrow account.
    Initialize {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Address of candy guard to update [defaults to cache value].
        #[clap(long)]
        candy_guard: Option<String>,

        /// Address of candy machine to update [defaults to cache value].
        #[clap(long)]
        candy_machine: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Candy guard group label.
        #[clap(long)]
        label: Option<String>,

        /// Freeze period in seconds (maximum 30 days).
        #[clap(long)]
        period: u64,
    },
    /// Thaw a NFT or all NFTs in a candy guard.
    Thaw {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Unthaw all NFTs in the candy machine.
        #[clap(long)]
        all: bool,

        /// Address of the NFT to thaw.
        nft_mint: Option<String>,

        /// Address of candy guard to update [defaults to cache value].
        #[clap(long)]
        candy_guard: Option<String>,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Address of candy machine to update [defaults to cache value].
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of the destination account on the freeze guard.
        #[clap(long)]
        destination: Option<String>,

        /// Candy guard group label.
        #[clap(long)]
        label: Option<String>,

        /// Indicates to create/use a cache file for mint list.
        #[clap(long)]
        use_cache: bool,

        /// RPC timeout to retrieve the mint list (in seconds).
        #[clap(short, long)]
        timeout: Option<u64>,

        /// Indicates whether this is a freeze token payment guard or not.
        #[clap(long)]
        token: bool,
    },
    /// Unlock treasury funds after freeze is turned off or expires.
    UnlockFunds {
        /// Path to the keypair file, uses Sol config or defaults to "~/.config/solana/id.json"
        #[clap(short, long)]
        keypair: Option<String>,

        /// RPC Url
        #[clap(short, long)]
        rpc_url: Option<String>,

        /// Path to the cache file, defaults to "cache.json"
        #[clap(long, default_value = DEFAULT_CACHE)]
        cache: String,

        /// Path to the config file
        #[clap(short, long, default_value = DEFAULT_CONFIG)]
        config: String,

        /// Priority fee value
        #[clap(short, long, default_value_t = DEFAULT_PRIORITY_FEE)]
        priority_fee: u64,

        /// Address of candy guard to update [defaults to cache value].
        #[clap(long)]
        candy_guard: Option<String>,

        /// Address of candy machine to update [defaults to cache value].
        #[clap(long)]
        candy_machine: Option<String>,

        /// Address of the destination (treasury) account.
        #[clap(long)]
        destination: Option<String>,

        /// Candy guard group label.
        #[clap(long)]
        label: Option<String>,

        /// Indicates whether this is a freeze token payment guard or not.
        #[clap(long)]
        token: bool,
    },
}
