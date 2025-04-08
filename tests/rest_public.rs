// tests/rest_public.rs

// Import the common setup function
mod common;

// Remove the use statement if present
// use assert_matches::assert_matches;

// Remove the unused import
// use orderly_connector_rs::error::Result;
use orderly_connector_rs::rest::OrderlyService;
use orderly_connector_rs::types::*; // This should import all types including the new ones

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

/// Tests the open interest endpoint.
///
/// This test verifies that:
/// - The API can retrieve open interest data for all trading pairs
/// - The response contains valid open interest records
/// - Each record has required fields (symbol, long_oi, short_oi)
/// - Open interest values are non-negative
/// - Both reference and consuming iterators work correctly on the response data
///
/// Note: This test is ignored by default as it requires network access.
#[tokio::test]
#[ignore]
async fn test_get_open_interest() {
    common::setup();
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_open_interest().await;
    println!("Open Interest Result: {:#?}", result);
    assert!(
        result.is_ok(),
        "Failed to get open interest: {:?}",
        result.err()
    );

    let oi_resp = result.unwrap();
    println!("Response structure: {:#?}", oi_resp);
    assert!(oi_resp.success, "API response indicates failure");

    // Test consuming iterator
    let oi_data = oi_resp.data;
    assert!(!oi_data.rows.is_empty(), "No open interest records found");

    // Validate first record
    let first_record = &oi_data.rows[0];
    assert!(!first_record.symbol.is_empty(), "Symbol is empty");
    assert!(first_record.long_oi >= 0.0, "Long OI is negative");
    assert!(
        first_record.short_oi <= 0.0,
        "Short OI should be negative or zero"
    );

    // Test reference iterator
    for record in &oi_data {
        assert!(!record.symbol.is_empty(), "Symbol is empty in record");
        assert!(record.long_oi >= 0.0, "Long OI is negative in record");
        assert!(
            record.short_oi <= 0.0,
            "Short OI should be negative or zero"
        );
    }

    // Test consuming iterator
    for record in oi_data {
        assert!(!record.symbol.is_empty(), "Symbol is empty in record");
        assert!(record.long_oi >= 0.0, "Long OI is negative in record");
        assert!(
            record.short_oi <= 0.0,
            "Short OI should be negative or zero"
        );
    }
}

#[tokio::test]
#[ignore] // Ignored by default to avoid hitting the API on every `cargo test` run
async fn test_get_positions_under_liquidation() {
    let service = OrderlyService::new(true, None).expect("Failed to create service"); // Use testnet

    // Test without params
    let result = service.get_positions_under_liquidation(None).await;
    println!("Get Positions Under Liquidation (no params): {:?}", result);
    assert!(result.is_ok());
    if let Ok(response) = result {
        assert!(response.success);
        // Basic checks on structure
        assert!(response.data.meta.records_per_page > 0); // Should have some default page size
    }

    // Test with params (example: page 1, size 5)
    let params = GetPositionsUnderLiquidationParams {
        page: Some(1),
        size: Some(5),
        ..Default::default()
    };
    let result_params = service.get_positions_under_liquidation(Some(params)).await;
    println!(
        "Get Positions Under Liquidation (with params): {:?}",
        result_params
    );
    assert!(result_params.is_ok());
    if let Ok(response) = result_params {
        assert!(response.success);
        // Check if pagination params are reflected (if API does)
        assert_eq!(response.data.meta.current_page, 1);
        // Note: API might return less than requested size if total < size
        assert!(response.data.meta.records_per_page <= 5 || response.data.rows.len() <= 5);
    }
}

#[tokio::test]
#[ignore] // Ignored by default to avoid hitting the API on every test run
async fn test_get_price_changes() {
    common::setup();
    let is_testnet = common::get_testnet_flag();

    let client = OrderlyService::new(is_testnet, None).expect("Failed to create REST client");

    let result = client.get_price_changes().await;
    println!("Get Price Changes Response: {:?}", result);
    assert!(result.is_ok());

    let price_changes = result.unwrap();
    assert!(price_changes.success);
    assert!(
        !price_changes.data.rows.is_empty(),
        "Expected at least one price change row"
    );

    // Test reference iterator on response data
    println!("Testing reference iterator on response data:");
    for price in &price_changes.data {
        assert!(!price.symbol.is_empty(), "Symbol should not be empty");
        assert!(price.last_price > 0.0, "Last price should be positive");
        // Note: Historical prices might be null, so we don't assert on them
        println!("Symbol: {}, Last Price: {}", price.symbol, price.last_price);
    }

    // Test consuming iterator on response data
    println!("Testing consuming iterator on response data:");
    let data = price_changes.data;
    let mut count = 0;
    for price in data {
        count += 1;
        assert!(!price.symbol.is_empty(), "Symbol should not be empty");
        if let Some(change_24h) = price.twenty_four_hour {
            println!("24h change for {}: {}", price.symbol, change_24h);
        }
    }
    assert!(count > 0, "Should have iterated over at least one price");

    // Test reference iterator on full response
    println!("Testing reference iterator on full response:");
    let result = client.get_price_changes().await.unwrap();
    for price in &result {
        assert!(!price.symbol.is_empty(), "Symbol should not be empty");
        if let Some(change_7d) = price.seven_day {
            println!("7d change for {}: {}", price.symbol, change_7d);
        }
    }

    // Test consuming iterator on full response
    println!("Testing consuming iterator on full response:");
    let result = client.get_price_changes().await.unwrap();
    let mut symbols = Vec::new();
    for price in result {
        symbols.push(price.symbol.clone());
    }
    assert!(!symbols.is_empty(), "Should have collected some symbols");
    println!("Found {} symbols", symbols.len());
}
