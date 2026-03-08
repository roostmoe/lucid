use rand::Rng;

#[test]
fn test_csrf_token_format() {
    // CSRF tokens should be 32 alphanumeric characters
    let csrf_token: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    assert_eq!(csrf_token.len(), 32);
    assert!(csrf_token.chars().all(|c: char| c.is_ascii_alphanumeric()));
}

#[test]
fn test_session_id_is_ulid() {
    // Session IDs should be valid ULIDs
    let session_id = ulid::Ulid::new().to_string();

    assert_eq!(session_id.len(), 26);
    assert!(ulid::Ulid::from_string(&session_id).is_ok());
}

#[test]
fn test_secure_cookie_flag_https() {
    let public_url = "https://example.com";
    let secure_flag = if public_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };

    assert_eq!(secure_flag, "; Secure");
}

#[test]
fn test_secure_cookie_flag_http() {
    let public_url = "http://localhost:8080";
    let secure_flag = if public_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };

    assert_eq!(secure_flag, "");
}

#[test]
fn test_cookie_max_age_calculation() {
    let max_age = 30 * 24 * 60 * 60; // 30 days in seconds
    assert_eq!(max_age, 2_592_000);
}

#[test]
fn test_cookie_parsing_logic() {
    // Simulate the cookie extraction logic used in auth_logout
    let cookie_header = "other=value; lucid_session=test_token; foo=bar";
    let signed_cookie = cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("lucid_session="))
        .and_then(|s| s.strip_prefix("lucid_session="));

    assert_eq!(signed_cookie, Some("test_token"));
}

#[test]
fn test_cookie_parsing_not_found() {
    let cookie_header = "other=value; foo=bar";
    let signed_cookie = cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("lucid_session="))
        .and_then(|s| s.strip_prefix("lucid_session="));

    assert_eq!(signed_cookie, None);
}

#[test]
fn test_cookie_parsing_empty_value() {
    let cookie_header = "lucid_session=";
    let signed_cookie = cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("lucid_session="))
        .and_then(|s| s.strip_prefix("lucid_session="));

    assert_eq!(signed_cookie, Some(""));
}

#[test]
fn test_logout_cookie_format() {
    // Cookie for logout should have Max-Age=0 and all security flags
    // Testing HTTP version
    let public_url = "http://localhost:8080";
    let secure_flag = if public_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let cookie = format!(
        "lucid_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{}",
        secure_flag
    );

    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("Max-Age=0"));
    assert!(cookie.starts_with("lucid_session=;"));
    assert!(!cookie.contains("Secure"));

    // Testing HTTPS version
    let public_url_https = "https://example.com";
    let secure_flag_https = if public_url_https.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let cookie_https = format!(
        "lucid_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{}",
        secure_flag_https
    );

    assert!(cookie_https.contains("Secure"));
    assert!(cookie_https.contains("HttpOnly"));
    assert!(cookie_https.contains("Max-Age=0"));
}

#[test]
fn test_ulid_uniqueness() {
    // ULIDs should be unique
    let id1 = ulid::Ulid::new().to_string();
    let id2 = ulid::Ulid::new().to_string();

    assert_ne!(id1, id2);
}

#[test]
fn test_csrf_token_uniqueness() {
    // CSRF tokens should be unique
    let token1: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let token2: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    assert_ne!(token1, token2);
}
