use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::cache::{Cache, CacheItem, CacheItems, CacheProgram};
use anyhow::{anyhow, Result};

/// Processes a list of Arweave metadata links and generates a sugar-style cache.json
pub fn process_import(input_file: &Path, output_file: &Path) -> Result<()> {
    // Open the input file
    let file = File::open(input_file)
        .map_err(|e| anyhow!("Failed to open input file: {}", e))?;
    let reader = BufReader::new(file);

    // Temporary map to collect CacheItems
    let mut items_map: HashMap<String, CacheItem> = HashMap::new();

    for (index, line_result) in reader.lines().enumerate() {
        let metadata_link = line_result
            .map_err(|e| anyhow!("Failed to read line {}: {}", index + 1, e))?;
        if metadata_link.trim().is_empty() {
            continue;
        }

        let name = format!("NFT #{}", index + 1);

        items_map.insert(
            index.to_string(),
            CacheItem {
                name,
                image_hash: String::new(),
                image_link: String::new(),
                metadata_hash: String::new(),
                metadata_link,
                on_chain: false,
                animation_hash: None,
                animation_link: None,
            },
        );
    }

    // Convert HashMap into CacheItems (IndexMap wrapper)
    let mut cache_items = CacheItems::new();
    for (k, v) in items_map {
        cache_items.insert(k, v);
    }

    // Build the final Cache (mutable for writing)
    let mut cache = Cache {
        program: CacheProgram::new(),
        items: cache_items,
        file_path: output_file.to_string_lossy().to_string(),
    };

    // Write cache to file
    cache
        .write_to_file(output_file)
        .map_err(|e| anyhow!("Failed to write cache file: {}", e))?;

    println!(
        "✅ Imported {} NFTs into cache: {:?}",
        cache.items.len(),
        output_file
    );

    Ok(())
}
