use ficswitch::core::{backup, steam::Branch};
use std::fs;
use std::path::PathBuf;

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
    // Cannot inject backup_root path; assert no panic when dir may or may not exist.
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
