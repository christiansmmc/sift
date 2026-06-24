use crate::db::profile::Profile;

const TEMPLATE: &str = include_str!("system_prompt.md");

pub fn build_system_prompt(profile: &Profile, answers: &[(String, String)], batch_size: u32) -> String {
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
    let screening_block = if profile.screening_json.trim().is_empty() {
        "{}".to_string()
    } else {
        profile.screening_json.clone()
    };
    let answer_bank = if answers.is_empty() {
        "(none saved yet)".to_string()
    } else {
        answers
            .iter()
            .map(|(q, a)| format!("Q: {q}\nA: {a}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    TEMPLATE
        .replace("{{BATCH_SIZE}}", &batch_size.to_string())
        .replace("{{PROFILE}}", &profile_block)
        .replace("{{CRITERIA}}", &criteria_block)
        .replace("{{SCREENING}}", &screening_block)
        .replace("{{ANSWER_BANK}}", &answer_bank)
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
        let answers = vec![
            ("Years of Rust?".to_string(), "8".to_string()),
        ];
        let out = build_system_prompt(&p, &answers, 10);
        assert!(out.contains("at most 10 jobs"));
        assert!(out.contains("Ada"));
        assert!(out.contains("8 years backend"));
        assert!(out.contains(r#"{"role":"backend"}"#));
        assert!(out.contains("Years of Rust?"));
        assert!(out.contains("A: 8"));
        assert!(!out.contains("{{")); // no leftover placeholders
    }

    #[test]
    fn empty_answers_shows_placeholder() {
        let p = Profile {
            full_name: "Ada".into(),
            screening_json: "{}".into(),
            criteria_json: "{}".into(),
            ..Default::default()
        };
        let out = build_system_prompt(&p, &[], 5);
        assert!(out.contains("(none saved yet)"));
        assert!(!out.contains("{{"));
    }
}
