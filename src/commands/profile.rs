use anyhow::Result;
use colored::Colorize;

use crate::core::{profiles, steam};

pub fn list() -> Result<()> {
    let smm_profiles = profiles::read_smm_profiles()?;

    if smm_profiles.profiles.is_empty() {
        println!(
            "{} No SMM profiles found. Is Satisfactory Mod Manager installed?",
            "ℹ".blue()
        );
        return Ok(());
    }

    println!("{}", "=== SMM Profiles ===".bold());
    println!();

    for (name, mods) in &smm_profiles.profiles {
        let enabled_count = mods.values().filter(|m| m.enabled).count();
        println!(
            "  {} ({} mods, {} enabled)",
            name.bold(),
            mods.len(),
            enabled_count
        );
    }

    Ok(())
}

pub fn link(profile_name: &str, branch_name: &str) -> Result<()> {
    let branch = steam::Branch::from_str(branch_name)?;

    profiles::link_profile(profile_name, &branch)?;

    println!(
        "{} Linked profile '{}' to {} branch",
        "✓".green(),
        profile_name.cyan(),
        branch.to_string().bold()
    );

    Ok(())
}

pub fn show() -> Result<()> {
    let branch_profiles = profiles::read_branch_profiles()?;

    if branch_profiles.mappings.is_empty() {
        println!(
            "{} No profile-branch mappings configured.",
            "ℹ".blue()
        );
        println!(
            "Use {} to link a profile to a branch.",
            "ficswitch profile link <name> <branch>".dimmed()
        );
        return Ok(());
    }

    println!("{}", "=== Branch → Profile Mappings ===".bold());
    println!();

    for (branch, profile) in &branch_profiles.mappings {
        println!("  {} → {}", branch.bold(), profile.cyan());
    }

    Ok(())
}
