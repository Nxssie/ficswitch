use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::core::{backup, profiles, saves, steam};

pub fn run(target: &str, no_backup: bool) -> Result<()> {
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

    // Switch branch in manifest
    println!("{} Modifying appmanifest...", "⚙".cyan());
    steam::switch_branch(&manifest_path, &target_branch)?;
    println!("{} Branch set to {}", "✓".green(), target_branch.to_string().bold());

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
    println!(
        "{} Done! Start Steam to download the {} branch delta.",
        "✓".green().bold(),
        target_branch.to_string().bold()
    );

    Ok(())
}
