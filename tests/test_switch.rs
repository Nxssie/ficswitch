use ficswitch::core::steam;

#[test]
fn test_no_backup_path_compiles() {
    // Compile-time check: verify the switch command module exists and is accessible.
    // The --no-backup flag is defined in main.rs Commands::Switch { no_backup: bool }.
    // Since we cannot call switch::run() without a live manifest, this test just
    // ensures the module structure compiles correctly.
    let _ = ficswitch::commands::switch::run;
    // If this compiles, the module exists with the expected function.
}

#[test]
fn test_is_steam_running_returns_bool() {
    // Exercise the sysinfo 0.30 code path (.to_string_lossy()).
    // Asserts the function completes without panicking and returns a bool.
    let result: bool = steam::is_steam_running();
    // The value itself doesn't matter in test env — just that it's a valid bool.
    let _ = result;
}
