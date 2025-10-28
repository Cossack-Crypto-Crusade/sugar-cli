use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::io::Write;

use anyhow::{Result, Context};
use tracing::info;
use serde_json::Value;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArDriveDrive {
    #[serde(rename = "appName")]
    pub app_name: Option<String>,
    #[serde(rename = "appVersion")]  
    pub app_version: Option<String>,
    #[serde(rename = "arFS")]
    pub ar_fs: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "driveId")]
    pub drive_id: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "txId")]
    pub tx_id: Option<String>,
    #[serde(rename = "unixTime")]
    pub unix_time: Option<u64>,
    #[serde(rename = "customMetaDataGqlTags")]
    pub custom_meta_data_gql_tags: Option<serde_json::Map<String, Value>>,
    #[serde(rename = "customMetaDataJson")]
    pub custom_meta_data_json: Option<serde_json::Map<String, Value>>,
    #[serde(rename = "drivePrivacy")]
    pub drive_privacy: Option<String>,
    #[serde(rename = "rootFolderId")]
    pub root_folder_id: Option<String>,
    // Optional encrypted fields
    #[serde(rename = "driveAuthMode")]
    pub drive_auth_mode: Option<String>,
    pub cipher: Option<String>,
    #[serde(rename = "cipherIV")]
    pub cipher_iv: Option<String>,
    #[serde(rename = "driveSignatureType")]
    pub drive_signature_type: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArDriveFile {
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "dataTxId")]
    pub data_tx_id: Option<String>,  // Arweave transaction ID for the file data
    #[serde(rename = "metadataTxId")]
    pub metadata_tx_id: Option<String>,  // Arweave transaction ID for the metadata
    #[serde(rename = "parentFolderId")]
    pub parent_folder_id: Option<String>,
    pub size: Option<u64>,
    #[serde(rename = "lastModifiedDate")]
    pub last_modified_date: Option<u64>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "dataContentType")]
    pub data_content_type: Option<String>,
}

impl ArDriveDrive {
    /// Find a drive by name or ID from a list of drives
    pub fn find_in_list<'a>(drives: &'a [ArDriveDrive], name_or_id: &str) -> Option<&'a ArDriveDrive> {
        drives.iter().find(|d| {
            d.drive_id.as_deref() == Some(name_or_id) || 
            d.name.as_deref() == Some(name_or_id)
        })
    }
}

// Placeholder implementations for ArDrive interactions.
// Replace these with real SDK calls / HTTP requests as needed.

pub fn process_ardrive_upload(file: PathBuf, bucket: Option<String>) -> Result<()> {
    info!("ArDrive: upload called: file={:?} bucket={:?}", file, bucket);
    println!("(ardrive) Uploading {:?} to {:?} (placeholder)", file, bucket);
    Ok(())
}

pub fn process_ardrive_list(bucket: Option<String>) -> Result<()> {
    info!("ArDrive: list called: bucket={:?}", bucket);
    println!("(ardrive) Listing contents of {:?} (placeholder)", bucket);
    Ok(())
}

pub fn process_ardrive_info(id: String) -> Result<()> {
    info!("ArDrive: info called: id={}", id);
    println!("(ardrive) Info for id {} (placeholder)", id);
    Ok(())
}

pub fn process_ardrive_delete(id: String) -> Result<()> {
    info!("ArDrive: delete called: id={}", id);
    println!("(ardrive) Delete id {} (placeholder)", id);
    Ok(())
}

/// Store the provided ardrive wallet file contents into the user's config
/// so other CLI calls can read it. We copy the file contents into
/// ~/.config/sugar-cli/ardrive_wallet.json (creates dirs if needed).
pub fn process_ardrive_set_wallet(wallet_file: std::path::PathBuf) -> Result<()> {
    use std::fs;
    use std::io::Write;
    info!("ArDrive: set wallet called: {:?}", wallet_file);
    // PathBuf is already imported at top-level
    // use std::path::PathBuf;

    let content = fs::read_to_string(&wallet_file)
        .map_err(|e| anyhow::anyhow!("Failed to read wallet file {}: {}", wallet_file.display(), e))?;

    let home = std::env::var("HOME").map_err(|e| anyhow::anyhow!("HOME not set: {}", e))?;
    let mut cfg_dir = PathBuf::from(home);
    cfg_dir.push(".config");
    cfg_dir.push("sugar-cli");

    fs::create_dir_all(&cfg_dir).map_err(|e| anyhow::anyhow!("Failed to create config dir: {}", e))?;

    let mut out = cfg_dir;
    out.push("ardrive_wallet.json");

    let mut f = fs::File::create(&out)
        .map_err(|e| anyhow::anyhow!("Failed to create wallet config file {}: {}", out.display(), e))?;
    f.write_all(content.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write wallet config file {}: {}", out.display(), e))?;

    println!(
        "✅ Stored ardrive wallet to {}. Other ardrive commands will use this wallet.",
        out.display()
    );

    println!(
        "To export into your shell session run: export ARDRIVE_WALLET=$(cat {})",
        out.display()
    );

    Ok(())
}

/// Resolve the ardrive wallet content from (in order):
/// 1) explicit PathBuf passed by user (read file),
/// 2) ARDRIVE_WALLET environment variable (expected to contain the JSON contents),
/// 3) saved file at ~/.config/sugar-cli/ardrive_wallet.json
fn resolve_ardrive_wallet_content(opt_wallet: Option<PathBuf>) -> anyhow::Result<String> {
    use std::fs;

    if let Some(p) = opt_wallet {
        let s = fs::read_to_string(&p)
            .map_err(|e| anyhow::anyhow!("Failed reading wallet file {}: {}", p.display(), e))?;
        return Ok(s);
    }

    if let Ok(env_val) = std::env::var("ARDRIVE_WALLET") {
        if !env_val.trim().is_empty() {
            return Ok(env_val);
        }
    }

    // fallback to saved path
    if let Ok(home) = std::env::var("HOME") {
        let mut cfg = PathBuf::from(home);
        cfg.push(".config");
        cfg.push("sugar-cli");
        cfg.push("ardrive_wallet.json");

        if cfg.exists() {
            let s = fs::read_to_string(&cfg)
                .map_err(|e| anyhow::anyhow!("Failed reading stored wallet {}: {}", cfg.display(), e))?;
            return Ok(s);
        }
    }

    Err(anyhow::anyhow!("No ardrive wallet provided: pass -w/--wallet, set ARDRIVE_WALLET env var, or run 'sugar ardrive set-wallet <file>' to store one."))
}

pub fn process_ardrive_list_drives(wallet: Option<PathBuf>, drive_id: String) -> Result<()> {
    info!("ArDrive: list-drives called (wallet override: {:?}, drive_id: {})", wallet, drive_id);

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Create a temporary file for the wallet
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("sugar-cli-wallet.tmp.json");
    
    let mut temp_file = fs::File::create(&temp_dir)
        .context("Failed to create temporary wallet file")?;
    temp_file.write_all(content.as_bytes())
        .context("Failed to write wallet content to temporary file")?;
    
    // Make sure the file is written and closed
    drop(temp_file);

    info!("Using ardrive wallet ({} bytes) from temporary file", content.len());

    // Run ardrive list-drive with our temporary wallet file
    let output = Command::new("ardrive")
        .arg("list-drive")
        .arg("-d")  // or --drive-id
        .arg(&drive_id)
        .arg("--wallet-file")
        .arg(&temp_dir)
        .output()
        .context("Failed to execute ardrive list-drive command")?;

    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    // Print the command output or error
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "ArDrive CLI command failed: {}", stderr
        ))
    }
}

pub fn process_ardrive_list_all_drives(wallet: Option<PathBuf>, output_path: Option<PathBuf>) -> Result<Vec<ArDriveDrive>> {
    info!("ArDrive: list-all-drives called (wallet override: {:?}, output: {:?})", wallet, output_path);

    // First check if ardrive CLI is available
    let ardrive_version = Command::new("ardrive")
        .arg("--version")
        .output()
        .context("Failed to run 'ardrive --version'. Is ArDrive CLI installed? Install with: npm install -g ardrive-cli")?;

    if !ardrive_version.status.success() {
        return Err(anyhow::anyhow!(
            "ArDrive CLI not found or failed. Install with: npm install -g ardrive-cli"
        ));
    }

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Create a temporary file for the wallet
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("sugar-cli-wallet.tmp.json");
    
    let mut temp_file = fs::File::create(&temp_dir)
        .context("Failed to create temporary wallet file")?;
    temp_file.write_all(content.as_bytes())
        .context("Failed to write wallet content to temporary file")?;
    
    // Make sure the file is written and closed
    drop(temp_file);

    info!("Using ardrive wallet ({} bytes) from temporary file", content.len());

    // Run ardrive list-all-drives with our temporary wallet file
    let output = Command::new("ardrive")
        .arg("list-all-drives")
        .arg("--wallet-file")
        .arg(&temp_dir)
        .arg("--json") // ensure output is JSON
        .output()
        .context("Failed to execute ardrive list-all-drives command")?;

    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    // Parse the command output or error
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let drives: Vec<ArDriveDrive> = serde_json::from_str(&stdout)
            .context("Failed to parse ArDrive CLI output as Vec<ArDriveDrive>")?;
        println!("Drives: {}", serde_json::to_string_pretty(&drives).unwrap_or_else(|_| stdout.to_string()));

        // Optionally write to file
        if let Some(path) = output_path {
            fs::write(&path, serde_json::to_string_pretty(&drives).context("Failed to serialize drives JSON")?)
                .context(format!("Failed to write drives to file: {}", path.display()))?;
            println!("✅ Drives written to {}", path.display());
        }

        Ok(drives)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "ArDrive CLI command failed: {}", stderr
        ));
    }
}

/// Debug helper: show which wallet the CLI resolved and print its JSON contents.
/// Use with an optional wallet path override. This is intended for debugging only
/// (the wallet contains private keys — don't paste the output publicly).
/// List all files in a specific drive. Returns a Vec of files with their names and Arweave URLs.
/// Can filter by file extension using filter_ext (e.g. Some("json") for .json files only).
pub fn process_ardrive_list_drive_files(
    wallet: Option<PathBuf>,
    drive_id: String,
    output_path: Option<PathBuf>,
    filter_ext: Option<&str>
) -> Result<Vec<ArDriveFile>> {
    info!("ArDrive: list-drive-files called for drive {} (wallet override: {:?}, filter: {:?})", 
        drive_id, wallet, filter_ext);

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Create a temporary file for the wallet
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("sugar-cli-wallet.tmp.json");
    
    let mut temp_file = fs::File::create(&temp_dir)
        .context("Failed to create temporary wallet file")?;
    temp_file.write_all(content.as_bytes())
        .context("Failed to write wallet content to temporary file")?;
    
    drop(temp_file);

    // Run ardrive list-files with our temporary wallet file
    let mut cmd = Command::new("ardrive");
    cmd.arg("list-files")
       .arg("--drive-id")
       .arg(&drive_id)
       .arg("--wallet-file")
       .arg(&temp_dir)
       .arg("--json"); // ensure JSON output

    let output = cmd.output()
        .context("Failed to execute ardrive list-files command")?;

    // Clean up temp file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files: Vec<ArDriveFile> = serde_json::from_str(&stdout)
            .context("Failed to parse ArDrive CLI output as Vec<ArDriveFile>")?;

        // Filter by extension if requested
        if let Some(ext) = filter_ext {
            files.retain(|f| {
                f.name.as_ref()
                    .map(|n| n.ends_with(&format!(".{}", ext)))
                    .unwrap_or(false)
            });
        }

        // Print summary
        println!("Found {} files in drive {}", files.len(), drive_id);
        
        // Optionally write to file
        if let Some(path) = output_path {
            fs::write(&path, serde_json::to_string_pretty(&files)
                .context("Failed to serialize files to JSON")?)
                .context(format!("Failed to write files list to {}", path.display()))?;
            println!("✅ File list written to {}", path.display());
        }

        Ok(files)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("ArDrive CLI command failed: {}", stderr))
    }
}

/// Helper function to generate an Arweave URL from a transaction ID
pub fn get_arweave_url(tx_id: &str) -> String {
    format!("https://arweave.net/{}", tx_id)
}

pub fn process_ardrive_show_wallet(wallet: Option<PathBuf>) -> Result<()> {
    info!("ArDrive: show-wallet called (wallet override: {:?})", wallet);

    use std::fs;

    // Resolve source (prefer explicit path, then env var, then stored file)
    let mut source = String::new();
    let content: String;

    if let Some(p) = wallet {
        content = fs::read_to_string(&p)
            .map_err(|e| anyhow::anyhow!("Failed reading wallet file {}: {}", p.display(), e))?;
        source = format!("file: {}", p.display());
    } else if let Ok(env_val) = std::env::var("ARDRIVE_WALLET") {
        if !env_val.trim().is_empty() {
            content = env_val;
            source = "environment variable ARDRIVE_WALLET".to_string();
        } else {
            content = String::new();
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let mut cfg = PathBuf::from(home);
        cfg.push(".config");
        cfg.push("sugar-cli");
        cfg.push("ardrive_wallet.json");

        if cfg.exists() {
            content = fs::read_to_string(&cfg)
                .map_err(|e| anyhow::anyhow!("Failed reading stored wallet {}: {}", cfg.display(), e))?;
            source = format!("stored file: {}", cfg.display());
        } else {
            return Err(anyhow::anyhow!("No ardrive wallet provided: pass -w/--wallet, set ARDRIVE_WALLET env var, or run 'sugar ardrive set-wallet <file>' to store one."));
        }
    } else {
        return Err(anyhow::anyhow!("No ardrive wallet provided: pass -w/--wallet, set ARDRIVE_WALLET env var, or run 'sugar ardrive set-wallet <file>' to store one."));
    }

    println!("ArDrive wallet source: {}", source);
    println!("Wallet size: {} bytes", content.len());
    println!("WARNING: wallet contains private keys — do not share output publicly.");

    // Try to parse JSON and pretty-print
    match serde_json::from_str::<Value>(&content) {
        Ok(val) => {
            // If it's an object, list top-level keys
            if let Value::Object(map) = &val {
                println!("Top-level keys ({}): {}", map.len(), map.keys().cloned().collect::<Vec<_>>().join(", "));
            }

            match serde_json::to_string_pretty(&val) {
                Ok(pretty) => println!("\n{}", pretty),
                Err(e) => println!("(failed to pretty-print JSON: {})\nRaw contents:\n{}", e, content),
            }
        }
        Err(_) => {
            println!("(wallet content is not valid JSON)\nRaw contents:\n{}", content);
        }
    }

    Ok(())
}
