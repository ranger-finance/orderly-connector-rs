// tests/rest_public.rs

// Import the common setup function
mod common;

// Remove the use statement if present
// use assert_matches::assert_matches;

use orderly_connector_rs::rest::OrderlyService;

/// Tests the system status endpoint of the Orderly Network API.
///
/// This test verifies that:
/// - The API client can successfully connect to the system status endpoint
/// - The response contains a success flag set to true
/// - The response data is a valid object with a status field equal to 0
///
/// Note: This test is ignored by default as it requires network access and credentials.
#[tokio::test]
#[ignore] // Ignored by default as it requires network access and credentials
async fn test_get_system_status() {
    common::setup(); // Load .env variables
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_system_status().await;
    println!("System Status Result: {:#?}", result);
    assert!(result.is_ok());
    let status_resp = result.unwrap();
    println!("System Status Response Structure: {:#?}", status_resp);
    assert!(status_resp["success"].as_bool().unwrap_or(false));
    assert!(status_resp["data"].is_object());
    assert_eq!(status_resp["data"]["status"].as_i64(), Some(0));
}

/// Tests the exchange information endpoint for all trading instruments.
///
/// This test verifies that:
/// - The API can retrieve information about all available trading instruments
/// - The response contains a valid array of instruments
/// - Each instrument has the required fields (symbol, base_min, base_max, etc.)
/// - The field types match the expected format (strings, numbers)
///
/// Note: This test is ignored by default as it requires network access.
#[tokio::test]
#[ignore] // Ignored by default as it requires network access
async fn test_get_exchange_info_all() {
    common::setup(); // Load .env variables
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_exchange_info(None).await;
    println!("Exchange Info (All) Result: {:#?}", result);
    assert!(
        result.is_ok(),
        "Failed to get exchange info: {:?}",
        result.err()
    );

    let info_resp = result.unwrap();
    println!("Response structure: {:#?}", info_resp);
    assert!(info_resp.success, "API response indicates failure");

    // Assert that data is the 'All' variant using match
    match info_resp.data {
        orderly_connector_rs::rest::client::ExchangeInfoData::All(_) => { /* Correct variant, do nothing */
        }
        _ => panic!(
            "Expected ExchangeInfoData::All variant, got {:?}",
            info_resp.data
        ),
    }

    // Get the rows array by matching the enum variant
    let instruments = if let orderly_connector_rs::rest::client::ExchangeInfoData::All(all_data) =
        info_resp.data
    {
        all_data.rows
    } else {
        panic!("Expected ExchangeInfoData::All variant");
    };

    assert!(
        !instruments.is_empty(),
        "instruments array should not be empty"
    );

    // Validate structure of first instrument
    let first_instrument = &instruments[0];

    // Print the first instrument to see its structure
    println!("First instrument structure: {:#?}", first_instrument);

    // Direct field access for assertions
    assert!(
        !first_instrument.symbol.is_empty(),
        "symbol field missing or empty"
    );

    // Check for base_min and base_max (assuming they are always present f64)
    assert!(
        first_instrument.base_min >= 0.0,
        "base_min missing or invalid"
    );
    assert!(
        first_instrument.base_max >= 0.0,
        "base_max missing or invalid"
    );

    // Check for base_tick
    assert!(
        first_instrument.base_tick > 0.0,
        "base_tick missing or invalid"
    );

    // Check for quote_min and quote_max
    assert!(
        first_instrument.quote_min >= 0.0,
        "quote_min missing or invalid"
    );
    assert!(
        first_instrument.quote_max >= 0.0,
        "quote_max missing or invalid"
    );

    // Check for quote_tick
    assert!(
        first_instrument.quote_tick > 0.0,
        "quote_tick missing or invalid"
    );
}

/// Tests the exchange information endpoint for a specific trading instrument.
///
/// This test verifies that:
/// - The API can retrieve information for a specific symbol (PERP_ETH_USDC)
/// - The response contains valid data for the requested symbol
/// - The response structure matches the expected format
///
/// Note: This test is ignored by default and assumes the specified symbol exists
/// on the selected network (testnet/mainnet).
#[tokio::test]
#[ignore] // Ignored by default
async fn test_get_exchange_info_specific() {
    common::setup();
    let is_testnet = common::get_testnet_flag();
    let symbol = "PERP_ETH_USDC"; // Assumes this symbol exists on testnet/mainnet

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_exchange_info(Some(symbol)).await;
    println!("Exchange Info ({}) Result: {:#?}", symbol, result);
    assert!(result.is_ok());
    let info_resp = result.unwrap();
    println!("Response structure for specific symbol: {:#?}", info_resp);

    assert!(info_resp.success, "API response indicates failure");

    // Assert that data is the 'Single' variant using match
    match info_resp.data {
        orderly_connector_rs::rest::client::ExchangeInfoData::Single(_) => { /* Correct variant, do nothing */
        }
        _ => panic!(
            "Expected ExchangeInfoData::Single variant, got {:?}",
            info_resp.data
        ),
    }

    // Check the symbol within the Single variant
    if let orderly_connector_rs::rest::client::ExchangeInfoData::Single(symbol_info) =
        info_resp.data
    {
        println!("Data field structure: {:#?}", symbol_info);
        assert_eq!(
            symbol_info.symbol, symbol,
            "Symbol in response does not match request"
        );
    } else {
        panic!("Expected ExchangeInfoData::Single variant");
    }
}

/// Tests the funding rate history endpoint.
///
/// This test verifies that:
/// - The API can retrieve historical funding rate data
/// - The response contains valid funding rate records
/// - Each record has required fields (symbol, data_start_time)
/// - Funding rates are within reasonable bounds (-1.0 to 1.0)
/// - One-day statistics contain valid positive/negative counts
/// - Both reference and consuming iterators work correctly on the response data
///
/// Note: This test is ignored by default as it requires network access.
#[tokio::test]
#[ignore] // Ignored by default as it requires network access
async fn test_get_funding_rate_history() {
    common::setup();
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_funding_rate_history().await;
    println!("Funding Rate History Result: {:#?}", result);
    assert!(result.is_ok());

    let history_resp = result.unwrap();
    assert!(history_resp.success);
    assert!(
        !history_resp.data.rows.is_empty(),
        "Should have at least one market"
    );

    // Test reference iterator on response data
    println!("Testing reference iterator on response data:");
    for funding_rate in &history_resp.data {
        assert!(
            !funding_rate.symbol.is_empty(),
            "Symbol should not be empty"
        );
        assert!(
            !funding_rate.data_start_time.is_empty(),
            "Start time should not be empty"
        );
        assert!(
            funding_rate.funding.last.rate.abs() <= 1.0,
            "Funding rate should be reasonable"
        );
    }

    // Test consuming iterator on response data
    println!("Testing consuming iterator on response data:");
    let data = history_resp.data;
    let mut count = 0;
    for funding_rate in data {
        count += 1;
        if let Some(one_day) = &funding_rate.funding.one_day {
            assert!(
                one_day.positive >= 0,
                "Positive count should be non-negative"
            );
            assert!(
                one_day.negative >= 0,
                "Negative count should be non-negative"
            );
        }
    }
    assert!(count > 0, "Should have iterated over at least one item");
}

// Add more tests for other public endpoints like get_futures_info...
