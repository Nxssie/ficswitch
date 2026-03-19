use ficswitch::commands::profile as profile_cmd;
use ficswitch::core::{backup, profiles, steam, steam::Branch};
use std::fs;
use std::path::{Path, PathBuf};

fn test_save_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("ficswitch_tests")
        .join(test_name);
    fs::create_dir_all(&dir).expect("failed to create test dir");
    dir
}

fn create_sav_file(dir: &PathBuf, name: &str) {
    let path = dir.join(name);
    let mut data = Vec::new();
    data.extend_from_slice(&13i32.to_le_bytes());
    data.extend_from_slice(&46i32.to_le_bytes());
    data.extend_from_slice(&264901i32.to_le_bytes());
    fs::write(&path, &data).expect("failed to create .sav file");
}

#[test]
fn test_backup_create_produces_manifest() {
    let save_dir = test_save_dir("backup_create");
    create_sav_file(&save_dir, "save1.sav");
    create_sav_file(&save_dir, "save2.sav");

    let result = backup::create_backup(&save_dir, &Branch::Stable, Some("test-create"));
    assert!(result.is_ok(), "create_backup failed: {:?}", result.err());

    let manifest = result.unwrap();
    assert_eq!(manifest.save_count, 2, "expected 2 saves");
    assert_eq!(manifest.branch, "stable");

    // ID must match YYYYMMDD_HHMMSS format (15 chars, underscore at position 8)
    assert_eq!(manifest.id.len(), 15, "id should be 15 chars");
    assert_eq!(&manifest.id[8..9], "_", "id should have underscore at position 8");

    let backup_root = backup::backup_root().expect("backup_root failed");
    let manifest_path = backup_root.join(&manifest.id).join("manifest.json");
    assert!(
        manifest_path.exists(),
        "manifest.json not found at: {}",
        manifest_path.display()
    );

    let _ = fs::remove_dir_all(&save_dir);
}

#[test]
fn test_backup_list_empty() {
    // Cannot inject backup_root path, so just assert no panic when dir may or may not exist.
    let result = backup::list_backups();
    assert!(result.is_ok(), "list_backups returned Err: {:?}", result.err());
}

#[test]
fn test_backup_list_after_create() {
    // Sleep 1s to ensure a unique timestamp ID separate from other parallel tests.
    std::thread::sleep(std::time::Duration::from_secs(1));

    let save_dir = test_save_dir("backup_list_after_create");
    create_sav_file(&save_dir, "a.sav");

    let manifest = backup::create_backup(&save_dir, &Branch::Experimental, Some("test-list"))
        .expect("create_backup failed");

    let backups = backup::list_backups().expect("list_backups failed");
    let found = backups.iter().any(|b| b.id == manifest.id);
    assert!(found, "created backup id '{}' not found in list", manifest.id);

    let _ = fs::remove_dir_all(&save_dir);
}

#[test]
fn test_backup_restore() {
    let save_dir = test_save_dir("backup_restore_src");
    create_sav_file(&save_dir, "world.sav");
    create_sav_file(&save_dir, "other.sav");

    let manifest = backup::create_backup(&save_dir, &Branch::Stable, Some("test-restore"))
        .expect("create_backup failed");

    let restore_dir = test_save_dir("backup_restore_dest");
    backup::restore_backup(&manifest.id, &restore_dir).expect("restore_backup failed");

    assert!(restore_dir.join("world.sav").exists(), "world.sav not restored");
    assert!(restore_dir.join("other.sav").exists(), "other.sav not restored");

    let _ = fs::remove_dir_all(&save_dir);
    let _ = fs::remove_dir_all(&restore_dir);
}

// --- ACF trailing newline regression tests ---

const SAMPLE_ACF_WITH_NEWLINE: &str = "\"AppState\"\n{\n\t\"appid\"\t\t\"526870\"\n\t\"UserConfig\"\n\t{\n\t\t\"betakey\"\t\t\"experimental\"\n\t}\n\t\"MountedConfig\"\n\t{\n\t\t\"betakey\"\t\t\"experimental\"\n\t}\n}\n";
const SAMPLE_ACF_WITHOUT_NEWLINE: &str = "\"AppState\"\n{\n\t\"appid\"\t\t\"526870\"\n\t\"UserConfig\"\n\t{\n\t\t\"betakey\"\t\t\"experimental\"\n\t}\n\t\"MountedConfig\"\n\t{\n\t\t\"betakey\"\t\t\"experimental\"\n\t}\n}";

fn write_temp_acf(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let path = dir.join(filename);
    fs::write(&path, content).expect("failed to write temp ACF");
    path
}

#[test]
fn test_acf_trailing_newline_preserved() {
    let dir = std::env::temp_dir().join("ficswitch_test_acf_newline");
    fs::create_dir_all(&dir).unwrap();

    let path = write_temp_acf(&dir, "appmanifest_526870.acf", SAMPLE_ACF_WITH_NEWLINE);
    steam::switch_branch(&path, &steam::Branch::Stable).expect("switch_branch failed");

    let result = fs::read_to_string(&path).expect("failed to read result");
    assert!(
        result.ends_with('\n'),
        "Expected trailing newline to be preserved; last 10 chars: {:?}",
        &result[result.len().saturating_sub(10)..]
    );

    fs::remove_file(&path).ok();
    fs::remove_dir(&dir).ok();
}

#[test]
fn test_acf_no_trailing_newline_not_added() {
    let dir = std::env::temp_dir().join("ficswitch_test_acf_no_newline");
    fs::create_dir_all(&dir).unwrap();

    let path = write_temp_acf(&dir, "appmanifest_526870.acf", SAMPLE_ACF_WITHOUT_NEWLINE);
    steam::switch_branch(&path, &steam::Branch::Stable).expect("switch_branch failed");

    let result = fs::read_to_string(&path).expect("failed to read result");
    assert!(
        !result.ends_with('\n'),
        "Expected no trailing newline; last 10 chars: {:?}",
        &result[result.len().saturating_sub(10)..]
    );

    fs::remove_file(&path).ok();
    fs::remove_dir(&dir).ok();
}

// --- Profile link ERR-03 regression test ---

/// When SMM is not installed (no profiles.json or it is empty), profile link
/// must return Ok(()) instead of propagating the error (satisfies ERR-03).
/// Skips automatically if SMM is installed or profiles.json fails to parse.
#[test]
fn test_profile_link_no_smm() {
    match profiles::read_smm_profiles() {
        Err(_) => {
            // Parse error means SMM is installed but in an unexpected format.
            // This machine has SMM; cannot test the no-SMM path here.
            eprintln!("Skipping test_profile_link_no_smm: SMM config parse error (SMM installed)");
            return;
        }
        Ok(smm) if !smm.profiles.is_empty() => {
            eprintln!("Skipping test_profile_link_no_smm: SMM profiles present on this system");
            return;
        }
        Ok(_) => {}
    }

    let result = profile_cmd::link("nonexistent-profile", "stable");
    assert!(
        result.is_ok(),
        "Expected Ok(()) when SMM has no profiles, got Err: {:?}",
        result.err()
    );
}
