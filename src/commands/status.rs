use anyhow::Result;
use colored::Colorize;

use crate::core::{profiles, saves, steam};

pub fn run() -> Result<()> {
    println!("{}", "=== Satisfactory Branch Switcher ===".bold());
    println!();

    // Find and show manifest info
    match steam::find_manifest() {
        Ok(manifest_path) => {
            println!("{} {}", "Manifest:".bold(), manifest_path.display());

            match steam::detect_branch(&manifest_path) {
                Ok(branch) => {
                    let branch_display = match &branch {
                        steam::Branch::Stable => "stable".green(),
                        steam::Branch::Experimental => "experimental".yellow(),
                    };
                    println!("{} {}", "Current branch:".bold(), branch_display);
                }
                Err(e) => println!("{} {}", "Branch detection:".bold(), e.to_string().red()),
            }

            match steam::get_install_dir(&manifest_path) {
                Ok(dir) => println!("{} {}", "Install dir:".bold(), dir.display()),
                Err(e) => println!("{} {}", "Install dir:".bold(), e.to_string().red()),
            }
        }
        Err(e) => {
            println!("{} {}", "Satisfactory not found:".red().bold(), e);
        }
    }

    println!();

    // Steam status
    if steam::is_steam_running() {
        println!("{} {}", "Steam:".bold(), "running".yellow());
    } else {
        println!("{} {}", "Steam:".bold(), "not running".green());
    }

    // Steam Cloud status
    match steam::is_steam_cloud_active() {
        true => {
            let backup_status = if steam::has_cloud_backup() {
                " (backup exists)".dimmed()
            } else {
                "".clear()
            };
            println!(
                "{} {}{}",
                "Steam Cloud:".bold(),
                "active".yellow(),
                backup_status
            );
        }
        false => {
            if steam::has_cloud_backup() {
                println!(
                    "{} {}",
                    "Steam Cloud:".bold(),
                    "inactive (backup exists)".dimmed()
                );
            } else {
                println!("{} {}", "Steam Cloud:".bold(), "inactive".green());
            }
        }
    }

    println!();

    // Save info
    match saves::find_save_dir() {
        Ok(save_dir) => {
            println!("{} {}", "Save directory:".bold(), save_dir.display());
            match saves::list_saves(&save_dir) {
                Ok(save_list) => {
                    println!("{} {}", "Save files:".bold(), save_list.len());
                    for save in &save_list {
                        if let Some(name) = save.file_name() {
                            let header_info = match saves::parse_save_header(save) {
                                Ok(h) => format!(
                                    "(header v{}, save v{}, build {})",
                                    h.header_version, h.save_version, h.build_version
                                ),
                                Err(_) => "(unable to read header)".to_string(),
                            };
                            println!("  - {} {}", name.to_string_lossy(), header_info.dimmed());
                        }
                    }
                }
                Err(e) => println!("{} {}", "Saves:".bold(), e.to_string().red()),
            }
        }
        Err(e) => println!("{} {}", "Save directory:".bold(), e.to_string().red()),
    }

    println!();

    // Profile mappings
    match profiles::read_branch_profiles() {
        Ok(bp) => {
            if bp.mappings.is_empty() {
                println!(
                    "{} {}",
                    "Profile mappings:".bold(),
                    "none configured".dimmed()
                );
            } else {
                println!("{}", "Profile mappings:".bold());
                for (branch, profile) in &bp.mappings {
                    println!("  {} → {}", branch, profile.cyan());
                }
            }
        }
        Err(e) => println!("{} {}", "Profiles:".bold(), e.to_string().red()),
    }

    Ok(())
}
