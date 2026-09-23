pub fn validate_bearer_token(auth_header: Option<&str>, expected_token: &str) -> bool {
    if expected_token.is_empty() {
        // If no auth token is configured, access is unauthenticated (or internal only)
        return true;
    }

    match auth_header {
        Some(header) => {
            if let Some(token) = header.strip_prefix("Bearer ") {
                token.trim() == expected_token.trim()
            } else {
                false
            }
        }
        None => false,
    }
}
