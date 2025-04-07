// tests/common.rs
use std::sync::Once;

static INIT: Once = Once::new();

// Initializes environment variables from .env for testing.
// Uses std::sync::Once to ensure it only runs once across all tests.
pub fn setup() {
    INIT.call_once(|| {
        // Explicitly try loading .env from the current directory
        let dotenv_result = dotenv::from_path(".env");
        if dotenv_result.is_ok() {
            println!("Loaded .env file from current directory.");
        } else {
            // Try loading from parent directory (workspace root?)
            let parent_dotenv_result = dotenv::from_path("../.env");
            if parent_dotenv_result.is_ok() {
                println!("Loaded .env file from parent directory.");
            } else {
                println!("Warning: .env file not found in current or parent directory.");
            }
        }
        // env_logger::builder().is_test(true).try_init().ok();
    });
}

#[allow(dead_code)]
pub fn get_env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{} environment variable not set", name))
}

pub fn get_testnet_flag() -> bool {
    std::env::var("ORDERLY_TESTNET")
        .unwrap_or_else(|_| "true".to_string()) // Default to testnet if not set
        .parse::<bool>()
        .expect("ORDERLY_TESTNET must be true or false")
}
