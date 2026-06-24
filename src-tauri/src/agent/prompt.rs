use crate::db::profile::Profile;

const TEMPLATE: &str = include_str!("system_prompt.md");

pub fn build_system_prompt(profile: &Profile, batch_size: u32) -> String {
    let profile_block = format!(
        "Name: {}\nEmail: {}\nPhone: {}\nLocation: {}\n\nResume:\n{}",
        profile.full_name, profile.email, profile.phone, profile.location, profile.cv_text
    );
    // criteria_json is already a JSON object; present it as-is for the agent.
    let criteria_block = if profile.criteria_json.trim().is_empty() {
        "{}".to_string()
    } else {
        profile.criteria_json.clone()
    };
    TEMPLATE
        .replace("{{BATCH_SIZE}}", &batch_size.to_string())
        .replace("{{PROFILE}}", &profile_block)
        .replace("{{CRITERIA}}", &criteria_block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_placeholders() {
        let p = Profile {
            full_name: "Ada".into(),
            email: "ada@x.com".into(),
            phone: "".into(),
            location: "Brazil".into(),
            cv_text: "8 years backend".into(),
            criteria_json: r#"{"role":"backend"}"#.into(),
            screening_json: "{}".into(),
        };
        let out = build_system_prompt(&p, 10);
        assert!(out.contains("at most 10 jobs"));
        assert!(out.contains("Ada"));
        assert!(out.contains("8 years backend"));
        assert!(out.contains(r#"{"role":"backend"}"#));
        assert!(!out.contains("{{")); // no leftover placeholders
    }
}
