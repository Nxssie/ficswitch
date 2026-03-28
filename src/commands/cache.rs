use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::core::{branch_cache, mod_deploy, steam};

pub fn create(dry_run: bool) -> Result<()> {
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

    if dry_run {
        println!("{} [DRY RUN] Would cache {} branch", "ℹ".blue(), branch);
        println!("  Game dir: {}", game_dir.display());
        println!("  Manifest: {}", manifest.display());

        let caches = branch_cache::list_caches()?;
        let current_cached = caches.iter().find(|c| c.branch == branch);
        if let Some(info) = current_cached {
            println!("  Existing cache: {} files", info.file_count);
        }

        return Ok(());
    }

    // Deploy SMM mods for this branch before caching so they are included.
    match mod_deploy::deploy_mods(&game_dir, &branch) {
        Ok(result) => {
            if result.sml_deployed || !result.mods_deployed.is_empty() {
                if result.sml_deployed {
                    println!("{} SML deployed", "✓".green());
                }
                for name in &result.mods_deployed {
                    println!("{} Mod deployed: {}", "✓".green(), name.cyan());
                }
            }
            if !result.mods_missing.is_empty() {
                for name in &result.mods_missing {
                    println!(
                        "{} Mod zip not found in SMM cache, skipping: {}",
                        "⚠".yellow(),
                        name
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{} Could not deploy mods ({}), caching without mods",
                "⚠".yellow(),
                e
            );
        }
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

pub fn clear(branch_name: &str, dry_run: bool) -> Result<()> {
    let branch = steam::Branch::from_str(branch_name)?;

    if dry_run {
        println!(
            "{} [DRY RUN] Would clear cache for {} branch",
            "ℹ".blue(),
            branch
        );
        let caches = branch_cache::list_caches()?;
        if let Some(info) = caches.iter().find(|c| c.branch == branch) {
            println!("  Files to remove: {}", info.file_count);
        }
        return Ok(());
    }

    branch_cache::clear_cache(&branch)?;
    println!(
        "{} Cache cleared for {} branch.",
        "✓".green(),
        branch.to_string().bold()
    );
    Ok(())
}
