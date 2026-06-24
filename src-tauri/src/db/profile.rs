use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub location: String,
    pub cv_text: String,
    pub criteria_json: String,
}

pub fn get(conn: &Connection) -> rusqlite::Result<Profile> {
    let found = conn
        .query_row(
            "SELECT full_name, email, phone, location, cv_text, criteria_json \
             FROM profile WHERE id = 1",
            [],
            |r| {
                Ok(Profile {
                    full_name: r.get(0)?,
                    email: r.get(1)?,
                    phone: r.get(2)?,
                    location: r.get(3)?,
                    cv_text: r.get(4)?,
                    criteria_json: r.get(5)?,
                })
            },
        )
        .optional()?;
    Ok(found.unwrap_or(Profile {
        criteria_json: "{}".into(),
        ..Default::default()
    }))
}

pub fn upsert(conn: &Connection, p: &Profile) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO profile (id, full_name, email, phone, location, cv_text, criteria_json, updated_at) \
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, datetime('now')) \
         ON CONFLICT(id) DO UPDATE SET \
            full_name=?1, email=?2, phone=?3, location=?4, cv_text=?5, criteria_json=?6, updated_at=datetime('now')",
        (&p.full_name, &p.email, &p.phone, &p.location, &p.cv_text, &p.criteria_json),
    )?;
    Ok(())
}

pub fn is_onboarding_complete(conn: &Connection) -> rusqlite::Result<bool> {
    let p = get(conn)?;
    let criteria_present = p.criteria_json.trim() != "{}" && !p.criteria_json.trim().is_empty();
    Ok(!p.full_name.trim().is_empty() && !p.cv_text.trim().is_empty() && criteria_present)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn get_on_empty_db_returns_blank_profile() {
        let conn = open_in_memory();
        let p = get(&conn).unwrap();
        assert_eq!(p.full_name, "");
        assert_eq!(p.criteria_json, "{}");
    }

    #[test]
    fn upsert_then_get_roundtrips() {
        let conn = open_in_memory();
        let p = Profile {
            full_name: "Christian".into(),
            email: "c@example.com".into(),
            phone: "".into(),
            location: "Brazil".into(),
            cv_text: "10 years backend".into(),
            criteria_json: r#"{"role":"backend"}"#.into(),
        };
        upsert(&conn, &p).unwrap();
        let got = get(&conn).unwrap();
        assert_eq!(got.full_name, "Christian");
        assert_eq!(got.criteria_json, r#"{"role":"backend"}"#);
    }

    #[test]
    fn upsert_twice_keeps_single_row() {
        let conn = open_in_memory();
        let mut p = Profile { full_name: "A".into(), criteria_json: "{}".into(), ..Default::default() };
        upsert(&conn, &p).unwrap();
        p.full_name = "B".into();
        upsert(&conn, &p).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM profile", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        assert_eq!(get(&conn).unwrap().full_name, "B");
    }

    #[test]
    fn onboarding_incomplete_until_all_fields_present() {
        let conn = open_in_memory();
        assert!(!is_onboarding_complete(&conn).unwrap());
        upsert(&conn, &Profile {
            full_name: "C".into(),
            cv_text: "cv".into(),
            criteria_json: r#"{"role":"backend"}"#.into(),
            ..Default::default()
        }).unwrap();
        assert!(is_onboarding_complete(&conn).unwrap());
    }
}
