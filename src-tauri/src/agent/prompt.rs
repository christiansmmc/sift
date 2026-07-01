use crate::db::profile::Profile;

const TEMPLATE: &str = include_str!("system_prompt.md");
const SUBMIT_TEMPLATE: &str = include_str!("submit_prompt.md");

fn mode_instructions(mode: &str, batch_size: u32) -> String {
    match mode {
        "scan" => format!(
            "MODE: SCAN. Your goal is to DISCOVER and report {batch_size} jobs that match the criteria. \
For each good match, report SIFT_JOB with title, company, url, and match_summary ONLY. \
Do NOT open Easy Apply, do NOT write a cover letter, do NOT answer screening questions. \
Leave cover_letter as \"\" and answers as []. Keep browsing and evaluating more postings until you \
have reported {batch_size} matches — a posting you skip because it does not fit does NOT count \
toward the {batch_size}. This is a fast discovery pass."
        ),
        _ => format!(
            "MODE: REVISAR. Your goal is to report {batch_size} good Easy-Apply matches. For each good \
match, open the application, read the screening questions, prepare a tailored cover letter and the \
answers, and report SIFT_JOB with cover_letter and answers filled in. Keep browsing and evaluating \
more postings until you reach {batch_size} reported matches — a posting you skip because it does not \
fit does NOT count toward the {batch_size}. Do NOT submit — the user reviews first."
        ),
    }
}

pub fn cover_letter_instruction(style: &str, custom: &str) -> String {
    match style {
        "short" => "Keep the cover letter SHORT and simple: at most 2 short paragraphs, first person, casual but professional, as if the candidate wrote it quickly themselves. No clichés, no formal template, plain prose.".to_string(),
        "detailed" => "Write a detailed, specific cover letter: exactly 4 short paragraphs, a concrete company-specific hook, quantified proof of achievements, no clichés ('passionate', 'results-driven'), plain prose.".to_string(),
        "custom" => {
            let c = custom.trim();
            if c.is_empty() {
                // fall back to balanced if custom is selected but empty
                "A balanced cover letter: 3 short paragraphs, specific to the company, one quantified proof. Professional and natural.".to_string()
            } else {
                format!("Follow the candidate's own instructions exactly: {c}")
            }
        }
        _ => "A balanced cover letter: 3 short paragraphs, specific to the company, one quantified proof. Professional and natural.".to_string(),
    }
}

pub fn build_system_prompt(profile: &Profile, answers: &[(String, String)], cover_letter: &str, mode: &str, batch_size: u32) -> String {
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
        .replace("{{COVER_LETTER_STYLE}}", cover_letter)
}

pub fn build_submit_prompt(items: &[crate::db::applications::SubmitItem], follow_company: bool) -> String {
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
    let follow = if follow_company {
        "if LinkedIn offers to follow the company, you may follow it."
    } else {
        "if LinkedIn offers to follow the company, do NOT follow it — decline or uncheck that option."
    };
    SUBMIT_TEMPLATE
        .replace("{{APPLICATIONS}}", &block)
        .replace("{{FOLLOW_COMPANY}}", follow)
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
        let out = build_submit_prompt(&items, false);
        assert!(out.contains("Application id 7"));
        assert!(out.contains("linkedin.com/jobs/7"));
        assert!(out.contains("SIFT_SUBMITTED"));
        assert!(!out.contains("{{"));
        assert!(out.contains("do NOT follow"));

        let out_follow = build_submit_prompt(&items, true);
        assert!(out_follow.contains("you may follow"));
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
        let cl = cover_letter_instruction("balanced", "");
        let out = build_system_prompt(&p, &answers, &cl, "revisar", 10);
        assert!(out.contains("MODE: REVISAR"));
        // The batch counts REPORTED matches, not evaluated candidates: skips must
        // not count, and the stop condition must be framed around matches.
        assert!(out.contains("does NOT count"));
        assert!(out.contains("{{BATCH_SIZE}}") == false && out.contains("MATCHING jobs"));
        assert!(out.contains("Ada"));
        assert!(out.contains("8 years backend"));
        assert!(out.contains(r#"{"role":"backend"}"#));
        assert!(out.contains("Years of Rust?"));
        assert!(out.contains("A: 8"));
        assert!(!out.contains("{{")); // no leftover placeholders
        let scan = build_system_prompt(&p, &answers, &cl, "scan", 5);
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
        let cl = cover_letter_instruction("balanced", "");
        let out = build_system_prompt(&p, &[], &cl, "revisar", 5);
        assert!(out.contains("(none saved yet)"));
        assert!(!out.contains("{{"));
    }

    #[test]
    fn cover_letter_instruction_variants() {
        assert!(cover_letter_instruction("short", "").contains("SHORT"));
        assert!(cover_letter_instruction("detailed", "").contains("4 short paragraphs"));
        assert!(cover_letter_instruction("custom", "use British English").contains("British English"));
        assert!(cover_letter_instruction("custom", "").contains("balanced")); // empty custom falls back
        assert!(cover_letter_instruction("anything", "").contains("balanced"));
    }
}
