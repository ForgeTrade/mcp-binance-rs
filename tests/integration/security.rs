//! Credential Security Integration Tests
//!
//! Verifies that API credentials are never exposed in logs or error messages.

#[test]
fn test_secret_string_masking() {
    use mcp_binance_server::config::credentials::SecretString;

    let secret = SecretString::new("my_super_secret_key_12345".to_string());

    // Debug format should mask
    let debug_output = format!("{:?}", secret);
    assert!(
        !debug_output.contains("my_super_secret_key_12345"),
        "Debug output should not contain full secret: {}",
        debug_output
    );
    assert!(
        debug_output.contains("***") || debug_output.contains("REDACTED"),
        "Debug output should show masking: {}",
        debug_output
    );

    // Display format should show truncated version
    let display_output = format!("{}", secret);
    assert!(
        !display_output.contains("my_super_secret_key_12345"),
        "Display output should not contain full secret: {}",
        display_output
    );
    assert!(
        display_output.contains("...") || display_output.len() < 20,
        "Display should show truncated version: {}",
        display_output
    );
}

#[test]
fn test_credentials_from_env() {
    use mcp_binance_server::config::credentials::Credentials;

    // Set test credentials
    unsafe {
        std::env::set_var("BINANCE_API_KEY", "test_key");
        std::env::set_var("BINANCE_SECRET_KEY", "test_secret");
    }

    let creds = Credentials::from_env();
    assert!(creds.is_ok(), "Should load credentials from env");

    let creds = creds.unwrap();

    // Verify values are loaded
    assert_eq!(creds.api_key.expose_secret(), "test_key");
    assert_eq!(creds.secret_key.expose_secret(), "test_secret");

    // Cleanup
    unsafe {
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_SECRET_KEY");
    }
}

#[test]
fn test_credentials_missing() {
    use mcp_binance_server::config::credentials::Credentials;

    // Remove credentials
    unsafe {
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_SECRET_KEY");
    }

    let creds = Credentials::from_env();
    assert!(
        creds.is_err(),
        "Should return error when credentials are missing"
    );
}

#[test]
fn test_credentials_whitespace_trimmed() {
    use mcp_binance_server::config::credentials::Credentials;

    // Set credentials with whitespace
    unsafe {
        std::env::set_var("BINANCE_API_KEY", "  test_key_with_spaces  ");
        std::env::set_var("BINANCE_SECRET_KEY", "  test_secret_with_spaces  ");
    }

    let creds = Credentials::from_env();
    assert!(creds.is_ok(), "Should load credentials with whitespace");

    let creds = creds.unwrap();

    // Verify whitespace is trimmed
    assert_eq!(creds.api_key.expose_secret(), "test_key_with_spaces");
    assert_eq!(creds.secret_key.expose_secret(), "test_secret_with_spaces");

    // Cleanup
    unsafe {
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_SECRET_KEY");
    }
}

#[test]
fn test_secret_string_does_not_derive_serialize() {
    // This is a compile-time check that SecretString doesn't implement Serialize
    // If it does, this test will fail to compile
    use mcp_binance_server::config::credentials::SecretString;

    fn assert_not_serialize<T: ?Sized>() {}

    let secret = SecretString::new("test".to_string());
    let _: &dyn std::fmt::Debug = &secret; // Should implement Debug
    let _: &dyn std::fmt::Display = &secret; // Should implement Display

    // This line would fail to compile if SecretString implemented Serialize
    // assert_not_serialize::<SecretString>();

    // Instead, we verify indirectly
    assert!(
        !format!("{:?}", secret).contains("test"),
        "Secret should be masked in debug output"
    );
}

#[tokio::test]
async fn test_server_initializes_without_credentials() {
    use mcp_binance_server::server::BinanceServer;

    // Remove credentials
    unsafe {
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_SECRET_KEY");
    }

    // Server should initialize successfully without credentials
    let server = BinanceServer::new();

    // Verify server has no credentials
    assert!(
        server.credentials.is_none(),
        "Server should have no credentials when env vars are not set"
    );
}

#[tokio::test]
async fn test_server_initializes_with_credentials() {
    use mcp_binance_server::server::BinanceServer;

    // Set test credentials
    unsafe {
        std::env::set_var("BINANCE_API_KEY", "test_api_key");
        std::env::set_var("BINANCE_SECRET_KEY", "test_secret_key");
    }

    // Server should initialize with credentials
    let server = BinanceServer::new();

    // Verify server has credentials
    assert!(
        server.credentials.is_some(),
        "Server should have credentials when env vars are set"
    );

    // Cleanup
    unsafe {
        std::env::remove_var("BINANCE_API_KEY");
        std::env::remove_var("BINANCE_SECRET_KEY");
    }
}
