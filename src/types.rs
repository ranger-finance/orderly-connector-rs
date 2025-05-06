use serde::de::Deserializer;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt; // Added for precise price/quantity representation

// --- Enums ---

/// Represents the different types of orders supported by the Orderly Network.
///
/// # Variants
///
/// * `Limit` - A limit order that executes at a specified price or better
/// * `Market` - A market order that executes at the current market price
/// * `Ioc` - Immediate or Cancel order that executes immediately or is cancelled
/// * `Fok` - Fill or Kill order that must be filled completely or cancelled
/// * `PostOnly` - Order that only adds liquidity to the order book
/// * `Ask` - A limit sell order
/// * `Bid` - A limit buy order
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    Ioc,
    Fok,
    PostOnly,
    Ask,
    Bid,
}

/// Represents the side of an order (buy or sell).
///
/// # Variants
///
/// * `Buy` - A buy order
/// * `Sell` - A sell order
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

/// Represents the current status of an order.
///
/// # Variants
///
/// * `New` - Order has been created but not yet accepted by the matching engine
/// * `Accepted` - Order has been accepted by the matching engine
/// * `Filled` - Order has been completely filled
/// * `Cancelled` - Order has been cancelled
/// * `Rejected` - Order has been rejected
/// * `Expired` - Order has expired
/// * `PartialFilled` - Order has been partially filled
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    New,      // Pending Create
    Accepted, // Accepted by matching engine
    Filled,
    Cancelled,
    Rejected,
    Expired,
    PartialFilled,
    // There might be more statuses, add as needed based on API docs
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Types of algorithmic orders supported by Orderly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlgoOrderType {
    StopMarket,
    StopLimit,
    TakeProfitMarket,
    TakeProfitLimit,
    TrailingStop,
}

/// Represents the time in force for an order.
/// Note: Some TIF values might be handled by OrderType (e.g., IOC, FOK).
/// This enum covers common explicit TIF settings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderlyTimeInForce {
    Gtc, // Good 'Til Canceled
         // Add others if supported explicitly via a TIF field, e.g., Gtd (Good 'Til Date)
}

// --- Request Structs ---

/// Request structure for creating a new order.
///
/// # Understanding Trading Pairs
///
/// In a trading pair like "PERP_ETH_USDC":
/// * Base currency (ETH) - The asset you are buying or selling
/// * Quote currency (USDC) - The asset used to price and pay for the base currency
///
/// For example:
/// * When buying ETH with USDC: order_quantity is in ETH (base), order_amount is in USDC (quote)
/// * When selling ETH for USDC: order_quantity is in ETH (base)
///
/// # Order Type Behaviors
///
/// * `Market` - Matches until full size is executed. If size is too large or exceeds price limit,
///   remaining quantity is cancelled.
/// * `Ioc` (Immediate or Cancel) - Matches as much as possible at order_price. Remaining quantity
///   is cancelled if not fully executed.
/// * `Fok` (Fill or Kill) - Either fully executed at order_price or completely cancelled.
/// * `PostOnly` - Cancelled without execution if it would match with any maker trades.
/// * `Ask` - Order price guaranteed to be best ask price when accepted.
/// * `Bid` - Order price guaranteed to be best bid price when accepted.
///
/// # Special Parameter Behaviors
///
/// * `visible_quantity` - Maximum quantity shown on orderbook. Defaults to order_quantity.
///   - Must not be negative or larger than order_quantity
///   - If 0, order is hidden from orderbook
///   - Not applicable for Market/IOC/FOK orders
///
/// * `order_amount` - Alternative to order_quantity for Market/Bid/Ask orders
///   - Specifies order size in quote currency (e.g., USDC) instead of base currency
///   - Cannot be used together with order_quantity (order will be rejected)
///   - Must have 8 or fewer decimal places
///   - For BUY orders: use order_amount (specify USDC amount)
///   - For SELL orders: use order_quantity (specify base currency amount)
///
/// * `client_order_id` - Custom unique ID for open orders
///   - Must be unique among open orders
///   - New orders with duplicate client_order_id are accepted only after previous one completes
///
/// * `order_tag` (Optional): A user-defined tag for the order.
///   Reference: https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/private/create-order
///
/// # Fields
///
/// * `symbol` - The trading pair symbol (e.g., "PERP_ETH_USDC")
/// * `order_type` - The type of order to create
/// * `side` - Whether to buy or sell
/// * `order_price` - The price for limit orders (optional for market orders)
/// * `order_quantity` - The quantity to buy/sell in base currency (e.g., ETH in PERP_ETH_USDC)
/// * `order_amount` - The total amount in quote currency (e.g., USDC in PERP_ETH_USDC)
/// * `client_order_id` - Optional client-specified order ID (36 chars max, can include hyphens)
/// * `visible_quantity` - Optional visible quantity for iceberg orders
/// Reference: https://orderly.network/docs/build-on-omnichain/evm-api/restful-api/private/create-order
#[derive(Serialize, Debug, Clone)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub order_type: OrderType,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_quantity: Option<f64>,
    // Add other optional fields like reduce_only, trigger_price etc. if needed
}

/// Parameters for retrieving multiple orders.
///
/// # Fields
///
/// * `symbol` - Optional symbol to filter orders
/// * `side` - Optional side to filter orders
/// * `order_type` - Optional order type to filter
/// * `status` - Optional order status to filter
/// * `start_t` - Optional start timestamp in milliseconds
/// * `end_t` - Optional end timestamp in milliseconds
/// * `page` - Optional page number for pagination
/// * `size` - Optional number of orders per page
#[derive(Serialize, Debug, Clone, Default)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    // Add is_triggered etc. if needed
}

/// Request parameters for creating an algorithmic order
#[derive(Debug, Clone, Serialize)]
pub struct CreateAlgoOrderRequest {
    pub symbol: String,
    pub order_type: AlgoOrderType,
    pub side: Side,
    pub quantity: f64,
    pub trigger_price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
}

/// Parameters for querying algorithmic orders
#[derive(Debug, Clone, Serialize, Default)]
pub struct GetAlgoOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<AlgoOrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

// --- Response Structs ---

/// A generic success response structure from the Orderly API.
///
/// # Type Parameters
///
/// * `T` - The type of data contained in the response
///
/// # Fields
///
/// * `success` - Whether the request was successful
/// * `data` - The response data
/// * `timestamp` - The server timestamp when the response was generated
#[derive(Deserialize, Debug, Clone)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
}

/// Represents an order in the Orderly Network.
///
/// # Fields
///
/// * `order_id` - The unique order ID
/// * `user_id` - The user ID associated with the order
/// * `client_order_id` - Optional client-specified order ID
/// * `symbol` - The trading pair symbol
/// * `side` - The order side (buy/sell)
/// * `order_type` - The type of order
/// * `price` - The order price (for limit orders)
/// * `quantity` - The order quantity
/// * `amount` - The order amount
/// * `executed_quantity` - The quantity that has been executed
/// * `total_executed_quantity` - The total quantity that has been executed
/// * `visible_quantity` - The visible quantity (for iceberg orders)
/// * `status` - The current order status
/// * `total_fee` - The total fee for the order
/// * `fee_asset` - The asset in which fees are paid
/// * `average_executed_price` - The average execution price
/// * `created_time` - The timestamp when the order was created
/// * `updated_time` - The timestamp when the order was last updated
/// * `realized_pnl` - The realized profit and loss for the order
#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    pub order_id: u64,
    pub user_id: u64,
    #[serde(deserialize_with = "crate::types::de_client_order_id_opt")]
    // custom deserializer for string/number
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: Side,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: f64,
    pub amount: Option<f64>,
    pub executed_quantity: f64,
    pub total_executed_quantity: f64,
    pub visible_quantity: f64,
    pub status: OrderStatus,
    pub total_fee: f64,
    pub fee_asset: String,
    pub average_executed_price: f64,
    pub created_time: u64,
    pub updated_time: u64,
    pub realized_pnl: f64,
}

/// Response structure for algorithmic order details
#[derive(Debug, Clone, Deserialize)]
pub struct AlgoOrderDetails {
    pub algo_order_id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub order_type: AlgoOrderType,
    pub side: Side,
    pub quantity: f64,
    pub trigger_price: f64,
    pub limit_price: Option<f64>,
    pub trailing_delta: Option<f64>,
    pub status: OrderStatus,
    pub reduce_only: bool,
    pub triggered_order_id: Option<String>,
    pub created_time: i64,
    pub updated_time: i64,
}

/// Response structure for a list of algorithmic orders
#[derive(Debug, Clone, Deserialize)]
pub struct GetAlgoOrdersResponse {
    pub rows: Vec<AlgoOrderDetails>,
    pub total: u32,
    pub current_page: u32,
    pub page_size: u32,
}

/// Response data for a create order request.
///
/// # Fields
///
/// * `order_id` - The unique order ID assigned by the exchange
/// * `client_order_id` - Optional client-specified order ID
#[derive(Deserialize, Debug, Clone)]
pub struct CreateOrderResponseData {
    pub order_id: u64,
    pub client_order_id: Option<String>,
    // May contain other fields like order status, need to verify API docs
}

pub type CreateOrderResponse = SuccessResponse<CreateOrderResponseData>;

#[derive(Deserialize, Debug, Clone)]
pub struct GetOrderResponseData {
    // Often, getting a single order returns the Order struct directly within data
    #[serde(flatten)]
    pub order: Order,
}

pub type GetOrderResponse = SuccessResponse<GetOrderResponseData>;

#[derive(Deserialize, Debug, Clone)]
pub struct GetOrdersResponseData {
    pub rows: Vec<Order>,
    pub meta: Option<PaginationMeta>, // If pagination is included
}

#[derive(Deserialize, Debug, Clone)]
pub struct PaginationMeta {
    pub total: u32,
    pub current_page: u32,
    pub records_per_page: u32,
}

pub type GetOrdersResponse = SuccessResponse<GetOrdersResponseData>;

// Response for successful cancellation (often just success:true)
#[derive(Deserialize, Debug, Clone)]
pub struct CancelOrderResponseData {
    pub status: String, // e.g., "CANCEL_SENT" or similar
}

pub type CancelOrderResponse = SuccessResponse<CancelOrderResponseData>;

// --- Account Information ---

#[derive(Deserialize, Debug, Clone)]
pub struct AccountInfo {
    pub account_id: String,
    #[serde(default)]
    pub email: Option<String>,
    pub account_mode: String,
    #[serde(default)]
    pub maintenance_cancel_orders: Option<bool>,
    pub taker_fee_rate: f64,
    pub maker_fee_rate: f64,
    pub max_leverage: f64,
    pub futures_taker_fee_rate: f64,
    pub futures_maker_fee_rate: f64,
    pub imr_factor: std::collections::HashMap<String, f64>,
    pub max_notional: std::collections::HashMap<String, i64>,
}

pub type GetAccountInfoResponse = SuccessResponse<AccountInfo>;

// --- Holdings / Balances ---

#[derive(Deserialize, Debug, Clone)]
pub struct Holding {
    pub token: String,
    pub holding: f64,                   // Total balance
    pub frozen: f64,                    // Amount locked in orders
    pub pending_short_qty: Option<f64>, // For futures?
    pub updated_time: u64,
    // ... other fields like valuation, interest etc.
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetHoldingResponseData {
    pub holding: Vec<Holding>,
}

pub type GetHoldingResponse = SuccessResponse<GetHoldingResponseData>;

// --- Positions ---

#[derive(Deserialize, Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub position_qty: f64,
    pub cost_position: f64,
    pub last_sum_unitary_funding: f64,
    pub pending_long_qty: f64,
    pub pending_short_qty: f64,
    pub unsettled_pnl: f64,
    pub mark_price: f64,
    #[serde(default)]
    pub liquidation_price: Option<f64>,
    pub average_open_price: f64,
    pub timestamp: u64,
    pub fee_24_h: f64,
    #[serde(default)]
    pub settlement_pnl: Option<f64>,
    pub est_liq_price: f64,
    pub seq: u64,
    pub imr: f64,
    pub mmr: f64,
    #[serde(rename = "IMR_withdraw_orders")]
    pub imr_with_orders: f64,
    #[serde(rename = "MMR_with_orders")]
    pub mmr_with_orders: f64,
    pub pnl_24_h: f64,
    pub settle_price: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPositionsResponseData {
    pub rows: Vec<Position>,
    // Does positions endpoint have pagination? Check API docs. Assuming no for now.
    // pub meta: Option<PaginationMeta>,
}

pub type GetPositionsResponse = SuccessResponse<GetPositionsResponseData>;

#[derive(Deserialize, Debug, Clone)]
pub struct GetSinglePositionResponseData {
    // Getting a single position often returns the Position struct directly
    #[serde(flatten)]
    pub position: Position,
}

pub type GetSinglePositionResponse = SuccessResponse<GetSinglePositionResponseData>;

// --- Asset History (Deposits/Withdrawals) ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetHistoryType {
    Deposit,
    Withdrawal,
    // Other types like Transfer, Interest, RealizedPnl, Fee, FundingFee, etc.?
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct GetAssetHistoryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<AssetHistoryType>, // Type of transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    // Add status filters if applicable
}

#[derive(Deserialize, Debug, Clone)]
pub struct AssetHistoryEntry {
    pub id: String,
    pub token: String,
    pub side: AssetHistoryType, // Matches the enum
    pub amount: f64,
    pub fee: Option<f64>,
    pub transaction_hash: Option<String>,
    pub chain_id: Option<String>, // Or u64?
    pub chain_name: Option<String>,
    pub created_time: u64,
    pub updated_time: u64,
    // ... other fields like address, network etc.
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAssetHistoryResponseData {
    pub rows: Vec<AssetHistoryEntry>,
    pub meta: Option<PaginationMeta>,
}

pub type GetAssetHistoryResponse = SuccessResponse<GetAssetHistoryResponseData>;

// --- Trades ---

#[derive(Serialize, Debug, Clone, Default)]
pub struct GetTradesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    // Add order_id, source filters if applicable
}

#[derive(Deserialize, Debug, Clone)]
pub struct Trade {
    pub id: u64,
    pub symbol: String,
    pub side: Side,
    pub order_id: u64,
    pub order_source: Option<String>, // e.g., "API", "WEB"
    pub executed_price: f64,
    pub executed_quantity: f64,
    pub fee: f64,
    pub fee_asset: String,
    pub is_maker: bool,
    pub executed_timestamp: u64, // Also called transaction_time?
                                 // ... other fields
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetTradesResponseData {
    pub rows: Vec<Trade>,
    pub meta: Option<PaginationMeta>,
}

pub type GetTradesResponse = SuccessResponse<GetTradesResponseData>;

// --- Client Statistics ---

#[derive(Deserialize, Debug, Clone)]
pub struct ClientStatistics {
    pub account_id: String,
    pub total_trading_volume_30_d: f64, // Assuming f64 for volume
    pub futures_trading_volume_30_d: f64,
    pub spot_trading_volume_30_d: f64,
    pub total_fee_30_d: f64,
    pub vip_tier: Option<u32>, // Or String?
                               // ... other stats fields
}

pub type GetClientStatisticsResponse = SuccessResponse<ClientStatistics>;

// --- Add other account-related structs as needed (e.g., Algo Orders, Liquidations) ---

// --- Withdrawals ---

// Note: Withdrawal history is often fetched via get_asset_history using AssetHistoryType::Withdrawal

#[derive(Serialize, Debug, Clone)]
pub struct WithdrawRequest<'a> {
    pub chain_id: &'a str, // Or u64? API specific
    pub token: &'a str,
    pub amount: f64,
    pub withdraw_address: &'a str,
    pub message: Option<&'a str>, // Optional message/memo
                                  // Add other fields like twoFactorCode if required by API
}

#[derive(Deserialize, Debug, Clone)]
pub struct WithdrawResponseData {
    pub withdraw_id: u64, // Or String?
                          // Other potential fields confirming withdrawal request
}

pub type WithdrawResponse = SuccessResponse<WithdrawResponseData>;

// --- Fee Rates ---

#[derive(Deserialize, Debug, Clone)]
pub struct FeeRate {
    pub symbol: String,
    pub maker_fee_rate: f64,
    pub taker_fee_rate: f64,
    pub rebate_rate: Option<f64>,
    pub source: Option<String>, // e.g., "DEFAULT", "VIP"
    pub updated_time: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetFeeRatesResponseData {
    pub fee_rates: Vec<FeeRate>,
    pub taker_fee_rate_30_d: Option<f64>, // Overall 30d taker rate
    pub maker_fee_rate_30_d: Option<f64>, // Overall 30d maker rate
    pub volume_30_d: Option<f64>,         // Overall 30d volume
    pub vip_level: Option<u32>,           // VIP level if applicable
}

pub type GetFeeRatesResponse = SuccessResponse<GetFeeRatesResponseData>;

// --- Liquidations ---

#[derive(Serialize, Debug, Clone, Default)]
pub struct GetLiquidationsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LiquidationEntry {
    pub id: u64, // Liquidation record ID
    pub symbol: String,
    pub liquidation_price: f64,
    pub mark_price: f64,
    pub quantity: f64,
    pub amount: f64,
    pub liquidation_fee: f64,
    pub created_time: u64, // Timestamp ms of liquidation event
                           // ... other potential fields like cost_position, etc.
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetLiquidationsResponseData {
    pub rows: Vec<LiquidationEntry>,
    pub meta: Option<PaginationMeta>,
}

pub type GetLiquidationsResponse = SuccessResponse<GetLiquidationsResponseData>;

// --- PnL Settlement ---

#[derive(Serialize, Debug, Clone, Default)]
pub struct GetSettlementsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SettlementEntry {
    pub id: u64, // Settlement record ID
    pub symbol: String,
    pub settlement_price: f64,
    pub settlement_pnl: f64,
    pub timestamp: u64, // Timestamp ms of settlement
                        // ... other fields like funding fee component?
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetSettlementsResponseData {
    pub rows: Vec<SettlementEntry>,
    pub meta: Option<PaginationMeta>,
}

pub type GetSettlementsResponse = SuccessResponse<GetSettlementsResponseData>;

// ===== Funding Fee =====

/// Parameters for retrieving funding fee history.
///
/// # Fields
///
/// * `start_t` - Optional start timestamp in milliseconds.
/// * `end_t` - Optional end timestamp in milliseconds.
/// * `page` - Optional page number for pagination.
/// * `size` - Optional number of records per page (Default: 60).
#[derive(Serialize, Debug, Clone, Default)]
pub struct GetFundingFeeParams {
    // Symbol is passed directly in the path, not as a query param here.
    // pub symbol: Option<&'a str>, // This is incorrect based on client implementation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_t: Option<u64>, // Timestamp ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    // Ensure the lifetime is used if needed, though serde_qs might handle owned data better.
    // Using PhantomData if no borrowed fields remain, but start_t/end_t etc. are owned.
    // #[serde(skip_serializing)] // Skip serializing this marker
    // _marker: std::marker::PhantomData<&'a ()>, // Use PhantomData to satisfy lifetime checker
}

#[derive(Deserialize, Debug, Clone)]
pub struct FundingFeeEntry {
    pub id: u64, // Assuming an ID field exists
    pub symbol: String,
    pub funding_rate: f64,
    pub funding_fee: f64,             // Amount paid/received
    pub payment_type: Option<String>, // e.g., "Pay", "Receive"
    pub position_qty: Option<f64>,    // Position size at the time
    pub mark_price: Option<f64>,      // Mark price at the time
    pub timestamp: u64,               // Timestamp of the funding event
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetFundingFeeHistoryResponseData {
    pub rows: Vec<FundingFeeEntry>,
    pub meta: Option<PaginationMeta>,
}

// Define a proper response struct for GetFundingFeeHistoryResponse
pub type GetFundingFeeHistoryResponse = SuccessResponse<GetFundingFeeHistoryResponseData>;

// ===== Funding Rate History =====

#[derive(Deserialize, Debug, Clone)]
pub struct FundingRateData {
    pub rate: f64,
    pub positive: i32,
    pub negative: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FundingRateHistory {
    pub symbol: String,
    pub data_start_time: String,
    pub funding: FundingRateHistoryPeriods,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FundingRateHistoryPeriods {
    pub last: FundingRateData,
    #[serde(rename = "1d")]
    pub one_day: Option<FundingRateData>,
    #[serde(rename = "3d")]
    pub three_day: Option<FundingRateData>,
    #[serde(rename = "7d")]
    pub seven_day: Option<FundingRateData>,
    #[serde(rename = "14d")]
    pub fourteen_day: Option<FundingRateData>,
    #[serde(rename = "30d")]
    pub thirty_day: Option<FundingRateData>,
    #[serde(rename = "90d")]
    pub ninety_day: Option<FundingRateData>,
    #[serde(rename = "180d")]
    pub one_eighty_day: Option<FundingRateData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetFundingRateHistoryResponseData {
    pub rows: Vec<FundingRateHistory>,
}

// Iterator implementation for response data
impl IntoIterator for GetFundingRateHistoryResponseData {
    type Item = FundingRateHistory;
    type IntoIter = std::vec::IntoIter<FundingRateHistory>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

// Reference iterator implementation for response data
impl<'a> IntoIterator for &'a GetFundingRateHistoryResponseData {
    type Item = &'a FundingRateHistory;
    type IntoIter = std::slice::Iter<'a, FundingRateHistory>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

pub type GetFundingRateHistoryResponse = SuccessResponse<GetFundingRateHistoryResponseData>;

// Iterator implementation for funding rate history
pub struct FundingRateHistoryIterator {
    response: GetFundingRateHistoryResponse,
    index: usize,
}

impl Iterator for FundingRateHistoryIterator {
    type Item = FundingRateHistory;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.response.data.rows.len() {
            let item = self.response.data.rows[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl IntoIterator for GetFundingRateHistoryResponse {
    type Item = FundingRateHistory;
    type IntoIter = FundingRateHistoryIterator;

    fn into_iter(self) -> Self::IntoIter {
        FundingRateHistoryIterator {
            response: self,
            index: 0,
        }
    }
}

// Also implement iterator for reference to avoid consuming the response
impl<'a> IntoIterator for &'a GetFundingRateHistoryResponse {
    type Item = &'a FundingRateHistory;
    type IntoIter = std::slice::Iter<'a, FundingRateHistory>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.rows.iter()
    }
}

// ===== Open Interest =====

/// Represents open interest data for a trading pair.
///
/// # Fields
///
/// * `symbol` - The trading pair symbol (e.g., "PERP_BTC_USDC")
/// * `long_oi` - Total long open interest, expected to be non-negative
/// * `short_oi` - Total short open interest, represented as a negative number or zero
#[derive(Deserialize, Debug, Clone)]
pub struct OpenInterest {
    pub symbol: String,
    /// Total long open interest, expected to be non-negative
    pub long_oi: f64,
    /// Total short open interest, represented as a negative number or zero
    pub short_oi: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetOpenInterestResponseData {
    pub rows: Vec<OpenInterest>,
}

// Iterator implementation for response data
impl IntoIterator for GetOpenInterestResponseData {
    type Item = OpenInterest;
    type IntoIter = std::vec::IntoIter<OpenInterest>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

// Reference iterator implementation for response data
impl<'a> IntoIterator for &'a GetOpenInterestResponseData {
    type Item = &'a OpenInterest;
    type IntoIter = std::slice::Iter<'a, OpenInterest>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

pub type GetOpenInterestResponse = SuccessResponse<GetOpenInterestResponseData>;

// Iterator implementation for open interest
pub struct OpenInterestIterator {
    response: GetOpenInterestResponse,
    index: usize,
}

impl Iterator for OpenInterestIterator {
    type Item = OpenInterest;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.response.data.rows.len() {
            let item = self.response.data.rows[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl IntoIterator for GetOpenInterestResponse {
    type Item = OpenInterest;
    type IntoIter = OpenInterestIterator;

    fn into_iter(self) -> Self::IntoIter {
        OpenInterestIterator {
            response: self,
            index: 0,
        }
    }
}

// Also implement iterator for reference to avoid consuming the response
impl<'a> IntoIterator for &'a GetOpenInterestResponse {
    type Item = &'a OpenInterest;
    type IntoIter = std::slice::Iter<'a, OpenInterest>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.rows.iter()
    }
}

// ===== Algo Orders =====

/// Represents a single position within a liquidation event.
#[derive(Deserialize, Debug, Clone)]
pub struct PositionByPerp {
    pub symbol: String,
    pub position_qty: f64,
    pub liquidator_fee: f64,
}

/// Represents a single liquidation event row.
#[derive(Deserialize, Debug, Clone)]
pub struct LiquidationPositionRow {
    pub timestamp: u64, // 13-digit timestamp
    #[serde(rename = "type")]
    pub event_type: String, // "liquidated"
    pub liquidation_id: u64,
    pub positions_by_perp: Vec<PositionByPerp>,
}

/// Metadata associated with paginated liquidation responses.
#[derive(Deserialize, Debug, Clone)]
pub struct LiquidationMeta {
    pub total: u32,
    pub records_per_page: u32,
    pub current_page: u32,
}

/// Data structure for the Get Positions Under Liquidation response.
#[derive(Deserialize, Debug, Clone)]
pub struct GetPositionsUnderLiquidationData {
    pub meta: LiquidationMeta,
    pub rows: Vec<LiquidationPositionRow>,
}

/// Response structure for the Get Positions Under Liquidation endpoint.
#[derive(Deserialize, Debug, Clone)]
pub struct GetPositionsUnderLiquidationResponse {
    pub success: bool,
    pub timestamp: u64, // 13-digit timestamp
    pub data: GetPositionsUnderLiquidationData,
}

// === Price Changes ===

/// Represents price information for a symbol at different time intervals
#[derive(Deserialize, Debug, Clone)]
pub struct PriceChange {
    pub symbol: String,
    pub last_price: f64,
    #[serde(rename = "5m")]
    pub five_min: Option<f64>,
    #[serde(rename = "30m")]
    pub thirty_min: Option<f64>,
    #[serde(rename = "1h")]
    pub one_hour: Option<f64>,
    #[serde(rename = "4h")]
    pub four_hour: Option<f64>,
    #[serde(rename = "24h")]
    pub twenty_four_hour: Option<f64>,
    #[serde(rename = "3d")]
    pub three_day: Option<f64>,
    #[serde(rename = "7d")]
    pub seven_day: Option<f64>,
    #[serde(rename = "30d")]
    pub thirty_day: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPriceChangesResponseData {
    pub rows: Vec<PriceChange>,
}

// Iterator implementation for response data
impl IntoIterator for GetPriceChangesResponseData {
    type Item = PriceChange;
    type IntoIter = std::vec::IntoIter<PriceChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

// Reference iterator implementation for response data
impl<'a> IntoIterator for &'a GetPriceChangesResponseData {
    type Item = &'a PriceChange;
    type IntoIter = std::slice::Iter<'a, PriceChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

pub type GetPriceChangesResponse = SuccessResponse<GetPriceChangesResponseData>;

// Iterator implementation for price changes
pub struct PriceChangeIterator {
    response: GetPriceChangesResponse,
    index: usize,
}

impl Iterator for PriceChangeIterator {
    type Item = PriceChange;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.response.data.rows.len() {
            let item = self.response.data.rows[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl IntoIterator for GetPriceChangesResponse {
    type Item = PriceChange;
    type IntoIter = PriceChangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        PriceChangeIterator {
            response: self,
            index: 0,
        }
    }
}

// Also implement iterator for reference to avoid consuming the response
impl<'a> IntoIterator for &'a GetPriceChangesResponse {
    type Item = &'a PriceChange;
    type IntoIter = std::slice::Iter<'a, PriceChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.rows.iter()
    }
}

// === Get Liquidated Positions ===

/// Optional parameters for querying liquidated positions.
#[derive(Serialize, Debug, Default, Clone)]
pub struct GetLiquidatedPositionsParams {
    pub symbol: Option<String>,
    pub start_t: Option<u64>, // 13-digit timestamp
    pub end_t: Option<u64>,   // 13-digit timestamp
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// Represents a single position within a liquidation event.
#[derive(Deserialize, Debug, Clone)]
pub struct LiquidatedPositionByPerp {
    pub symbol: String,
    pub seq: Option<u64>,
    pub position_qty: f64,
    pub liquidator_fee: f64,
    pub cost_position_transfer: f64,
    pub transfer_price: f64,
    pub insurance_fund_fee: f64,
    pub abs_insurance_fund_fee: f64,
    pub abs_liquidator_fee: Option<f64>,
}

/// Represents a single liquidation event row.
#[derive(Deserialize, Debug, Clone)]
pub struct LiquidatedPositionRow {
    pub timestamp: u64,
    pub liquidation_id: u64,
    pub transfer_amount_to_insurance_fund: f64,
    #[serde(rename = "type")]
    pub event_type: String, // "liquidated"
    pub positions_by_perp: Vec<LiquidatedPositionByPerp>,
}

/// Metadata associated with paginated liquidation responses.
#[derive(Deserialize, Debug, Clone)]
pub struct LiquidatedPositionMeta {
    pub total: u32,
    pub records_per_page: u32,
    pub current_page: u32,
}

/// Data structure for the Get Liquidated Positions response.
#[derive(Deserialize, Debug, Clone)]
pub struct GetLiquidatedPositionsData {
    pub meta: LiquidatedPositionMeta,
    pub rows: Vec<LiquidatedPositionRow>,
}

/// Response structure for the Get Liquidated Positions endpoint.
#[derive(Deserialize, Debug, Clone)]
pub struct GetLiquidatedPositionsResponse {
    pub success: bool,
    pub timestamp: u64,
    pub data: GetLiquidatedPositionsData,
}

// ===== WebSocket Subscription Types =====

/// WebSocket subscription request message
#[derive(Serialize, Debug, Clone)]
pub struct WebSocketSubscriptionRequest {
    #[serde(default = "default_subscription_id")]
    pub id: String,
    pub event: String,
    pub topic: String,
}

impl Default for WebSocketSubscriptionRequest {
    fn default() -> Self {
        Self {
            id: default_subscription_id(),
            event: "subscribe".to_string(),
            topic: "liquidation".to_string(),
        }
    }
}

fn default_subscription_id() -> String {
    "sub_liquidations".to_string()
}

/// WebSocket subscription response
#[derive(Deserialize, Debug, Clone)]
pub struct WebSocketSubscriptionResponse {
    #[serde(default = "default_subscription_id")]
    pub id: String,
    #[serde(default = "default_subscription_event")]
    pub event: String,
    #[serde(default = "default_subscription_success")]
    pub success: bool,
    pub ts: u64,
}

fn default_subscription_event() -> String {
    "subscribe".to_string()
}

fn default_subscription_success() -> bool {
    false
}

/// WebSocket liquidation message
#[derive(Deserialize, Debug, Clone)]
pub struct WebSocketLiquidationMessage {
    pub topic: String,
    pub ts: u64,
    pub data: Vec<WebSocketLiquidationData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebSocketLiquidationData {
    #[serde(default)]
    pub liquidation_id: u64,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default = "default_liquidation_type")]
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub positions_by_perp: Vec<WebSocketPositionByPerp>,
}

fn default_liquidation_type() -> String {
    "liquidated".to_string()
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebSocketPositionByPerp {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub position_qty: f64,
    #[serde(default)]
    pub liquidator_fee: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PublicTradeData {
    pub symbol: String,
    pub side: String,
    pub executed_price: f64,
    pub executed_quantity: f64,
    pub executed_timestamp: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetPublicTradesResponseData {
    pub rows: Vec<PublicTradeData>,
}

pub type GetPublicTradesResponse = SuccessResponse<GetPublicTradesResponseData>;

#[derive(Deserialize, Debug, Clone)]
pub struct WebSocketTradeData {
    pub topic: String,
    pub ts: u64,
    pub data: TradeData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TradeData {
    pub symbol: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
}

/// Optional parameters for querying positions under liquidation.
#[derive(Serialize, Debug, Default, Clone)]
pub struct GetPositionsUnderLiquidationParams {
    pub symbol: Option<String>,
    pub start_t: Option<u64>, // 13-digit timestamp
    pub end_t: Option<u64>,   // 13-digit timestamp
    pub page: Option<u32>,
    pub size: Option<u32>,
}

// --- WebSocket Message Structs ---

/// Represents a single level in the order book (price and quantity).
#[derive(Debug, Clone)]
pub struct OrderbookLevel {
    pub price: f64,
    pub quantity: f64,
}

impl<'de> Deserialize<'de> for OrderbookLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderbookLevelVisitor;

        impl<'de> Visitor<'de> for OrderbookLevelVisitor {
            type Value = OrderbookLevel;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array [price, quantity] or a map {price, quantity}")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let price = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let quantity = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                Ok(OrderbookLevel { price, quantity })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut price = None;
                let mut quantity = None;
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "price" => price = Some(map.next_value()?),
                        "quantity" => quantity = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let price = price.ok_or_else(|| serde::de::Error::missing_field("price"))?;
                let quantity =
                    quantity.ok_or_else(|| serde::de::Error::missing_field("quantity"))?;
                Ok(OrderbookLevel { price, quantity })
            }
        }

        deserializer.deserialize_any(OrderbookLevelVisitor)
    }
}

/// Represents an order book update received via WebSocket.
/// This could be a snapshot or an incremental update.
#[derive(Deserialize, Debug, Clone)]
pub struct OrderbookUpdate {
    pub topic: String, // e.g., "orderbook:PERP_BTC_USDC"
    pub ts: u64,       // Timestamp of the update
    pub data: OrderbookData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderbookData {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub bids: Vec<OrderbookLevel>, // List of bid levels
    #[serde(default)]
    pub asks: Vec<OrderbookLevel>, // List of ask levels
    #[serde(rename = "checksum")]
    #[serde(default)]
    pub checksum: Option<u32>, // Optional checksum for verification
    #[serde(rename = "lastUpdateId")]
    #[serde(default)]
    pub last_update_id: Option<u64>, // Identifier for the update sequence
    // Add prevTs as optional for @orderbookupdate
    #[serde(rename = "prevTs")]
    #[serde(default)]
    pub prev_ts: Option<u64>,
}

/// Represents ticker data received via WebSocket.
#[derive(Deserialize, Debug, Clone)]
pub struct Ticker {
    pub topic: String, // e.g., "ticker:PERP_BTC_USDC"
    pub ts: u64,       // Timestamp
    pub data: TickerData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TickerData {
    pub symbol: String,
    #[serde(rename = "open")]
    pub open_price: Option<f64>,
    #[serde(rename = "high")]
    pub high_price: Option<f64>,
    #[serde(rename = "low")]
    pub low_price: Option<f64>,
    #[serde(rename = "close")]
    pub close_price: f64, // Last traded price
    #[serde(rename = "volume")]
    pub volume: Option<f64>, // 24h volume in base asset
    #[serde(rename = "amount")]
    pub amount: Option<f64>, // 24h volume in quote asset
    #[serde(rename = "count")]
    pub trade_count: Option<u64>, // Number of trades in 24h
                                  // Add other relevant fields like mark_price, index_price, funding_rate if included
}

/// Represents different types of parsed WebSocket messages from public streams.
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Orderbook(OrderbookData),
    Ticker(TickerData),
    Trade(TradeData),
    Liquidation(WebSocketLiquidationData),
    Ping { ts: u64 },
    Other,
}

impl<'de> serde::Deserialize<'de> for WebSocketMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;
        let value = Value::deserialize(deserializer)?;

        // Try to match on "event" for ping
        if let Some(event) = value.get("event") {
            if event == "ping" {
                if let Some(ts) = value.get("ts").and_then(|v| v.as_u64()) {
                    return Ok(WebSocketMessage::Ping { ts });
                }
            }
        }

        // Try to match on "topic"
        if let Some(topic) = value.get("topic").and_then(|v| v.as_str()) {
            match topic {
                t if t.starts_with("orderbook:") => {
                    let data: OrderbookUpdate =
                        serde_json::from_value(value.clone()).map_err(D::Error::custom)?;
                    Ok(WebSocketMessage::Orderbook(data.data))
                }
                t if t.ends_with("@orderbookupdate") => {
                    // Parse as OrderbookData directly from the data field
                    let data = value
                        .get("data")
                        .ok_or_else(|| D::Error::custom("missing data field"))?;
                    let ob: OrderbookData =
                        serde_json::from_value(data.clone()).map_err(D::Error::custom)?;
                    Ok(WebSocketMessage::Orderbook(ob))
                }
                t if t.starts_with("ticker:") => {
                    let data: Ticker =
                        serde_json::from_value(value.clone()).map_err(D::Error::custom)?;
                    Ok(WebSocketMessage::Ticker(data.data))
                }
                t if t.starts_with("trade:") => {
                    let data: WebSocketTradeData =
                        serde_json::from_value(value.clone()).map_err(D::Error::custom)?;
                    Ok(WebSocketMessage::Trade(data.data))
                }
                t if t.starts_with("liquidation") => {
                    let data: WebSocketLiquidationMessage =
                        serde_json::from_value(value.clone()).map_err(D::Error::custom)?;
                    if let Some(first) = data.data.into_iter().next() {
                        Ok(WebSocketMessage::Liquidation(first))
                    } else {
                        Ok(WebSocketMessage::Other)
                    }
                }
                _ => Ok(WebSocketMessage::Other),
            }
        } else {
            Ok(WebSocketMessage::Other)
        }
    }
}

/// Represents the REST orderbook snapshot response data.
#[derive(Deserialize, Debug, Clone)]
pub struct OrderbookSnapshotData {
    pub asks: Vec<OrderbookLevel>,
    pub bids: Vec<OrderbookLevel>,
    pub timestamp: u64,
}

/// Type alias for the REST orderbook snapshot response.
pub type GetOrderbookSnapshotResponse = SuccessResponse<OrderbookSnapshotData>;

// Response for GET /v1/public/wallet_registered
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletRegisteredResponse {
    pub success: bool,
    pub status: String,
    pub data: Option<WalletRegisteredData>, // Data might be null if not registered
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletRegisteredData {
    pub is_registered: bool,
    // Add other fields if the API returns more info
}

// Response for GET /v1/registration_nonce
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistrationNonceResponse {
    pub success: bool,
    pub status: String,
    pub data: RegistrationNonceData,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistrationNonceData {
    #[serde(rename = "registrationNonce")]
    pub registration_nonce: String,
}

// Request body for POST /v1/register_account
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterAccountRequest<'a> {
    pub message: RegisterAccountMessage<'a>,
    pub signature: &'a str,
    #[serde(rename = "userAddress")]
    pub user_address: &'a str, // Solana address string
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterAccountMessage<'a> {
    #[serde(rename = "brokerId")]
    pub broker_id: &'a str,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    #[serde(rename = "chainType")]
    pub chain_type: &'a str, // Should be "SOL"
    pub timestamp: u64,
    #[serde(rename = "registrationNonce")]
    pub registration_nonce: &'a str,
}

// Response for POST /v1/register_account
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterAccountResponse {
    pub success: bool,
    pub status: String,
    pub data: RegisterAccountData,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterAccountData {
    #[serde(rename = "accountId")]
    pub account_id: String,
    // Add other fields if the API returns more info
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WithdrawNonceResponse {
    pub success: bool,
    pub status: String,
    pub data: WithdrawNonceData,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WithdrawNonceData {
    #[serde(rename = "withdrawNonce")]
    pub withdraw_nonce: String,
}

/// Details of a specific trade as returned by GET /v1/trade/{trade_id}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TradeDetails {
    pub id: u64,
    pub symbol: String,
    pub fee: f64,
    pub fee_asset: String,
    pub side: String, // Consider using an enum if you want stricter typing
    pub order_id: u64,
    pub executed_price: f64,
    pub executed_quantity: f64,
    pub executed_timestamp: u64,
    pub is_maker: u8, // 1 or 0
    pub realized_pnl: f64,
    pub match_id: String,
}

/// Response for GET /v1/trade/{trade_id}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GetTradeResponse {
    pub success: bool,
    pub timestamp: u64,
    pub data: TradeDetails,
}

// Custom deserializer for client_order_id (string or number)
pub fn de_client_order_id_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct StringOrNumber;
    impl<'de> serde::de::Visitor<'de> for StringOrNumber {
        type Value = Option<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or number or null")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
        where
            D2: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_any(StringOrNumber)
        }
    }
    deserializer.deserialize_option(StringOrNumber)
}
