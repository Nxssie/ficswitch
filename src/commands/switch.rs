use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::core::{backup, branch_cache, mod_deploy, profiles, saves, steam};

fn sync_out(save_dir: &std::path::Path, branch: &steam::Branch) {
    match profiles::profile_name_for_branch(branch) {
        Ok(Some(profile)) => {
            match saves::sync_saves_out(save_dir, &profile) {
                Ok(n) => println!(
                    "{} Saves synced to profile '{}' ({} files)",
                    "✓".green(),
                    profile.cyan(),
                    n
                ),
                Err(e) => println!("{} Save sync failed: {}", "⚠".yellow(), e),
            }
        }
        _ => {}
    }
}


pub fn run(target: &str, no_backup: bool, backend: &str, username: Option<&str>) -> Result<()> {
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

    // Sync current profile's saves out before switching
    if let Ok(save_dir) = saves::find_save_dir() {
        sync_out(&save_dir, &current_branch);
    }

    // Backup saves
    if !no_backup {
        println!("{} Creating backup of saves...", "📦".cyan());
        match saves::find_save_dir() {
            Ok(save_dir) => {
                let label = format!(
                    "Auto-backup before switch to {}",
                    target_branch
                );
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
    let used_cache = match (steam::get_install_dir(&manifest_path), branch_cache::is_cached(&target_branch)) {
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
                    println!("{} Branch set to {}", "✓".green(), target_branch.to_string().bold());
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
                println!("Caching {} branch game files...", target_branch.to_string().bold());
                match branch_cache::cache_branch(&install_dir, &manifest_path, &target_branch) {
                    Ok(count) => println!(
                        "{} Cached {} branch: {} files hardlinked",
                        "✓".green(), target_branch, count
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
                    println!(
                        "{} Could not activate SMM profile: {}",
                        "⚠".yellow(),
                        e
                    );
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
