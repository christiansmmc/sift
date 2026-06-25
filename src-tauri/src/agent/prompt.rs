use crate::db::profile::Profile;

const TEMPLATE: &str = include_str!("system_prompt.md");
const SUBMIT_TEMPLATE: &str = include_str!("submit_prompt.md");

fn mode_instructions(mode: &str, batch_size: u32) -> String {
    match mode {
        "scan" => format!(
            "MODE: SCAN. Quickly DISCOVER up to {batch_size} jobs that match the criteria. \
For each good match, report APPLYBOT_JOB with title, company, url, and match_summary ONLY. \
Do NOT open Easy Apply, do NOT write a cover letter, do NOT answer screening questions. \
Leave cover_letter as \"\" and answers as []. This is a fast discovery pass."
        ),
        _ => format!(
            "MODE: REVISAR. For up to {batch_size} good Easy-Apply matches, open the application, \
read the screening questions, prepare a tailored cover letter and the answers, and report \
APPLYBOT_JOB with cover_letter and answers filled in. Do NOT submit — the user reviews first."
        ),
    }
}

pub fn build_system_prompt(profile: &Profile, answers: &[(String, String)], mode: &str, batch_size: u32) -> String {
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
        .replace("{{MODE_INSTRUCTIONS}}", &mode_instructions(mode, batch_size))
        .replace("{{BATCH_SIZE}}", &batch_size.to_string())
        .replace("{{PROFILE}}", &profile_block)
        .replace("{{CRITERIA}}", &criteria_block)
        .replace("{{SCREENING}}", &screening_block)
        .replace("{{ANSWER_BANK}}", &answer_bank)
}

pub fn build_submit_prompt(items: &[crate::db::applications::SubmitItem]) -> String {
    let block = if items.is_empty() {
        "(none)".to_string()
    } else {
        items
            .iter()
            .map(|it| {
                format!(
                    "Application id {}: {}\n  Cover letter: {}\n  Answers (JSON): {}",
                    it.application_id, it.url, it.cover_letter, it.answers_json
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    SUBMIT_TEMPLATE.replace("{{APPLICATIONS}}", &block)
}

#[cfg(test)]
mod submit_tests {
    use super::*;
    use crate::db::applications::SubmitItem;

    #[test]
    fn submit_prompt_lists_applications() {
        let items = vec![SubmitItem {
            application_id: 7,
            url: "https://linkedin.com/jobs/7".into(),
            cover_letter: "Dear Acme".into(),
            answers_json: r#"[{"question":"Q","answer":"A"}]"#.into(),
        }];
        let out = build_submit_prompt(&items);
        assert!(out.contains("Application id 7"));
        assert!(out.contains("linkedin.com/jobs/7"));
        assert!(out.contains("APPLYBOT_SUBMITTED"));
        assert!(!out.contains("{{"));
    }
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
        let out = build_system_prompt(&p, &answers, "revisar", 10);
        assert!(out.contains("MODE: REVISAR"));
        assert!(out.contains("Ada"));
        assert!(out.contains("8 years backend"));
        assert!(out.contains(r#"{"role":"backend"}"#));
        assert!(out.contains("Years of Rust?"));
        assert!(out.contains("A: 8"));
        assert!(!out.contains("{{")); // no leftover placeholders
        let scan = build_system_prompt(&p, &answers, "scan", 5);
        assert!(scan.contains("MODE: SCAN"));
    }

    #[test]
    fn empty_answers_shows_placeholder() {
        let p = Profile {
            full_name: "Ada".into(),
            screening_json: "{}".into(),
            criteria_json: "{}".into(),
            ..Default::default()
        };
        let out = build_system_prompt(&p, &[], "revisar", 5);
        assert!(out.contains("(none saved yet)"));
        assert!(!out.contains("{{"));
    }
}
