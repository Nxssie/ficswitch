use anyhow::Result;
use colored::Colorize;

use crate::core::steam;

pub fn status() -> Result<()> {
    println!("{}", "=== Steam Cloud Status ===".bold());
    println!();

    match steam::find_cloud_remote_dir() {
        Ok(remote_dir) => {
            println!("{} {}", "Cloud directory:".bold(), remote_dir.display());

            let files_count = std::fs::read_dir(&remote_dir)
                .map(|entries| entries.count())
                .unwrap_or(0);

            println!(
                "{} {} {}",
                "Status:".bold(),
                "active".yellow(),
                format!("({} files)", files_count).dimmed()
            );

            if steam::has_cloud_backup() {
                println!("{} {}", "Ficswitch backup:".bold(), "exists".green());
            } else {
                println!("{} {}", "Ficswitch backup:".bold(), "none".dimmed());
            }
        }
        Err(_) => {
            if steam::has_cloud_backup() {
                println!(
                    "{} {}",
                    "Status:".bold(),
                    "inactive (backup exists)".dimmed()
                );
                println!("  Run 'ficswitch cloud restore' to restore your Steam Cloud data.");
            } else {
                println!("{} {}", "Status:".bold(), "inactive".green());
                println!("  No Steam Cloud data found for Satisfactory.");
            }
        }
    }

    Ok(())
}

pub fn backup(dry_run: bool) -> Result<()> {
    if !steam::is_steam_cloud_active() {
        println!("{} Steam Cloud is not active for Satisfactory.", "ℹ".blue());
        return Ok(());
    }

    if steam::has_cloud_backup() {
        println!(
            "{} A backup already exists. Clear it first with 'ficswitch cloud clear'.",
            "⚠".yellow()
        );
        return Ok(());
    }

    if dry_run {
        println!("{} [DRY RUN] Would backup Steam Cloud data", "ℹ".blue());
        if let Ok(remote_dir) = steam::find_cloud_remote_dir() {
            if let Ok(backup_path) = steam::get_cloud_backup_path() {
                println!("  {} -> {}", remote_dir.display(), backup_path.display());
            }
        }
        return Ok(());
    }

    match steam::backup_steam_cloud() {
        Ok(backup_path) => {
            println!("{} Steam Cloud backed up to:", "✓".green());
            println!("  {}", backup_path.display());
            println!();
            println!(
                "{} Steam Cloud sync is now disabled for Satisfactory.",
                "ℹ".blue()
            );
            println!("  Your local saves will not be synced with Steam Cloud.");
            println!("  Run 'ficswitch cloud restore' to re-enable Steam Cloud sync.");
        }
        Err(e) => {
            println!("{} Failed to backup Steam Cloud: {}", "✗".red(), e);
        }
    }

    Ok(())
}

pub fn restore(dry_run: bool) -> Result<()> {
    if !steam::has_cloud_backup() {
        println!("{} No Steam Cloud backup found.", "ℹ".blue());
        return Ok(());
    }

    if dry_run {
        println!("{} [DRY RUN] Would restore Steam Cloud data", "ℹ".blue());
        if let Ok(backup_path) = steam::get_cloud_backup_path() {
            if let Ok(remote_dir) = steam::find_cloud_remote_dir() {
                println!("  {} -> {}", backup_path.display(), remote_dir.display());
            } else {
                println!("  From: {}", backup_path.display());
            }
        }
        return Ok(());
    }

    match steam::restore_steam_cloud() {
        Ok(()) => {
            println!("{} Steam Cloud data restored.", "✓".green());
            println!();
            println!("{} Steam Cloud sync is now re-enabled.", "ℹ".blue());
            println!("  Steam may sync your saves on next launch.");
        }
        Err(e) => {
            println!("{} Failed to restore Steam Cloud: {}", "✗".red(), e);
        }
    }

    Ok(())
}

pub fn clear(dry_run: bool) -> Result<()> {
    if !steam::has_cloud_backup() {
        println!("{} No backup to clear.", "ℹ".blue());
        return Ok(());
    }

    let backup_path = steam::get_cloud_backup_path()?;

    if dry_run {
        println!(
            "{} [DRY RUN] Would delete: {}",
            "ℹ".blue(),
            backup_path.display()
        );
        return Ok(());
    }

    match std::fs::remove_dir_all(&backup_path) {
        Ok(()) => {
            println!("{} Steam Cloud backup cleared.", "✓".green());
        }
        Err(e) => {
            println!("{} Failed to clear backup: {}", "✗".red(), e);
        }
    }

    Ok(())
}
