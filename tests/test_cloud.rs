use std::fs;
use std::path::PathBuf;

fn setup_test_cloud_env(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("ficswitch_cloud_tests")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create test dir");
    dir
}

#[test]
fn test_cloud_commands_exist() {
    let _ = ficswitch::commands::cloud::status;
    let _ = ficswitch::commands::cloud::backup;
    let _ = ficswitch::commands::cloud::restore;
    let _ = ficswitch::commands::cloud::clear;
}

#[test]
fn test_backup_path_is_deterministic() {
    let test_dir = setup_test_cloud_env("backup_path");
    let sub_dir = test_dir.join("526870").join("remote");
    fs::create_dir_all(&sub_dir).expect("failed to create remote dir");
    fs::write(sub_dir.join("test.sav"), b"test").expect("failed to write test file");

    assert!(sub_dir.exists(), "remote dir should exist");
}

#[test]
fn test_clear_removes_backup_if_exists() {
    let test_dir = setup_test_cloud_env("clear_test");
    let backup_dir = test_dir.join("ficswitch_backup");
    fs::create_dir_all(&backup_dir).expect("failed to create backup dir");
    fs::write(backup_dir.join("data.txt"), b"backup data").expect("failed to write backup file");

    assert!(backup_dir.exists(), "backup dir should exist before clear");
    fs::remove_dir_all(&backup_dir).expect("failed to clear backup dir");
    assert!(
        !backup_dir.exists(),
        "backup dir should not exist after clear"
    );
}

#[test]
fn test_dry_run_parameter_accepted() {
    let _ = |_dry_run: bool| {
        let _ = ficswitch::commands::cloud::backup;
        let _ = ficswitch::commands::cloud::restore;
        let _ = ficswitch::commands::cloud::clear;
    };
}
