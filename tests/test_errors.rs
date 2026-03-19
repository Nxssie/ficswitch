use ficswitch::core::steam;

#[test]
fn test_find_manifest_no_steam() {
    // On Linux in CI/test env, Steam candidate dirs don't exist.
    // find_manifest must return Err with a meaningful message — no panic.
    let result = steam::find_manifest();
    // On a dev machine with Steam installed this may return Ok — that's fine.
    // The test verifies: no panic, and if Err then message contains "Could not find".
    match result {
        Ok(_) => {
            // Steam is installed on this machine; the function works correctly.
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("Could not find"),
                "error message should contain 'Could not find', got: {msg}"
            );
        }
    }
}
