use std::fs;
use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};

use super::ImportNFTsArgs;

/// Process the `import-nfts` command.
/// Reads a list of metadata URLs, fetches each JSON,
/// and generates a Candy Machine `cache.json`.
pub fn process_import_nfts(args: ImportNFTsArgs) -> Result<()> {
    let data = fs::read_to_string(&args.input)
        .map_err(|e| anyhow!("Failed to read input file {:?}: {}", args.input, e))?;

    let client = Client::new();
    let mut entries = serde_json::Map::new();

    for (i, line) in data.lines().enumerate() {
        let url = line.trim();
        if url.is_empty() {
            continue;
        }

        println!("📦 Fetching metadata #{} from {}", i, url);

        let resp = client
            .get(url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch {}: {}", url, e))?;

        if !resp.status().is_success() {
            eprintln!("⚠️ Skipping {} (HTTP {})", url, resp.status());
            continue;
        }

        let meta: Value = resp.json()
            .map_err(|e| anyhow!("Invalid JSON at {}: {}", url, e))?;

        let name = meta
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed");

        let image_link = meta
            .get("image")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        entries.insert(
            i.to_string(),
            json!({
                "name": name,
                "image_link": image_link,
                "metadata_link": url,
                "onChain": false
            }),
        );
    }

    let cache = json!({
        "program": {
            "candyMachine": null,
            "candyMachineCreator": null,
            "collectionMint": null
        },
        "items": entries
    });

    fs::write(&args.output, serde_json::to_string_pretty(&cache)?)
        .map_err(|e| anyhow!("Failed to write {:?}: {}", args.output, e))?;

    println!("✅ Successfully generated cache file at {:?}", args.output);
    Ok(())
}
