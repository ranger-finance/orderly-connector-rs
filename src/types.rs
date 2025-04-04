use serde::{Deserialize, Serialize};

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

// --- Request Structs ---

/// Request structure for creating a new order.
///
/// # Fields
///
/// * `symbol` - The trading pair symbol (e.g., "PERP_ETH_USDC")
/// * `order_type` - The type of order to create
/// * `side` - Whether to buy or sell
/// * `order_price` - The price for limit orders (optional for market orders)
/// * `order_quantity` - The quantity to buy/sell
/// * `order_amount` - The total amount to spend (optional)
/// * `client_order_id` - Optional client-specified order ID
/// * `visible_quantity` - Optional visible quantity for iceberg orders
#[derive(Serialize, Debug, Clone)]
pub struct CreateOrderRequest<'a> {
    pub symbol: &'a str,
    pub order_type: OrderType,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_price: Option<f64>,
    pub order_quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<&'a str>,
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
pub struct GetOrdersParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<&'a str>,
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
/// * `client_order_id` - Optional client-specified order ID
/// * `symbol` - The trading pair symbol
/// * `side` - The order side (buy/sell)
/// * `order_type` - The type of order
/// * `order_price` - The order price (for limit orders)
/// * `order_quantity` - The order quantity
/// * `order_amount` - The order amount
/// * `status` - The current order status
/// * `executed_quantity` - The quantity that has been executed
/// * `executed_value` - The value of executed quantity
/// * `average_executed_price` - The average execution price
/// * `total_fee` - The total fee for the order
/// * `fee_asset` - The asset in which fees are paid
/// * `visible_quantity` - The visible quantity (for iceberg orders)
/// * `created_time` - The timestamp when the order was created
/// * `updated_time` - The timestamp when the order was last updated
#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    pub order_id: u64,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub order_price: Option<f64>,
    pub order_quantity: Option<f64>,
    pub order_amount: Option<f64>,
    pub status: OrderStatus,
    pub executed_quantity: Option<f64>,
    pub executed_value: Option<f64>,
    pub average_executed_price: Option<f64>,
    pub total_fee: Option<f64>,
    pub fee_asset: Option<String>,
    pub visible_quantity: Option<f64>,
    pub created_time: u64,
    pub updated_time: u64,
    // Add reduce_only, source, trigger_price etc. if present in actual response
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
    pub email: String,
    pub market_type: Option<String>, // SPOT or FUTURES
    pub leverage: f64,
    pub max_leverage: f64,
    pub maintenance_margin_ratio: f64,
    pub imr_factor: Option<f64>,
    pub max_notional: Option<f64>,
    pub free_collateral: f64,
    pub total_collateral: f64,
    pub total_collateral_value: Option<f64>, // Added based on potential API responses
    pub total_pnl: Option<f64>,              // Added based on potential API responses
    pub imr_withdraw_safe: Option<f64>,      // Added based on potential API responses
    pub mmr_withdraw_safe: Option<f64>,      // Added based on potential API responses
                                             // ... other fields as per API documentation
}

pub type GetAccountInfoResponse = SuccessResponse<AccountInfo>;

// --- Holdings / Balances ---

#[derive(Deserialize, Debug, Clone)]
pub struct Holding {
    pub token: String,
    pub holding: f64,                   // Total balance
    pub frozen: f64,                    // Amount locked in orders
    pub pending_short_qty: Option<f64>, // For futures?
    pub pending_long_qty: Option<f64>,  // For futures?
    pub available_balance: f64,         // holding - frozen
    pub cost_position: Option<f64>,
    pub mark_price: Option<f64>,
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
    pub unrealized_pnl: f64, // Also called unrealised_pnl
    pub mark_price: f64,
    pub liquidation_price: Option<f64>, // Can be null if position_qty is 0
    pub average_open_price: f64,
    pub timestamp: u64,
    pub fee_24_h: Option<f64>,       // Added
    pub settlement_pnl: Option<f64>, // Added
    pub est_liq_price: Option<f64>,  // Added, alternative name?
                                     // ... other fields
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
pub struct GetAssetHistoryParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<&'a str>,
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
    pub id: u64,
    pub token: String,
    pub side: AssetHistoryType, // Matches the enum
    pub amount: f64,
    pub fee: Option<f64>,
    pub status: String, // e.g., "COMPLETED", "PENDING", "FAILED"
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
pub struct GetTradesParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<&'a str>,
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
pub struct GetLiquidationsParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<&'a str>,
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
pub struct GetSettlementsParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<&'a str>,
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
