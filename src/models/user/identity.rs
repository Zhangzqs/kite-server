use crate::error::{ApiError, Result};
use crate::models::user::UserError;

use super::Identity;

impl Identity {
    pub fn new(uid: i32, student_id: String) -> Self {
        Self {
            uid,
            student_id,
            ..Identity::default()
        }
    }

    pub async fn validate_oa_account(student_id: &str, oa_secret: &str) -> Result<()> {
        if is_default_digit(oa_secret) {
            return Err(ApiError::new(UserError::DefaultSecretDenied));
        }
        if !is_student_id(student_id) {
            return Err(ApiError::new(UserError::NoSuchStudentNo));
        }
        if is_not_undergraduate(student_id) {
            return Err(ApiError::new(UserError::NoSupport));
        }
        super::authserver::verify_portal_login(student_id, oa_secret).await?;
        Ok(())
    }
}

fn is_default_digit(secret: &str) -> bool {
    secret.len() == 6 && secret.chars().filter(char::is_ascii_digit).count() == 6
}

fn is_student_id(account: &str) -> bool {
    account.len() == 9 || account.len() == 10
}

fn is_not_undergraduate(account: &str) -> bool {
    account.chars().nth(2) == Some('3')
}

#[test]
fn test_is_ascii_digit() {
    assert!(is_default_digit("123456"));
    assert!(!is_default_digit("1234567"));
    assert!(!is_default_digit("Hello1"));
}

#[test]
fn test_is_student_id() {
    assert!(is_student_id("2111421206"));
    assert!(!is_student_id("21310101208032"));
}

#[test]
fn test_is_undergraduate() {
    assert!(!is_not_undergraduate("12456789"));
    assert!(is_not_undergraduate("12345678"))
}
