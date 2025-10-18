use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::import_nfts::process::process_import;

pub mod process;

/// Arguments for importing existing NFTs metadata links into a Sugar cache.
#[derive(Debug, Args)]
pub struct ImportNFTsArgs {
    /// Path to the text file containing Arweave metadata URLs.
    #[clap(short, long, value_name = "FILE")]
    pub import: PathBuf,

    /// Path to the output cache file (e.g. ./cache.json)
    #[clap(short, long, default_value = "cache.json", value_name = "CACHE")]
    pub output: PathBuf,
}

/// Entry point for handling `sugar import` command.
pub async fn process_import_nfts_cmd(args: ImportNFTsArgs) -> Result<()> {
    // `process_import` is synchronous; call it and convert the result into anyhow::Result
    process_import(&args.import, &args.output)?;
    Ok(())
}
