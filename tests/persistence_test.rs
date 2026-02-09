use amnesia::persistence;
use std::fs;

#[test]
fn test_persistence_full_cycle() {
    let path = "test_persistence.amnesio";
    let content = "VERIFIED CONTENT 1.2.0 WITH ARGON2ID";
    let password = "supersecretpassword888";

    // Clean up if it exists
    fs::remove_file(path).ok();

    // 1. Save
    persistence::save_encrypted(path, content, password).expect("Save failed");

    // 2. Load
    let loaded = persistence::load_encrypted(path, password).expect("Load failed");
    assert_eq!(content, loaded);

    // 3. Wrong Password
    let result = persistence::load_encrypted(path, "wrongpassword");
    assert!(result.is_err());

    // 4. Cleanup
    fs::remove_file(path).ok();
}
