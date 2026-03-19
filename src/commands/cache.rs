use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::core::{branch_cache, steam};

pub fn create() -> Result<()> {
    let manifest = steam::find_manifest()?;
    let branch = steam::detect_branch(&manifest)?;
    let game_dir = steam::get_install_dir(&manifest)?;

    if steam::is_steam_running() {
        return Err(anyhow!(
            "Steam is running. Close Steam before caching to ensure a clean, complete installation."
        ));
    }

    if steam::is_download_pending(&manifest)? {
        return Err(anyhow!(
            "A download or update is pending for this branch. \
             Let Steam finish the download, then close Steam and run cache create again."
        ));
    }

    println!(
        "Caching {} branch game files (this may take a while)...",
        branch.to_string().bold()
    );

    let count = branch_cache::cache_branch(&game_dir, &manifest, &branch)?;

    println!(
        "{} Cached {} branch: {} files hardlinked",
        "✓".green(),
        branch.to_string().bold(),
        count
    );

    Ok(())
}

pub fn status() -> Result<()> {
    let caches = branch_cache::list_caches()?;

    if caches.is_empty() {
        println!("{} No branch caches found.", "ℹ".blue());
        println!(
            "Use {} to cache the current branch.",
            "ficswitch cache create".dimmed()
        );
        return Ok(());
    }

    println!("{}", "=== Branch Cache ===".bold());
    println!();

    for info in caches {
        println!(
            "  {} — {} files",
            info.branch.to_string().bold(),
            info.file_count
        );
    }

    Ok(())
}

pub fn clear(branch_name: &str) -> Result<()> {
    let branch = steam::Branch::from_str(branch_name)?;
    branch_cache::clear_cache(&branch)?;
    println!(
        "{} Cache cleared for {} branch.",
        "✓".green(),
        branch.to_string().bold()
    );
    Ok(())
}
