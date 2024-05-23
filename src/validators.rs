use validator::ValidationError;

pub fn not_empty_when_trimmed<S: AsRef<str>>(s: S) -> Result<(), ValidationError> {
    if s.as_ref().trim().is_empty() {
        return Err(ValidationError::new("cannot be empty"));
    }

    Ok(())
}
