use core::fmt;

use error::{UserError, UserResult};
use regex::Regex;

#[derive(Debug)]
pub struct Username {
    content: String,
}

impl Username {
    pub fn new(content: &str) -> UserResult<Self> {
        if content.trim().is_empty() {
            return Err(error("username is empty"));
        }

        if content.len() < 3 {
            return Err(error("username is less than 3 characters"));
        }

        if content.len() > 20 {
            return Err(error("username is more than 20 characters"));
        }

        let regex =
            Regex::new(r"[a-zA-Z0-9_]").map_err(|e| UserError::CodecError(e.to_string()))?;

        if !regex.is_match(content) {
            return Err(error(
                "username should contain only alphanumeric characters",
            ));
        }

        Ok(Self {
            content: content.to_string(),
        })
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

fn error(text: &str) -> UserError {
    UserError::InvalidCredentials(text.to_string())
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use error::UserError;

    use crate::user::username::Username;

    #[test]
    fn new_works() {
        let name = Username::new("new_works").unwrap();

        assert_eq!(name.to_string(), "new_works");
    }

    #[test]
    fn new_fails_name_empty() {
        let name = Username::new("");

        assert!(name.is_err());

        assert_matches!(
            name.unwrap_err(),  UserError::InvalidCredentials(m) if m == "username is empty"
        );
    }

    #[test]
    fn new_fails_name_too_short() {
        let name = Username::new("12");

        assert!(name.is_err());

        assert_matches!(
            name.unwrap_err(),  UserError::InvalidCredentials(m) if m == "username is less than 3 characters"
        );
    }

    #[test]
    fn new_fails_name_too_long() {
        let name = Username::new("Lorem ipsum dolor eu.");

        assert!(name.is_err());

        assert_matches!(
            name.unwrap_err(),  UserError::InvalidCredentials(m) if m == "username is more than 20 characters"
        );
    }

    #[test]
    fn new_fails_name_has_illegal_character() {
        let name = Username::new("$#!@$%");

        assert!(name.is_err());

        assert_matches!(
            name.unwrap_err(),  UserError::InvalidCredentials(m) if m == "username should contain only alphanumeric characters"
        );
    }
}
