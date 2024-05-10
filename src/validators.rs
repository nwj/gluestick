use validator::ValidationError;

pub fn not_empty_when_trimmed<S: AsRef<str>>(s: S) -> Result<(), ValidationError> {
    if s.as_ref().trim().is_empty() {
        return Err(ValidationError::new("cannot be empty"));
    }

    Ok(())
}

pub fn is_valid_username<S: AsRef<str>>(s: S) -> Result<(), ValidationError> {
    if s.as_ref().trim().len() < 3 {
        return Err(ValidationError::new(
            "usernames must be 3 characters or longer",
        ));
    }

    Ok(())
}
