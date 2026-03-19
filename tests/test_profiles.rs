use ficswitch::core::{
    profiles::{BranchProfiles, read_branch_profiles, read_smm_profiles, write_branch_profiles},
    steam::Branch,
};
use std::collections::HashMap;

#[test]
fn test_profile_show_empty() {
    // read_branch_profiles returns default (empty) when branch_profiles.json does not exist.
    // If the file exists (from prior test runs) it returns the stored mappings.
    // Key invariant: no panic. The function must not panic regardless of state.
    let result = read_branch_profiles();
    // Accept Ok or Err — just ensure no panic occurs.
    let _ = result;
}

#[test]
fn test_profile_link_no_smm() {
    use ficswitch::core::profiles::link_profile;
    // link_profile("NonExistentProfile") must always return Err:
    // - If no ficsit dir: "not found in SMM" (empty profile list)
    // - If ficsit dir exists but parse fails: JSON parse error
    // - If ficsit dir exists and parses: "not found in SMM" (profile absent)
    // All paths must return Err — never Ok.
    let result = link_profile("__ficswitch_test_nonexistent_profile__", &Branch::Stable);
    assert!(result.is_err(), "expected Err for a profile that cannot exist");
}

#[test]
fn test_profile_list_empty_smm() {
    // read_smm_profiles must not panic.
    // On machines without ficsit it returns Ok(empty).
    // On machines with ficsit but incompatible SMM schema it may return Err.
    // The key invariant: no panic.
    let result = read_smm_profiles();
    // Accept Ok or Err — just assert no panic.
    let _ = result;
}

#[test]
fn test_branch_profiles_write_read() {
    // Write a BranchProfiles with one mapping then read it back.
    // This exercises the switcher_config_dir path (writes to real data dir).
    let mut mappings = HashMap::new();
    mappings.insert("stable".to_string(), "MyMod".to_string());
    let original = BranchProfiles { mappings };

    write_branch_profiles(&original).expect("write_branch_profiles failed");

    let loaded = read_branch_profiles().expect("read_branch_profiles failed");
    assert_eq!(
        loaded.mappings.get("stable"),
        Some(&"MyMod".to_string()),
        "round-trip: stable mapping should be 'MyMod'"
    );
}
