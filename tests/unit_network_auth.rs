use desktop_mcp::security::auth::validate_bearer_token;

#[test]
fn test_bearer_token_validation() {
    let expected = "my-super-secret-token";

    // Valid header
    let valid_header = "Bearer my-super-secret-token";
    assert!(validate_bearer_token(Some(valid_header), expected));

    // Invalid token
    let invalid_header = "Bearer wrong-token";
    assert!(!validate_bearer_token(Some(invalid_header), expected));

    // Missing Bearer prefix
    let malformed = "my-super-secret-token";
    assert!(!validate_bearer_token(Some(malformed), expected));

    // Missing header
    assert!(!validate_bearer_token(None, expected));
}
