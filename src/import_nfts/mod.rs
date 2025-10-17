use std::path::PathBuf;
use clap::Args;

pub mod process;
pub use process::process_import_nfts;

/// CLI arguments for the `import-nfts` subcommand.
#[derive(Debug, Args)]
#[command(
    about = "Import a list of existing Arweave metadata links to generate a Sugar cache file"
)]
pub struct ImportNFTsArgs {
    /// Path to a text file containing one Arweave metadata link per line.
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Output path for the generated Candy Machine cache.json
    #[arg(short, long, default_value = "cache.json", value_name = "FILE")]
    pub output: PathBuf,
}
