use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

/// Find node executable in common locations or PATH
#[allow(dead_code)]
fn find_node() -> Result<PathBuf> {
    // Common locations
    let common_paths = ["/usr/bin/node", "/usr/local/bin/node", "/opt/node/bin/node"];

    // Check common paths first
    for path in common_paths.iter() {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    // Check PATH environment
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let full_path = PathBuf::from(dir).join("node");
            if full_path.exists() {
                return Ok(full_path);
            }
        }
    }

    Err(anyhow!(
        "Node.js not found. Please install Node.js and make sure it's in your PATH"
    ))
}

/// Walk upwards from cwd to find node_modules/.bin/ardrive
fn find_local_ardrive() -> Option<PathBuf> {
    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            let candidate = dir.join("node_modules/.bin/ardrive");
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
}

/// Given a serde_json::Value try to extract an array of drive objects
/// Handles common shapes returned by CLIs: array, { drives: [...] }, { data: [...] },
/// or an object map where values are drive objects.
fn extract_drives_from_value(v: &Value) -> Option<Vec<Value>> {
    info!(
        "Extracting drives from value type: {}",
        match v {
            serde_json::Value::Object(_) => "object",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Null => "null",
        }
    );

    match v {
        serde_json::Value::Array(arr) => {
            info!("Found array with {} items", arr.len());
            Some(arr.clone())
        }
        serde_json::Value::Object(map) => {
            info!(
                "Found object with keys: {}",
                map.keys()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            // Common wrappers
            for key in ["drives", "data", "result", "items", "rows"].iter() {
                if let Some(inner) = map.get(*key) {
                    if let serde_json::Value::Array(arr) = inner {
                        info!("Found array under key '{}' with {} items", key, arr.len());
                        return Some(arr.clone());
                    }
                    info!("Found non-array value under key '{}'", key);
                }
            }

            // If it's an object whose values look like drive objects, return the values
            let mut vals = Vec::new();
            for (_k, v) in map.iter() {
                // Heuristic: object entries
                if v.is_object() {
                    vals.push(v.clone());
                }
            }
            if !vals.is_empty() {
                info!(
                    "Found {} object values that look like drive objects",
                    vals.len()
                );
                return Some(vals);
            }

            info!("No valid drive objects found in object");
            None
        }
        _ => {
            info!("Value is neither array nor object, cannot extract drives");
            None
        }
    }
}

/// Remove common ANSI escape sequences from output (very small scanner, no external deps).
#[allow(dead_code)]
fn strip_ansi_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until a letter in the range [@-~] (final byte of CSI sequences) or end
            if let Some('[') = chars.peek() {
                // consume '['
                chars.next();
                // consume until ascii letter
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if ('@'..='~').contains(&nc) {
                        break;
                    }
                }
                continue;
            } else {
                // Unknown escape, skip next
                continue;
            }
        }
        out.push(c);
    }

    out
}

/// Try several heuristics to extract a JSON document from CLI output and parse it.
#[allow(dead_code)]
fn try_parse_json_flex(raw: &str) -> Result<Value, serde_json::Error> {
    let cleaned = strip_ansi_codes(raw).trim().to_string();

    // If the whole cleaned string parses, return it.
    if let Ok(v) = serde_json::from_str::<Value>(&cleaned) {
        return Ok(v);
    }

    // Try to find first json start and last json end and parse that slice.
    if let Some(start) = cleaned.find('{').or_else(|| cleaned.find('[')) {
        // find last occurrence of closing braces
        if let Some(end_brace) = cleaned.rfind('}') {
            if end_brace > start {
                let candidate = &cleaned[start..=end_brace];
                if let Ok(v) = serde_json::from_str::<Value>(candidate) {
                    return Ok(v);
                }
            }
        }
        if let Some(end_bracket) = cleaned.rfind(']') {
            if end_bracket > start {
                let candidate = &cleaned[start..=end_bracket];
                if let Ok(v) = serde_json::from_str::<Value>(candidate) {
                    return Ok(v);
                }
            }
        }
    }

    // As a last resort, try parsing each non-empty line that looks like JSON
    for line in cleaned.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('{') || t.starts_with('[') {
            if let Ok(v) = serde_json::from_str::<Value>(t) {
                return Ok(v);
            }
        }
    }

    // give up and return the last parse error from attempting the whole cleaned string
    serde_json::from_str::<Value>(&cleaned)
}

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
    pub data_tx_id: Option<String>, // Arweave transaction ID for the file data
    #[serde(rename = "metadataTxId")]
    pub metadata_tx_id: Option<String>, // Arweave transaction ID for the metadata
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
    pub fn find_in_list<'a>(
        drives: &'a [ArDriveDrive],
        name_or_id: &str,
    ) -> Option<&'a ArDriveDrive> {
        drives.iter().find(|d| {
            d.drive_id.as_deref() == Some(name_or_id) || d.name.as_deref() == Some(name_or_id)
        })
    }
}

// Placeholder implementations for ArDrive interactions.
// Replace these with real SDK calls / HTTP requests as needed.

pub fn process_ardrive_upload(file: PathBuf, bucket: Option<String>) -> Result<()> {
    info!(
        "ArDrive: upload called: file={:?} bucket={:?}",
        file, bucket
    );
    println!(
        "(ardrive) Uploading {:?} to {:?} (placeholder)",
        file, bucket
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_extract_drives_from_array() {
        let v = json!([
            {"driveId": "a", "name": "A"},
            {"driveId": "b", "name": "B"}
        ]);
        let res = extract_drives_from_value(&v).expect("should extract array");
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn test_extract_drives_from_wrapper() {
        let v = json!({"drives": [{"driveId":"x"}], "meta": {}});
        let res = extract_drives_from_value(&v).expect("should extract from wrapper");
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_extract_drives_from_map_of_objects() {
        let v = json!({"a": {"driveId":"a"}, "b": {"driveId":"b"}});
        let res = extract_drives_from_value(&v).expect("should extract from map");
        assert_eq!(res.len(), 2);
    }
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
    info!("ArDrive: set wallet called: {:?}", wallet_file);
    // PathBuf is already imported at top-level
    // use std::path::PathBuf;

    let content = fs::read_to_string(&wallet_file).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read wallet file {}: {}",
            wallet_file.display(),
            e
        )
    })?;

    let home = std::env::var("HOME").map_err(|e| anyhow::anyhow!("HOME not set: {}", e))?;
    let mut cfg_dir = PathBuf::from(home);
    cfg_dir.push(".config");
    cfg_dir.push("sugar-cli");

    fs::create_dir_all(&cfg_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create config dir: {}", e))?;

    let mut out = cfg_dir;
    out.push("ardrive_wallet.json");

    let mut f = fs::File::create(&out).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create wallet config file {}: {}",
            out.display(),
            e
        )
    })?;
    f.write_all(content.as_bytes()).map_err(|e| {
        anyhow::anyhow!(
            "Failed to write wallet config file {}: {}",
            out.display(),
            e
        )
    })?;

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
            let s = fs::read_to_string(&cfg).map_err(|e| {
                anyhow::anyhow!("Failed reading stored wallet {}: {}", cfg.display(), e)
            })?;
            return Ok(s);
        }
    }

    Err(anyhow::anyhow!("No ardrive wallet provided: pass -w/--wallet, set ARDRIVE_WALLET env var, or run 'sugar ardrive set-wallet <file>' to store one."))
}

pub fn process_ardrive_list_drives(wallet: Option<PathBuf>, drive_id: String) -> Result<()> {
    info!(
        "ArDrive: list-drives called (wallet override: {:?}, drive_id: {})",
        wallet, drive_id
    );

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create a temporary file for the wallet
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("sugar-cli-wallet.tmp.json");

    let mut temp_file =
        fs::File::create(&temp_dir).context("Failed to create temporary wallet file")?;
    temp_file
        .write_all(content.as_bytes())
        .context("Failed to write wallet content to temporary file")?;

    // Make sure the file is written and closed
    drop(temp_file);

    info!(
        "Using ardrive wallet ({} bytes) from temporary file",
        content.len()
    );

    // Run ardrive list-drive with our temporary wallet file
    // Prefer a local wrapper in node_modules/.bin; otherwise fall back to system `ardrive`.
    let ardrive_local = find_local_ardrive();
    let output = if let Some(ref ardrive_path) = ardrive_local {
        let mut cmd = Command::new(ardrive_path);
        cmd.arg("list-drive")
            .arg("-d") // or --drive-id
            .arg(&drive_id)
            .arg("--wallet-file")
            .arg(&temp_dir)
            .env("NODE_ENV", "production");
        cmd.output()
            .context("Failed to execute local ardrive bin for list-drive")?
    } else {
        let mut cmd = Command::new("ardrive");
        cmd.arg("list-drive")
            .arg("-d")
            .arg(&drive_id)
            .arg("--wallet-file")
            .arg(&temp_dir)
            .env("NODE_ENV", "production");
        cmd.output()
            .context("Failed to execute system ardrive for list-drive")?
    };

    // Clean up the temporary file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    // Print the command output or error
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        if !stderr.is_empty() {
            info!("ArDrive CLI stderr (non-fatal): {}", stderr);
        }
        println!("{}", stdout);
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "ArDrive CLI command failed:\nStdout: {}\nStderr: {}\nCommand: list-drive -d {} --wallet-file <wallet>",
            stdout, stderr, drive_id
        ))
    }
}

pub fn process_ardrive_list_all_drives(
    wallet: Option<PathBuf>,
    output_path: Option<PathBuf>,
) -> Result<Vec<ArDriveDrive>> {
    info!(
        "ArDrive: list-all-drives called (wallet override: {:?}, output: {:?})",
        wallet, output_path
    );

    // Prefer a local wrapper for version check if present
    let ardrive_local = find_local_ardrive();
    let ardrive_version = if let Some(local) = &ardrive_local {
        Command::new(local).arg("--version").output()
    } else {
        Command::new("ardrive").arg("--version").output()
    }
    .context("Failed to run 'ardrive --version'. Is ArDrive CLI installed? Install with: pnpm add ardrive-cli --save-exact or `pnpm add -g ardrive-cli` for global usage")?;

    if !ardrive_version.status.success() {
        return Err(anyhow::anyhow!(
            "ArDrive CLI not found or failed. Install with: pnpm add ardrive-cli or pnpm add -g ardrive-cli"
        ));
    }

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create a temporary file for the wallet
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("sugar-cli-wallet.tmp.json");

    let mut temp_file =
        fs::File::create(&temp_dir).context("Failed to create temporary wallet file")?;
    temp_file
        .write_all(content.as_bytes())
        .context("Failed to write wallet content to temporary file")?;

    // Make sure the file is written and closed
    drop(temp_file);

    info!(
        "Using ardrive wallet ({} bytes) from temporary file",
        content.len()
    );

    // Try to get help output from local wrapper or system binary
    if let Some(local) = &ardrive_local {
        if let Ok(help_output) = Command::new(local)
            .arg("list-all-drives")
            .arg("--help")
            .output()
        {
            info!(
                "ArDrive command help (local): {}",
                String::from_utf8_lossy(&help_output.stdout)
            );
        }
    } else if let Ok(help_output) = Command::new("ardrive")
        .arg("list-all-drives")
        .arg("--help")
        .output()
    {
        info!(
            "ArDrive command help (system): {}",
            String::from_utf8_lossy(&help_output.stdout)
        );
    }

    // Run ardrive list-all-drives with our temporary wallet file
    let output = if let Some(ref ardrive_path) = ardrive_local {
        // Execute the wrapper directly (it will invoke node via its shebang)
        let mut cmd = Command::new(ardrive_path);
        cmd.arg("list-all-drives")
            .arg("--wallet-file")
            .arg(&temp_dir)
            .env("NODE_ENV", "production");

        info!(
            "Running local ardrive bin: {} list-all-drives --wallet-file {:?}",
            ardrive_path.display(),
            temp_dir
        );
        cmd.output()
            .context("Failed to execute local ardrive bin")?
    } else {
        // Fallback: run system ardrive
        let mut cmd = Command::new("ardrive");
        cmd.arg("list-all-drives")
            .arg("--wallet-file")
            .arg(&temp_dir)
            .env("NODE_ENV", "production");

        info!(
            "Running system ardrive: list-all-drives --wallet-file {:?}",
            temp_dir
        );
        cmd.output().context("Failed to execute system ardrive")?
    }; // Clean up the temporary file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    // Parse the command output or error
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Log both stdout and stderr for debugging
        info!("ArDrive CLI stdout: {}", stdout);
        if !stderr.is_empty() {
            info!("ArDrive CLI stderr: {}", stderr);
        }

        // Try to normalize potential line noise
        let clean_stdout = stdout.trim().trim_start_matches('\u{feff}');

        // Try to parse and provide detailed error information
        let val: Value = match serde_json::from_str(clean_stdout) {
            Ok(v) => v,
            Err(e) => {
                let error_msg = format!(
                    "\nParse error: {}\nCommand output:\n{}\n\nNote: Make sure ardrive-cli is installed with `pnpm add ardrive-cli`",
                    e, clean_stdout
                );
                return Err(anyhow!(
                    "ArDrive CLI returned invalid output: {}",
                    error_msg
                ));
            }
        };

        let items = extract_drives_from_value(&val).ok_or_else(|| {
            anyhow!(
                "Unexpected JSON shape from ArDrive CLI. Expected array or object with drives.\n\
                Actual JSON structure: {}\n\
                Raw stdout: {}\n\
                Stderr: {}",
                serde_json::to_string_pretty(&val)
                    .unwrap_or_else(|_| "<failed to pretty print>".to_string()),
                stdout,
                stderr
            )
        })?;

        // Deserialize each item into ArDriveDrive
        let mut drives: Vec<ArDriveDrive> = Vec::with_capacity(items.len());
        for item in items.into_iter() {
            let d: ArDriveDrive = serde_json::from_value(item)
                .context("Failed to deserialize drive object from ArDrive output")?;
            drives.push(d);
        }

        println!("Found {} drive(s)", drives.len());
        println!(
            "Drives: {}",
            serde_json::to_string_pretty(&drives).unwrap_or_else(|_| stdout.to_string())
        );

        // Optionally write to file
        if let Some(path) = output_path {
            fs::write(
                &path,
                serde_json::to_string_pretty(&drives).context("Failed to serialize drives JSON")?,
            )
            .context(format!(
                "Failed to write drives to file: {}",
                path.display()
            ))?;
            println!("✅ Drives written to {}", path.display());
        }

        Ok(drives)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(anyhow::anyhow!(
            "ArDrive CLI command failed.\nStderr: {}\nStdout: {}\n\nIf you don't have ArDrive installed locally, run: `pnpm add ardrive-cli` in this project (then `pnpm install`).\nOr install globally: `pnpm add -g ardrive-cli`. Ensure Node.js is installed and the wrapper is executable.",
            stderr, stdout
        ))
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
    filter_ext: Option<&str>,
) -> Result<Vec<ArDriveFile>> {
    info!(
        "ArDrive: list-drive-files called for drive {} (wallet override: {:?}, filter: {:?})",
        drive_id, wallet, filter_ext
    );

    let content = resolve_ardrive_wallet_content(wallet).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create a temporary file for the wallet
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("sugar-cli-wallet.tmp.json");

    let mut temp_file =
        fs::File::create(&temp_dir).context("Failed to create temporary wallet file")?;
    temp_file
        .write_all(content.as_bytes())
        .context("Failed to write wallet content to temporary file")?;

    drop(temp_file);

    // Check for ardrive CLI and get its help output
    let ardrive_local = find_local_ardrive();
    let output = if let Some(ref ardrive_path) = ardrive_local {
        info!("Using local ardrive at: {}", ardrive_path.display());

        // Try getting help output first
        if let Ok(help) = Command::new(ardrive_path)
            .arg("list-files")
            .arg("--help")
            .output()
        {
            info!(
                "ArDrive list-files help: {}",
                String::from_utf8_lossy(&help.stdout)
            );
        }

        let mut cmd = Command::new(ardrive_path);
        // Use the working `list-drive` command with --all to enumerate files
        cmd.arg("list-drive")
            .arg("-d")
            .arg(&drive_id)
            .arg("--all")
            .arg("--wallet-file")
            .arg(&temp_dir)
            .env("NODE_ENV", "production");

        info!(
            "Executing: {} list-drive -d {} --all --wallet-file {}",
            ardrive_path.display(),
            drive_id,
            temp_dir.display()
        );

        cmd.output().context("Failed to execute local ardrive")?
    } else {
        info!("Local ardrive not found, trying system ardrive");

        // Verify ardrive is installed
        if let Err(e) = Command::new("ardrive").arg("--version").output() {
            return Err(anyhow!(
                "ArDrive CLI not found ({}). Install with: pnpm add ardrive-cli",
                e
            ));
        }

        let mut cmd = Command::new("ardrive");
        // Use `list-drive --all` to retrieve files for the drive
        cmd.arg("list-drive")
            .arg("-d")
            .arg(&drive_id)
            .arg("--all")
            .arg("--wallet-file")
            .arg(&temp_dir)
            .env("NODE_ENV", "production");

        info!(
            "Executing: ardrive list-drive -d {} --all --wallet-file {}",
            drive_id,
            temp_dir.display()
        );

        cmd.output().context("Failed to execute system ardrive")?
    };

    // Clean up temp file
    if let Err(e) = fs::remove_file(&temp_dir) {
        info!("Note: Failed to clean up temporary wallet file: {}", e);
    }

    // Process output
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Enhanced debug output
    info!(
        "ArDrive CLI command completed with status: {}",
        output.status
    );
    info!("ArDrive CLI stdout (raw): {:?}", stdout);
    if !stderr.is_empty() {
        info!("ArDrive CLI stderr (raw): {:?}", stderr);
    }

    if !output.status.success() {
        return Err(anyhow!(
            "ArDrive CLI command failed.\n\
             Exit Code: {}\n\
             Command output:\n{}\n{}\n\n\
             Common fixes:\n\
             1. Install ArDrive CLI: pnpm add ardrive-cli\n\
             2. Check your wallet file\n\
             3. Verify drive ID: {}\n\
             4. Try: sugar ardrive show-wallet",
            output.status.code().unwrap_or(-1),
            stderr.trim(),
            stdout.trim(),
            drive_id
        ));
    }

    // Pre-process output
    let clean_stdout = stdout.trim().trim_start_matches('\u{feff}');

    // First check for common error patterns
    if stdout.contains("Invalid entity ID") || stderr.contains("Invalid entity ID") {
        return Err(anyhow!(
            "Invalid drive ID '{}'. Please check that the drive ID exists and is accessible.",
            drive_id
        ));
    } else if clean_stdout.is_empty() {
        return Err(anyhow!(
            "ArDrive CLI returned empty output.\n\
             This usually means:\n\
             1. The drive is empty\n\
             2. You don't have access to this drive\n\
             3. The drive ID is incorrect\n\
             Drive ID: {}",
            drive_id
        ));
    }

    // Parse JSON with enhanced logging
    info!(
        "Attempting to parse JSON output (length: {}): {}",
        clean_stdout.len(),
        clean_stdout
    );
    // Try to detect common output patterns before parsing
    if clean_stdout.contains("Error:") || clean_stdout.contains("error:") {
        info!("Detected error message in output");
        return Err(anyhow!("ArDrive CLI error in output:\n{}", clean_stdout));
    }

    let val: Value = match serde_json::from_str(clean_stdout) {
        Ok(v) => {
            let pretty = serde_json::to_string_pretty(&v)
                .unwrap_or_else(|_| "<failed to pretty print>".to_string());
            info!("Successfully parsed JSON structure:\n{}", pretty);
            info!(
                "JSON type: {}",
                match &v {
                    serde_json::Value::Object(_) => "object",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Null => "null",
                }
            );
            v
        }
        Err(e) => {
            let line_num = e.line();
            let context = {
                let lines: Vec<_> = clean_stdout.lines().collect();
                if let Some(problem_line) = lines.get(line_num.saturating_sub(1)) {
                    format!(" (line {}: {})", line_num, problem_line)
                } else {
                    String::new()
                }
            };

            return Err(anyhow!(
                "ArDrive CLI returned invalid JSON{}.\n\
                 Error: {}\n\
                 Output: {}\n\n\
                 This usually means:\n\
                 1. The drive ID might be incorrect\n\
                 2. There might be permission issues\n\
                 3. The ArDrive CLI output format might have changed\n\
                 Try: sugar ardrive show-wallet",
                context,
                e,
                clean_stdout
            ));
        }
    };

    // Extract file list
    let items = extract_drives_from_value(&val).ok_or_else(|| {
        let structure = serde_json::to_string_pretty(&val)
            .unwrap_or_else(|_| "<cannot display JSON>".to_string());

        anyhow!(
            "Unexpected response format.\n\
             Expected: Array of file objects\n\
             Got: {}\n\n\
             This usually means:\n\
             1. The drive is empty\n\
             2. You don't have access\n\
             3. The drive ID is incorrect: {}",
            structure,
            drive_id
        )
    })?;

    // Parse items into ArDriveFile structs
    let mut files: Vec<ArDriveFile> = Vec::with_capacity(items.len());
    for item in items {
        let file = serde_json::from_value(item)
            .context("Failed to parse file data from ArDrive output")?;
        files.push(file);
    }

    // Apply extension filter if requested
    if let Some(ext) = filter_ext {
        files.retain(|f| {
            f.name
                .as_ref()
                .map(|n| n.ends_with(&format!(".{}", ext)))
                .unwrap_or(false)
        });
    }

    // Print summary and write output
    println!("Found {} files in drive {}", files.len(), drive_id);

    println!("Detailed files:");
    // Added an extra column for the Arweave link (derived from data tx or metadata tx)
    println!(
        "{:>3} | {:30} | {:>10} | {:>43} | {:>43} | {:64} | type",
        "idx", "name", "size", "data tx", "meta tx", "arweave"
    );
    println!(
        "{:-<3} | {:-<30} | {:-<10} | {:-<43} | {:-<43} | {:-<64} | {:-<20}",
        "", "", "", "", "", "", ""
    );

    for (i, f) in files.iter().enumerate() {
        let name = f.name.as_deref().unwrap_or("<unnamed>");
        let size = f
            .size
            .map(|s| s.to_string())
            .unwrap_or_else(|| "?".to_string());
        let data_tx = f.data_tx_id.as_deref().unwrap_or("");
        let meta_tx = f.metadata_tx_id.as_deref().unwrap_or("");

        // Derive an arweave URL from the preferred tx id (data tx preferred, then metadata)
        let arweave_url = if !data_tx.is_empty() {
            get_arweave_url(data_tx)
        } else if !meta_tx.is_empty() {
            get_arweave_url(meta_tx)
        } else {
            String::new()
        };

        // Detect content type from filename if not explicitly set
        let ctype = if let Some(ct) = f.content_type.as_deref().or(f.data_content_type.as_deref()) {
            ct.to_string()
        } else {
            // Try to infer from extension
            if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "json" => "application/json",
                    _ => "application/octet-stream",
                }
                .to_string()
            } else {
                "application/octet-stream".to_string()
            }
        };

        // Format size with units if available
        let size_fmt = if let Ok(size_num) = size.parse::<u64>() {
            if size_num > 1024 * 1024 {
                format!("{:.1}M", size_num as f64 / (1024.0 * 1024.0))
            } else if size_num > 1024 {
                format!("{:.1}K", size_num as f64 / 1024.0)
            } else {
                format!("{}B", size_num)
            }
        } else {
            size
        };

        println!(
            "{:>3} | {:30} | {:>10} | {:>43} | {:>43} | {:64} | {}",
            i,
            name,
            size_fmt,
            if data_tx.is_empty() { "-" } else { data_tx },
            if meta_tx.is_empty() { "-" } else { meta_tx },
            arweave_url,
            ctype
        );
    }

    if let Some(path) = output_path {
        fs::write(
            &path,
            serde_json::to_string_pretty(&files).context("Failed to format file list as JSON")?,
        )
        .with_context(|| format!("Failed to write file list to {}", path.display()))?;
        println!("✅ File list written to {}", path.display());
    }

    Ok(files)
}

/// Helper function to generate an Arweave URL from a transaction ID
pub fn get_arweave_url(tx_id: &str) -> String {
    format!("https://arweave.net/{}", tx_id)
}

pub fn process_ardrive_show_wallet(wallet: Option<PathBuf>) -> Result<()> {
    info!(
        "ArDrive: show-wallet called (wallet override: {:?})",
        wallet
    );

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
            content = fs::read_to_string(&cfg).map_err(|e| {
                anyhow::anyhow!("Failed reading stored wallet {}: {}", cfg.display(), e)
            })?;
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
            if let serde_json::Value::Object(map) = &val {
                println!(
                    "Top-level keys ({}): {}",
                    map.len(),
                    map.keys().cloned().collect::<Vec<_>>().join(", ")
                );
            }

            match serde_json::to_string_pretty(&val) {
                Ok(pretty) => println!("\n{}", pretty),
                Err(e) => println!(
                    "(failed to pretty-print JSON: {})\nRaw contents:\n{}",
                    e, content
                ),
            }
        }
        Err(_) => {
            println!(
                "(wallet content is not valid JSON)\nRaw contents:\n{}",
                content
            );
        }
    }

    Ok(())
}

/// Generate a candy-machine-style cache file from files stored in an ArDrive drive.
/// - `wallet` optional wallet override
/// - `drive_id` the drive to list
/// - `cache_file` path to write the cache JSON
/// - `candy_machine` optional candy machine pubkey (will populate program.candyMachine)
pub fn process_ardrive_generate_cache(
    wallet: Option<PathBuf>,
    drive_id: String,
    cache_file: PathBuf,
    candy_machine: Option<String>,
) -> Result<()> {
    use std::str::FromStr;

    use anchor_client::solana_sdk::pubkey::Pubkey;

    use crate::cache::{Cache, CacheItem, CacheProgram};

    info!(
        "Generating cache for drive {} -> {}",
        drive_id,
        cache_file.display()
    );

    // Reuse the existing listing function to fetch files
    let files = process_ardrive_list_drive_files(wallet, drive_id, None, None)
        .context("Failed to list drive files for cache generation")?;

    let mut cache = Cache::new();
    // If candy_machine provided, try to set program data
    if let Some(cm) = candy_machine {
        if let Ok(pk) = Pubkey::from_str(&cm) {
            cache.program = CacheProgram::new_from_cm(&pk);
        } else {
            info!("Invalid candy machine pubkey provided, leaving program defaults");
        }
    }

    // Fill items: key by numeric index (0-based as string) to be compatible with typical caches
    for (i, f) in files.iter().enumerate() {
        let key = i.to_string();
        let name = f.name.clone().unwrap_or_else(|| key.clone());
        let image_hash = f.data_tx_id.clone().unwrap_or_default();
        let image_link = if !image_hash.is_empty() {
            get_arweave_url(&image_hash)
        } else {
            String::new()
        };
        let metadata_hash = f.metadata_tx_id.clone().unwrap_or_default();
        let metadata_link = if !metadata_hash.is_empty() {
            get_arweave_url(&metadata_hash)
        } else {
            String::new()
        };

        let item = CacheItem {
            name: name.clone(),
            image_hash: image_hash.clone(),
            image_link,
            metadata_hash: metadata_hash.clone(),
            metadata_link,
            on_chain: false,
            animation_hash: None,
            animation_link: None,
        };

        cache.items.insert(key, item);
    }

    cache.file_path = cache_file.to_string_lossy().to_string();
    // avoid borrowing `cache` immutably while calling a mutable method
    let out_path = cache.file_path.clone();
    cache
        .write_to_file(&out_path)
        .with_context(|| format!("Failed to write cache to {}", cache_file.display()))?;

    println!(
        "✅ Wrote cache with {} items to {}",
        cache.items.len(),
        cache_file.display()
    );
    Ok(())
}

/// Wipe (delete) an existing cache file. Useful when switching Candy Machines.
pub fn process_ardrive_wipe_cache(cache_file: PathBuf) -> Result<()> {
    if cache_file.exists() {
        fs::remove_file(&cache_file)
            .with_context(|| format!("Failed to remove cache file {}", cache_file.display()))?;
        println!("✅ Removed cache file {}", cache_file.display());
    } else {
        println!("No cache file found at {}", cache_file.display());
    }

    Ok(())
}
