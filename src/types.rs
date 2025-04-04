use serde::{Deserialize, Serialize};

// --- Enums ---

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

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

// General success response structure often includes success:bool and data:T
#[derive(Deserialize, Debug, Clone)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: u64,
}

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
