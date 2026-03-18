use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::core::{backup as backup_core, saves, steam};

pub fn create(label: Option<&str>) -> Result<()> {
    let manifest_path = steam::find_manifest()?;
    let branch = steam::detect_branch(&manifest_path)?;
    let save_dir = saves::find_save_dir()?;

    println!("{} Creating backup...", "📦".cyan());
    let manifest = backup_core::create_backup(&save_dir, &branch, label)?;

    println!("{} Backup created successfully!", "✓".green());
    println!("  ID:         {}", manifest.id);
    println!("  Branch:     {}", manifest.branch);
    println!("  Saves:      {}", manifest.save_count);
    println!("  Blueprints: {}", manifest.blueprint_count);
    if let Some(label) = &manifest.label {
        println!("  Label:      {}", label);
    }
    println!("  Timestamp:  {}", manifest.timestamp);

    Ok(())
}

pub fn list() -> Result<()> {
    let backups = backup_core::list_backups()?;

    if backups.is_empty() {
        println!("{} No backups found.", "ℹ".blue());
        return Ok(());
    }

    println!("{}", "=== Backups ===".bold());
    println!();

    for backup in &backups {
        let label_str = backup
            .label
            .as_deref()
            .unwrap_or("(no label)");

        println!(
            "  {} [{}] {} saves, {} blueprints - {}",
            backup.id.bold(),
            backup.branch.cyan(),
            backup.save_count,
            backup.blueprint_count,
            label_str.dimmed()
        );
    }

    println!();
    println!("Total: {} backups", backups.len());

    Ok(())
}

pub fn restore(id: &str) -> Result<()> {
    let backups = backup_core::list_backups()?;
    let backup = backups
        .iter()
        .find(|b| b.id == id)
        .ok_or_else(|| anyhow!("Backup '{}' not found", id))?;

    let save_dir = saves::find_save_dir()?;

    println!(
        "{} Restoring backup {} ({} branch, {} saves)...",
        "📦".cyan(),
        backup.id.bold(),
        backup.branch,
        backup.save_count
    );

    backup_core::restore_backup(id, &save_dir)?;

    println!("{} Backup restored successfully!", "✓".green());

    Ok(())
}
