# Orderly Connector Implementation Plan

## Overview

This document outlines the implementation plan for integrating Orderly Network into the Smart Order Router (SOR) service. The implementation will use the `orderly-connector-rs` SDK, with additional extensions to support advanced order types and WebSocket integration.

## Project Goals

1. Implement missing features in `orderly-connector-rs` (algo orders)
2. Create a fully-featured `OrderlyVenue` implementation for SOR
3. Support real-time market data and order updates via WebSockets
4. Provide robust error handling and retry mechanisms
5. Implement comprehensive testing and performance optimization

## Implementation Phases

### Phase 1: SDK Enhancement (Week 1)

#### Day 1-2: Environment Setup and SDK Analysis
- [x] Fork `orderly-connector-rs` repository
- [ ] Analyze missing endpoints (`create_algo_order`, `cancel_algo_order`, etc.)
- [ ] Create database schema for credential management
- [ ] Set up development environment with proper dependencies

#### Day 3-4: Implement Missing Endpoints
- [ ] Add `AlgoOrderType` and related types
- [ ] Implement `create_algo_order` endpoint
- [ ] Implement `cancel_algo_order` endpoint 
- [ ] Implement `get_algo_orders` endpoint
- [ ] Add proper error handling for algo orders

#### Day 5: Tests and Documentation
- [ ] Write unit tests for new endpoints
- [ ] Document the new APIs with examples
- [ ] Submit PR to upstream `orderly-connector-rs` repository
- [ ] Set up fork as temporary dependency for SOR

### Phase 2: SOR Integration (Week 2)

#### Day 1-2: Core Implementation
- [ ] Implement `OrderlyVenue` struct with configuration
- [ ] Implement credential management functions
- [ ] Create type conversion utilities (SOR ↔ Orderly SDK)
- [ ] Implement `check_readiness` and `get_balance` methods

#### Day 3-4: Order Management
- [ ] Implement `place_order` with pre-trade validation
- [ ] Implement order status retrieval methods
- [ ] Implement query capabilities with filtering
- [ ] Add order cancellation functionality

#### Day 5: Trade History and Market Data
- [ ] Implement `get_trades` and related methods
- [ ] Implement market data retrieval (orderbook, ticker)
- [ ] Add caching for frequently accessed data
- [ ] Implement proper logging infrastructure

### Phase 3: Advanced Features (Week 3)

#### Day 1-2: Algo Order Implementation
- [ ] Implement `place_algo_order` with validation
- [ ] Implement methods to query and manage algo orders
- [ ] Add conversion between SOR and Orderly algo order types
- [ ] Implement specific handling for stop-loss and take-profit orders

#### Day 3-4: WebSocket Integration
- [ ] Implement public market data streaming
- [ ] Implement private user data streaming
- [ ] Add real-time order and position tracking
- [ ] Implement reconnection logic and error recovery

#### Day 5: Error Handling and Optimization
- [ ] Implement retry mechanism for transient errors
- [ ] Add detailed error mapping and context
- [ ] Optimize performance with connection reuse
- [ ] Add metrics for monitoring

### Phase 4: Testing and Documentation (Week 4)

#### Day 1-2: Unit and Integration Testing
- [ ] Write comprehensive unit tests for all methods
- [ ] Create integration tests with mock API responses
- [ ] Test error handling and edge cases
- [ ] Test WebSocket reconnection logic

#### Day 3-4: Performance Testing
- [ ] Conduct load testing with concurrent orders
- [ ] Optimize memory usage and connection management
- [ ] Tune caching strategies and retry parameters
- [ ] Benchmark API call latency

#### Day 5: Documentation and Finalization
- [ ] Update API documentation with examples
- [ ] Finalize configuration and deployment scripts
- [ ] Create operation runbook
- [ ] Conduct final review and release

## Dependencies

```toml
[dependencies]
# Orderly SDK (fork until PR is accepted)
orderly-connector-rs = { git = "https://github.com/ranger-finance/orderly-connector-rs", branch = "feature/algo-orders" }

# Async runtime
tokio = { version = "1.0", features = ["full", "time"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Async traits
async-trait = "0.1"

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "chrono", "json", "decimal"] }

# Decimal handling
rust_decimal = "1.30"
rust_decimal_macros = "1.30"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Testing
mockito = "1.2"
tokio-test = "0.4"
```

## Implementation Details

### AlgoOrderType Implementation

Algo orders in Orderly include:
- `StopMarket`: Execute market order when price reaches trigger
- `StopLimit`: Place limit order when price reaches trigger
- `TakeProfitMarket`: Market order at profit target
- `TakeProfitLimit`: Limit order at profit target
- `TrailingStop`: Moving stop-loss that follows price movements

### WebSocket Integration

The WebSocket integration will:
1. Subscribe to public market data for all supported symbols
2. Create user-specific private WebSocket connections on demand
3. Process updates in dedicated async tasks
4. Maintain cache of latest market data
5. Implement automatic reconnection with backoff

### Error Handling Strategy

1. Map Orderly API error codes to meaningful messages
2. Implement retry for transient errors (network, timeout, rate limit)
3. Log detailed error information including stack traces
4. Provide context-specific error messages to clients

## Success Criteria

1. All Orderly API endpoints properly integrated into SOR
2. Algo orders working correctly with proper validation
3. Real-time updates via WebSockets with reconnection
4. Comprehensive test coverage (>90%)
5. Documented API with examples
6. Performance benchmarks meeting latency targets (<100ms median) 