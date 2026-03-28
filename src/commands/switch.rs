use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::core::{backup, branch_cache, mod_deploy, profiles, saves, steam};

fn sync_out(save_dir: &std::path::Path, branch: &steam::Branch) {
    match profiles::profile_name_for_branch(branch) {
        Ok(Some(profile)) => match saves::sync_saves_out(save_dir, &profile) {
            Ok(n) => println!(
                "{} Saves synced to profile '{}' ({} files)",
                "✓".green(),
                profile.cyan(),
                n
            ),
            Err(e) => println!("{} Save sync failed: {}", "⚠".yellow(), e),
        },
        _ => {}
    }
}

fn get_cached_file_count(branch: &steam::Branch) -> Option<usize> {
    branch_cache::list_caches()
        .ok()
        .and_then(|caches| caches.into_iter().find(|c| c.branch == *branch))
        .map(|c| c.file_count)
}

pub fn run(
    target: &str,
    no_backup: bool,
    backend: &str,
    username: Option<&str>,
    ignore_cloud: bool,
    dry_run: bool,
) -> Result<()> {
    let target_branch = steam::Branch::from_str(target)?;

    // Find manifest
    let manifest_path = steam::find_manifest()?;
    let current_branch = steam::detect_branch(&manifest_path)?;

    if current_branch == target_branch {
        println!(
            "{} Already on {} branch.",
            "ℹ".blue(),
            target_branch.to_string().bold()
        );
        return Ok(());
    }

    if dry_run {
        println!(
            "{} [DRY RUN] Would switch from {} to {}",
            "ℹ".blue(),
            current_branch.to_string().bold(),
            target_branch.to_string().bold()
        );
        println!();

        // Show what would happen
        println!("Operations that would be performed:");
        println!("  1. Update appmanifest betakey to: {}", target_branch);

        if !no_backup {
            println!("  2. Create backup of current saves");
        }

        if steam::is_steam_cloud_active() {
            if ignore_cloud {
                println!("  3. Backup Steam Cloud data (--ignore-cloud)");
            } else {
                println!("  ⚠ Steam Cloud is active - may cause conflicts");
                println!("    Use --ignore-cloud to backup during switch");
            }
        }

        if let Ok(_install_dir) = steam::get_install_dir(&manifest_path) {
            if branch_cache::is_cached(&target_branch)? {
                let count_str = get_cached_file_count(&target_branch)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".to_string());
                println!(
                    "  {}. Restore {} files from cache",
                    if !no_backup { 4 } else { 3 },
                    count_str
                );
            } else {
                println!(
                    "  {}. Download {} via {} (no cache)",
                    if !no_backup { 4 } else { 3 },
                    target_branch,
                    if backend == "steamcmd" {
                        "SteamCMD"
                    } else {
                        "Steam"
                    }
                );
                if backend == "steamcmd" && username.is_none() {
                    println!("  ⚠ --username required for SteamCMD");
                }
            }

            if profiles::profile_name_for_branch(&target_branch)?.is_some() {
                println!(
                    "  {}. Activate SMM profile for {}",
                    if !no_backup { 5 } else { 4 },
                    target_branch
                );
            }
        }

        if ignore_cloud && steam::is_steam_cloud_active() {
            println!(
                "  {}. Restore Steam Cloud backup",
                if !no_backup { 6 } else { 5 }
            );
        }

        println!();
        println!("Run without --dry-run to execute.");
        return Ok(());
    }

    println!(
        "{} Switching from {} to {}",
        "→".cyan(),
        current_branch.to_string().bold(),
        target_branch.to_string().bold()
    );

    // Check Steam is not running
    if steam::is_steam_running() {
        return Err(anyhow!(
            "Steam is currently running. Please close Steam before switching branches."
        ));
    }

    // Steam Cloud handling
    let cloud_backed_up = if steam::is_steam_cloud_active() {
        if ignore_cloud {
            println!("{} Backing up Steam Cloud data...", "☁".cyan());
            match steam::backup_steam_cloud() {
                Ok(backup_path) => {
                    println!(
                        "{} Steam Cloud backed up to: {}",
                        "✓".green(),
                        backup_path.display()
                    );
                    true
                }
                Err(e) => {
                    println!("{} Failed to backup Steam Cloud: {}", "⚠".yellow(), e);
                    println!("{} Continuing without Steam Cloud backup...", "ℹ".blue());
                    false
                }
            }
        } else {
            println!(
                "{} {}",
                "⚠".yellow(),
                "Steam Cloud is active for Satisfactory.".bold()
            );
            println!("  This may cause save conflicts when switching branches.");
            println!("  Use --ignore-cloud to temporarily backup Steam Cloud data.");
            println!();
            false
        }
    } else {
        false
    };

    // Sync current profile's saves out before switching
    if let Ok(save_dir) = saves::find_save_dir() {
        sync_out(&save_dir, &current_branch);
    }

    // Backup saves
    if !no_backup {
        println!("{} Creating backup of saves...", "📦".cyan());
        match saves::find_save_dir() {
            Ok(save_dir) => {
                let label = format!("Auto-backup before switch to {}", target_branch);
                match backup::create_backup(&save_dir, &current_branch, Some(&label)) {
                    Ok(manifest) => {
                        println!(
                            "{} Backup created: {} ({} saves)",
                            "✓".green(),
                            manifest.id,
                            manifest.save_count
                        );
                    }
                    Err(e) => {
                        println!(
                            "{} Backup failed: {} (continuing without backup)",
                            "⚠".yellow(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                println!(
                    "{} Could not find saves: {} (continuing without backup)",
                    "⚠".yellow(),
                    e
                );
            }
        }
    }

    // Restore game files and appmanifest from cache if available, otherwise just update betakey
    let used_cache = match (
        steam::get_install_dir(&manifest_path),
        branch_cache::is_cached(&target_branch),
    ) {
        (Ok(install_dir), Ok(true)) => {
            println!(
                "{} Restoring {} from cache...",
                "⚙".cyan(),
                target_branch.to_string().bold()
            );
            match branch_cache::restore_branch(&install_dir, &manifest_path, &target_branch) {
                Ok(count) => {
                    println!("{} Restored {} files from cache", "✓".green(), count);
                    true
                }
                Err(e) => {
                    println!(
                        "{} Cache restore failed: {} (falling back to Steam download)",
                        "⚠".yellow(),
                        e
                    );
                    steam::switch_branch(&manifest_path, &target_branch)?;
                    println!(
                        "{} Branch set to {}",
                        "✓".green(),
                        target_branch.to_string().bold()
                    );
                    false
                }
            }
        }
        _ => {
            steam::switch_branch(&manifest_path, &target_branch)?;

            if backend == "steamcmd" {
                let user = username.ok_or_else(|| {
                    anyhow!("--username <steam_user> is required with --backend steamcmd")
                })?;
                let install_dir = steam::get_install_dir(&manifest_path)?;
                println!(
                    "{} Downloading {} via SteamCMD...",
                    "⬇".cyan(),
                    target_branch.to_string().bold()
                );
                steam::download_with_steamcmd(&target_branch, user, &install_dir)?;
                // Deploy mods and cache
                if let Ok(result) = mod_deploy::deploy_mods(&install_dir, &target_branch) {
                    if result.sml_deployed {
                        println!("{} SML deployed", "✓".green());
                    }
                    for name in &result.mods_deployed {
                        println!("{} Mod deployed: {}", "✓".green(), name.cyan());
                    }
                }
                println!(
                    "Caching {} branch game files...",
                    target_branch.to_string().bold()
                );
                match branch_cache::cache_branch(&install_dir, &manifest_path, &target_branch) {
                    Ok(count) => println!(
                        "{} Cached {} branch: {} files hardlinked",
                        "✓".green(),
                        target_branch,
                        count
                    ),
                    Err(e) => println!("{} Cache failed: {}", "⚠".yellow(), e),
                }
                return Ok(());
            }

            println!(
                "{} Launching Steam to download {}...",
                "⬇".cyan(),
                target_branch.to_string().bold()
            );

            match steam::launch_steam() {
                Ok(()) => {
                    if let Err(e) = steam::wait_for_download(&manifest_path, &target_branch) {
                        println!("{} Error monitoring download: {}", "⚠".yellow(), e);
                        false
                    } else {
                        println!(
                            "{} Download complete. Close Steam and ficswitch will cache automatically.",
                            "✓".green()
                        );
                        steam::wait_for_steam_close();

                        // Deploy mods and cache
                        match steam::get_install_dir(&manifest_path) {
                            Ok(install_dir) => {
                                if let Ok(result) =
                                    mod_deploy::deploy_mods(&install_dir, &target_branch)
                                {
                                    if result.sml_deployed {
                                        println!("{} SML deployed", "✓".green());
                                    }
                                    for name in &result.mods_deployed {
                                        println!("{} Mod deployed: {}", "✓".green(), name.cyan());
                                    }
                                }
                                println!(
                                    "Caching {} branch game files...",
                                    target_branch.to_string().bold()
                                );
                                match branch_cache::cache_branch(
                                    &install_dir,
                                    &manifest_path,
                                    &target_branch,
                                ) {
                                    Ok(count) => println!(
                                        "{} Cached {} branch: {} files hardlinked",
                                        "✓".green(),
                                        target_branch,
                                        count
                                    ),
                                    Err(e) => println!("{} Cache failed: {}", "⚠".yellow(), e),
                                }
                                true
                            }
                            Err(e) => {
                                println!("{} Could not determine install dir: {}", "⚠".yellow(), e);
                                false
                            }
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "{} Could not launch Steam ({}). Open Steam manually to download {}.",
                        "⚠".yellow(),
                        e,
                        target_branch.to_string().bold()
                    );
                    false
                }
            }
        }
    };

    // Activate SMM profile
    match steam::get_install_dir(&manifest_path) {
        Ok(install_dir) => {
            match profiles::activate_profile_for_branch(&target_branch, &install_dir) {
                Ok(Some(profile_name)) => {
                    println!(
                        "{} SMM profile activated: {}",
                        "✓".green(),
                        profile_name.cyan()
                    );
                }
                Ok(None) => {
                    println!(
                        "{} No SMM profile linked to {} branch",
                        "ℹ".blue(),
                        target_branch
                    );
                }
                Err(e) => {
                    println!("{} Could not activate SMM profile: {}", "⚠".yellow(), e);
                }
            }
        }
        Err(e) => {
            println!(
                "{} Could not determine install dir for SMM: {}",
                "⚠".yellow(),
                e
            );
        }
    }

    // Restore Steam Cloud if we backed it up
    if cloud_backed_up {
        println!("{} Restoring Steam Cloud data...", "☁".cyan());
        if let Err(e) = steam::restore_steam_cloud() {
            println!("{} Failed to restore Steam Cloud: {}", "⚠".yellow(), e);
        } else {
            println!("{} Steam Cloud restored", "✓".green());
        }
    }

    println!();
    if used_cache {
        println!(
            "{} Done! Launch Satisfactory directly — no Steam download needed.",
            "✓".green().bold()
        );
    } else {
        println!(
            "{} Done! Start Steam to download the {} branch delta.",
            "✓".green().bold(),
            target_branch.to_string().bold()
        );
    }

    Ok(())
}
