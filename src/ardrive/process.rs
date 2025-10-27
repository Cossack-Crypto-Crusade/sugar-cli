use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::io::Write;

use anyhow::{Result, Context};
use tracing::info;
use serde_json::Value;

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

pub fn process_ardrive_list_drives(wallet: Option<PathBuf>) -> Result<()> {
    info!("ArDrive: list-drives called (wallet override: {:?})", wallet);

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Placeholder: we don't know the wallet shape here; print a summary and pretend to list drives.
    println!("Using ardrive wallet ({} bytes).", content.len());
    println!("(placeholder) Drives:\n- drive_alpha\n- drive_beta\n");

    Ok(())
}

pub fn process_ardrive_list_all_drives(wallet: Option<PathBuf>) -> Result<()> {
    info!("ArDrive: list-all-drives called (wallet override: {:?})", wallet);

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
        .output()
        .context("Failed to execute ardrive list-all-drives command")?;

    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    // Print the command output or error
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "ArDrive CLI command failed: {}", stderr
        ));
    }

    Ok(())
}

/// Debug helper: show which wallet the CLI resolved and print its JSON contents.
/// Use with an optional wallet path override. This is intended for debugging only
/// (the wallet contains private keys — don't paste the output publicly).
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
