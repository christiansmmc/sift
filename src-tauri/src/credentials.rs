//! LinkedIn credential storage in the OS keychain (Windows Credential Manager).
//! Credentials never touch SQLite, plain text, or logs.

use keyring::Entry;

const SERVICE: &str = "applybot-linkedin";
const USER_POINTER: &str = "__current_user__";

fn entry(account: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, account).map_err(|e| format!("keychain entry: {e}"))
}

pub fn save_linkedin(username: &str, password: &str) -> Result<(), String> {
    entry(username)?
        .set_password(password)
        .map_err(|e| format!("save password: {e}"))?;
    entry(USER_POINTER)?
        .set_password(username)
        .map_err(|e| format!("save username pointer: {e}"))?;
    Ok(())
}

pub fn current_username() -> Option<String> {
    entry(USER_POINTER).ok()?.get_password().ok()
}

pub fn has_linkedin() -> bool {
    match current_username() {
        Some(user) => entry(&user)
            .ok()
            .and_then(|e| e.get_password().ok())
            .is_some(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_then_has_and_read_username() {
        let user = "applybot-test-user@example.com";
        save_linkedin(user, "secret-pw").expect("save");
        assert!(has_linkedin());
        assert_eq!(current_username().as_deref(), Some(user));
        let _ = entry(user).unwrap().delete_credential();
        let _ = entry(USER_POINTER).unwrap().delete_credential();
    }
}
