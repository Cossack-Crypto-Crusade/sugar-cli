use std::path::PathBuf;

use anyhow::Result;
use tracing::info;

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

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Using ardrive wallet ({} bytes).", content.len());
    println!("(placeholder) All drives (detailed):\n- drive_alpha: id=AAA created=2024-01-01\n- drive_beta: id=BBB created=2024-02-02\n");

    Ok(())
}
