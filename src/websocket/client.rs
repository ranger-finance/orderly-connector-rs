use crate::auth; // Import the auth module
use crate::error::{OrderlyError, Result};
use futures_util::{SinkExt, StreamExt};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

const MAINNET_WS_PUBLIC_URL: &str = "wss://ws.orderly.network/ws/stream";
const TESTNET_WS_PUBLIC_URL: &str = "wss://testnet-ws.orderly.network/ws/stream";
const MAINNET_WS_PRIVATE_URL: &str = "wss://ws-private.orderly.network/v2/ws/private/stream";
const TESTNET_WS_PRIVATE_URL: &str =
    "wss://testnet-ws-private.orderly.network/v2/ws/private/stream";
const MAX_RETRIES: u32 = 30; // Max number of consecutive reconnect attempts
const RETRY_DELAY_SECS: u64 = 5; // Delay between reconnect attempts

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsocketClientConfig {
    pub base_url: String, // The specific WS URL (public/private, mainnet/testnet)
    pub orderly_key: Option<String>,
    pub orderly_secret: Option<String>,
    pub orderly_account_id: String,
    pub wss_id: Option<String>, // Optional request ID for WS messages
}

// Type alias for the WebSocket stream
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

// Type alias for shared subscription state
type SubscriptionState = Arc<Mutex<HashSet<String>>>;
// Type alias for shared sender channel
type SharedSender = Arc<Mutex<Option<mpsc::Sender<Message>>>>;

/// Spawns tasks to manage a WebSocket connection, returning handles.
///
/// Returns a tuple containing:
/// - `JoinHandle<()>`: Handle for the combined reader/writer task group.
/// - `mpsc::Sender<Message>`: Channel sender to send outbound messages (Text, Pong, Close, etc.).
async fn connect_managed(
    config: WebsocketClientConfig,
    on_message: Arc<dyn Fn(String) + Send + Sync + 'static>,
    on_close: Arc<dyn Fn() + Send + Sync + 'static>,
) -> Result<(JoinHandle<()>, mpsc::Sender<Message>)> {
    let url_obj = Url::parse(&config.base_url).map_err(OrderlyError::UrlParseError)?;

    info!("Connecting to WebSocket: {}", url_obj);
    let (ws_stream, response) = connect_async(url_obj.as_str())
        .await
        .map_err(|e| OrderlyError::WebsocketError(format!("WebSocket connection failed: {}", e)))?;
    info!(
        "WebSocket connected successfully. Response: {:?}",
        response.status()
    );

    let (write, mut read) = ws_stream.split();
    let write = Arc::new(Mutex::new(write)); // Wrap writer in Arc<Mutex> for shared access

    // Channel for sending outbound messages to the writer task
    let (tx, mut rx) = mpsc::channel::<Message>(32);
    let tx_clone_for_ping = tx.clone(); // Clone sender for the read task (to send pongs)

    // --- Writer Task ---
    // Reads messages from the channel and sends them to the WebSocket sink.
    let writer_handle = tokio::spawn({
        let write = Arc::clone(&write);
        async move {
            while let Some(message) = rx.recv().await {
                trace!("Sending WS message: {:?}", message.to_string()); // Avoid logging sensitive data
                let mut writer = write.lock().await;
                if let Err(e) = writer.send(message).await {
                    error!("WebSocket send error: {}. Stopping writer task.", e);
                    break;
                }
            }
            info!("WebSocket writer task finished.");
            // Optionally notify closure if writer fails?
        }
    });

    // --- Reader Task ---
    // Reads messages from the WebSocket stream, handles Pings, and calls callbacks.
    let reader_handle = tokio::spawn({
        let on_message = Arc::clone(&on_message);
        let on_close = Arc::clone(&on_close);
        async move {
            loop {
                match read.next().await {
                    Some(Ok(msg)) => match msg {
                        Message::Text(text) => {
                            trace!("Received WS Text: {}", text);
                            on_message(text);
                        }
                        Message::Binary(bin) => {
                            trace!("Received WS Binary: {:?}", bin);
                        }
                        Message::Ping(ping_data) => {
                            trace!("Received WS Ping, sending Pong via channel");
                            if tx_clone_for_ping
                                .send(Message::Pong(ping_data))
                                .await
                                .is_err()
                            {
                                error!("Failed to send Pong: writer channel closed.");
                                break;
                            }
                        }
                        Message::Pong(_) => {
                            trace!("Received WS Pong");
                        }
                        Message::Close(close_frame) => {
                            warn!("Received WS Close frame: {:?}", close_frame);
                            break; // Exit loop
                        }
                        Message::Frame(_) => { /* Ignore */ }
                    },
                    Some(Err(e)) => {
                        error!("WebSocket read error: {}", e);
                        break; // Exit loop on error
                    }
                    None => {
                        info!("WebSocket stream ended (read None).");
                        break; // Stream closed
                    }
                }
            }
            info!("WebSocket reader task finished.");
            on_close(); // Notify external listener
                        // Attempt to gracefully close the writer task by dropping the sender
            drop(tx_clone_for_ping);
        }
    });

    // Combine handles - if one fails, we might want to abort the other.
    // For simplicity now, just return the reader handle, assuming writer failure is handled internally.
    // A more robust approach might use select! or a dedicated manager task.
    let combined_handle = tokio::spawn(async move {
        tokio::select! {
            _ = reader_handle => { info!("Reader task completed."); },
            _ = writer_handle => { info!("Writer task completed."); },
        }
        info!("WebSocket combined task group finished.");
    });

    Ok((combined_handle, tx))
}

// --- Public Client ---

pub struct WebsocketPublicClient {
    // Shared sender to allow sending messages while connection task runs/reconnects
    shared_tx: SharedSender,
    // Shared state to store active subscriptions for resubscription
    subscriptions: SubscriptionState,
    // Handle to the main connection management task
    manager_handle: JoinHandle<()>,
}

impl WebsocketPublicClient {
    pub async fn connect(
        account_id: String,
        is_testnet: bool,
        on_message: Arc<dyn Fn(String) + Send + Sync + 'static>,
        on_close: Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Self> {
        let shared_tx: SharedSender = Arc::new(Mutex::new(None));
        let subscriptions: SubscriptionState = Arc::new(Mutex::new(HashSet::new()));

        let manager_handle = tokio::spawn({
            // Clone Arcs for the manager task
            let shared_tx = Arc::clone(&shared_tx);
            let subscriptions = Arc::clone(&subscriptions);
            let on_message = Arc::clone(&on_message);
            let on_close = Arc::clone(&on_close);
            let account_id = account_id.clone(); // Clone needed data

            async move {
                let mut retries = 0;
                loop {
                    let base_url = (if is_testnet {
                        TESTNET_WS_PUBLIC_URL
                    } else {
                        MAINNET_WS_PUBLIC_URL
                    })
                    .to_string();
                    let url_with_account = format!("{}/{}", base_url, account_id);
                    let config = WebsocketClientConfig {
                        base_url: url_with_account,
                        orderly_key: None,
                        orderly_secret: None,
                        orderly_account_id: account_id.clone(),
                        wss_id: None,
                    };

                    info!("[Manager] Attempting connection (Retry {})...", retries);
                    match connect_managed(config, Arc::clone(&on_message), Arc::clone(&on_close))
                        .await
                    {
                        Ok((handle, tx)) => {
                            info!("[Manager] Connection established successfully.");
                            retries = 0; // Reset retries on successful connection
                                         // Store the new sender
                            *shared_tx.lock().await = Some(tx.clone());

                            // --- Resubscribe to existing topics ---
                            let subs_guard = subscriptions.lock().await;
                            if !subs_guard.is_empty() {
                                info!("[Manager] Resubscribing to {} topics...", subs_guard.len());
                                for msg_str in subs_guard.iter() {
                                    if let Err(e) = tx.send(Message::Text(msg_str.clone())).await {
                                        error!("[Manager] Failed to send resubscription message '{}': {}. Aborting resubscribe.", msg_str, e);
                                        // Decide if we should break or continue?
                                        break; // Break resubscribe loop for this connection attempt
                                    }
                                }
                            }
                            drop(subs_guard);
                            // --- End Resubscribe ---

                            // Wait for this connection to end (disconnect/error)
                            handle.await.unwrap_or_else(|e| {
                                error!("[Manager] Connection task panicked: {}", e);
                            });
                            info!("[Manager] Connection task ended.");
                        }
                        Err(e) => {
                            error!("[Manager] Failed to establish connection: {}", e);
                        }
                    }

                    // Connection failed or handle finished, prepare for retry
                    *shared_tx.lock().await = None; // Clear sender
                    on_close(); // Notify external listener about disconnection

                    retries += 1;
                    if retries >= MAX_RETRIES {
                        error!(
                            "[Manager] Max retries ({}) reached. Stopping connection attempts.",
                            MAX_RETRIES
                        );
                        break; // Exit the manager loop
                    }

                    warn!(
                        "[Manager] Disconnected. Retrying in {} seconds...",
                        RETRY_DELAY_SECS
                    );
                    sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                }
                info!("[Manager] Task finished.");
            }
        });

        // Return the client struct immediately
        Ok(Self {
            shared_tx,
            subscriptions,
            manager_handle,
        })
    }

    /// Sends a raw JSON message to the WebSocket server if connected.
    async fn send_json(&self, json_value: Value) -> Result<()> {
        let msg_str = serde_json::to_string(&json_value)?;
        self.send_str(&msg_str).await
    }

    /// Sends a raw text message to the WebSocket server if connected.
    async fn send_str(&self, text: &str) -> Result<()> {
        let guard = self.shared_tx.lock().await;
        if let Some(tx) = guard.as_ref() {
            tx.send(Message::Text(text.to_string()))
                .await
                .map_err(|e| OrderlyError::WebsocketError(format!("Failed to send message: {}", e)))
        } else {
            Err(OrderlyError::WebsocketError("Not connected".to_string()))
        }
    }

    /// Helper to add subscription and send message.
    async fn subscribe(&self, topic_msg: Value) -> Result<()> {
        let msg_str = serde_json::to_string(&topic_msg)?;
        // Add to subscriptions *before* sending
        self.subscriptions.lock().await.insert(msg_str.clone());
        self.send_str(&msg_str).await
    }

    /// Helper to remove subscription and send message.
    async fn unsubscribe(&self, topic_msg: Value) -> Result<()> {
        let msg_str = serde_json::to_string(&topic_msg)?;
        // Remove from subscriptions *before* sending
        self.subscriptions.lock().await.remove(&msg_str);
        self.send_str(&msg_str).await
    }

    // --- Subscription Methods (Updated) ---

    pub async fn subscribe_tickers(&self) -> Result<()> {
        let msg = json!({
            "id": "subscribe_tickers",
            "topic": "tickers",
            "event": "subscribe"
        });
        self.subscribe(msg).await
    }

    pub async fn unsubscribe_tickers(&self) -> Result<()> {
        let msg = json!({
            "id": "unsubscribe_tickers",
            "topic": "tickers",
            "event": "unsubscribe"
        });
        self.unsubscribe(msg).await
    }

    pub async fn subscribe_orderbook(&self, symbol: &str) -> Result<()> {
        let topic = format!("orderbook@{}", symbol);
        let msg = json!({
            "id": format!("subscribe_orderbook_{}", symbol),
            "topic": topic,
            "event": "subscribe"
        });
        self.subscribe(msg).await
    }

    pub async fn unsubscribe_orderbook(&self, symbol: &str) -> Result<()> {
        let topic = format!("orderbook@{}", symbol);
        let msg = json!({
            "id": format!("unsubscribe_orderbook_{}", symbol),
            "topic": topic,
            "event": "unsubscribe"
        });
        self.unsubscribe(msg).await
    }

    // --- Stop Method ---
    pub async fn stop(&self) {
        info!("Stopping WebSocket client...");
        // Abort the manager task
        self.manager_handle.abort();
        // Optionally try sending a Close frame if connected
        let guard = self.shared_tx.lock().await;
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(Message::Close(None)).await;
        }
        info!("Stop signal sent.");
    }
}

// --- Private Client ---

pub struct WebsocketPrivateClient {
    shared_tx: SharedSender,
    subscriptions: SubscriptionState,
    manager_handle: JoinHandle<()>, // Handle to the manager task
}

impl WebsocketPrivateClient {
    pub async fn connect(
        orderly_key: String,
        orderly_secret: String,
        account_id: String,
        is_testnet: bool,
        on_message: Arc<dyn Fn(String) + Send + Sync + 'static>,
        on_close: Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Self> {
        let shared_tx: SharedSender = Arc::new(Mutex::new(None));
        let subscriptions: SubscriptionState = Arc::new(Mutex::new(HashSet::new()));

        let manager_handle = tokio::spawn({
            // Clone Arcs and credentials
            let shared_tx = Arc::clone(&shared_tx);
            let subscriptions = Arc::clone(&subscriptions);
            let on_message = Arc::clone(&on_message);
            let on_close = Arc::clone(&on_close);
            let orderly_key = orderly_key.clone();
            let orderly_secret = orderly_secret.clone();
            let account_id = account_id.clone();

            async move {
                let mut retries = 0;
                loop {
                    let base_url = if is_testnet {
                        TESTNET_WS_PRIVATE_URL
                    } else {
                        MAINNET_WS_PRIVATE_URL
                    };
                    let config = WebsocketClientConfig {
                        base_url: base_url.to_string(),
                        orderly_key: Some(orderly_key.clone()), // Needed by connect_managed if it logged them
                        orderly_secret: Some(orderly_secret.clone()),
                        orderly_account_id: account_id.clone(),
                        wss_id: None,
                    };

                    info!(
                        "[Manager-Priv] Attempting connection (Retry {})...",
                        retries
                    );
                    match connect_managed(config, Arc::clone(&on_message), Arc::clone(&on_close))
                        .await
                    {
                        Ok((handle, tx)) => {
                            info!("[Manager-Priv] Connection established. Authenticating...");
                            retries = 0;

                            // --- Authenticate ---
                            let auth_success = match Self::authenticate(
                                &tx,
                                &orderly_key,
                                &orderly_secret,
                            )
                            .await
                            {
                                Ok(_) => {
                                    info!("[Manager-Priv] Auth message sent successfully.");
                                    // Ideally wait for auth confirmation message via on_message callback
                                    // before setting shared_tx and resubscribing. This needs more complex state.
                                    // For now, assume auth will likely succeed if message sent.
                                    true
                                }
                                Err(e) => {
                                    error!("[Manager-Priv] Failed to send auth message: {}. Cannot proceed.", e);
                                    false
                                }
                            };

                            if auth_success {
                                // Store sender *after* attempting auth
                                *shared_tx.lock().await = Some(tx.clone());

                                // --- Resubscribe ---
                                let subs_guard = subscriptions.lock().await;
                                if !subs_guard.is_empty() {
                                    info!(
                                        "[Manager-Priv] Resubscribing to {} topics...",
                                        subs_guard.len()
                                    );
                                    for msg_str in subs_guard.iter() {
                                        if let Err(e) =
                                            tx.send(Message::Text(msg_str.clone())).await
                                        {
                                            error!("[Manager-Priv] Failed to send resubscription '{}': {}. Aborting.", msg_str, e);
                                            break;
                                        }
                                    }
                                }
                                drop(subs_guard);
                                // --- End Resubscribe ---
                            } else {
                                // Auth failed, don't store sender, proceed to retry logic
                                error!(
                                    "[Manager-Priv] Authentication failed. Will retry connection."
                                );
                            }

                            // Wait for connection task to end (if auth succeeded)
                            if auth_success {
                                handle.await.unwrap_or_else(|e| {
                                    error!("[Manager-Priv] Connection task panicked: {}", e);
                                });
                                info!("[Manager-Priv] Connection task ended.");
                            }
                            // If auth failed, handle is implicitly dropped, proceed to retry.
                        }
                        Err(e) => {
                            error!("[Manager-Priv] Failed to establish connection: {}", e);
                        }
                    }

                    // Connection failed, auth failed, or handle finished
                    *shared_tx.lock().await = None; // Clear sender
                    on_close();

                    retries += 1;
                    if retries >= MAX_RETRIES {
                        error!(
                            "[Manager-Priv] Max retries ({}) reached. Stopping.",
                            MAX_RETRIES
                        );
                        break;
                    }

                    warn!(
                        "[Manager-Priv] Disconnected. Retrying in {} seconds...",
                        RETRY_DELAY_SECS
                    );
                    sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                }
                info!("[Manager-Priv] Task finished.");
            }
        });

        Ok(Self {
            shared_tx,
            subscriptions,
            manager_handle,
        })
    }

    /// Helper function to send the authentication message.
    async fn authenticate(tx: &mpsc::Sender<Message>, key: &str, secret: &str) -> Result<()> {
        let timestamp = auth::get_timestamp_ms()?;
        let signature = auth::generate_signature(secret, &timestamp.to_string())?;
        let auth_msg = json!({
            "id": "auth",
            "event": "auth",
            "params": {
                "orderly_key": key,
                "sign": signature,
                "timestamp": timestamp,
            }
        });
        let auth_msg_str = serde_json::to_string(&auth_msg)?;
        tx.send(Message::Text(auth_msg_str)).await.map_err(|e| {
            OrderlyError::WebsocketError(format!("Failed to send auth message: {}", e))
        })
    }

    /// Sends a raw JSON message (helper).
    async fn send_json(&self, json_value: Value) -> Result<()> {
        let msg_str = serde_json::to_string(&json_value)?;
        self.send_str(&msg_str).await
    }

    /// Sends a raw text message (helper).
    async fn send_str(&self, text: &str) -> Result<()> {
        let guard = self.shared_tx.lock().await;
        if let Some(tx) = guard.as_ref() {
            tx.send(Message::Text(text.to_string()))
                .await
                .map_err(|e| OrderlyError::WebsocketError(format!("Failed to send message: {}", e)))
        } else {
            Err(OrderlyError::WebsocketError("Not connected".to_string()))
        }
    }

    /// Helper to add subscription and send message.
    async fn subscribe(&self, topic_msg: Value) -> Result<()> {
        let msg_str = serde_json::to_string(&topic_msg)?;
        self.subscriptions.lock().await.insert(msg_str.clone());
        self.send_str(&msg_str).await
    }

    /// Helper to remove subscription and send message.
    async fn unsubscribe(&self, topic_msg: Value) -> Result<()> {
        let msg_str = serde_json::to_string(&topic_msg)?;
        self.subscriptions.lock().await.remove(&msg_str);
        self.send_str(&msg_str).await
    }

    // --- Private Subscription Methods (Updated) ---

    pub async fn subscribe_execution_reports(&self) -> Result<()> {
        let msg = json!({
            "id": "subscribe_execution",
            "topic": "execution",
            "event": "subscribe"
        });
        self.subscribe(msg).await
    }

    pub async fn unsubscribe_execution_reports(&self) -> Result<()> {
        let msg = json!({
            "id": "unsubscribe_execution",
            "topic": "execution",
            "event": "unsubscribe"
        });
        self.unsubscribe(msg).await
    }

    pub async fn subscribe_positions(&self) -> Result<()> {
        let msg = json!({
            "id": "subscribe_position",
            "topic": "position",
            "event": "subscribe"
        });
        self.subscribe(msg).await
    }

    pub async fn unsubscribe_positions(&self) -> Result<()> {
        let msg = json!({
            "id": "unsubscribe_position",
            "topic": "position",
            "event": "unsubscribe"
        });
        self.unsubscribe(msg).await
    }

    // Added balance subscription methods
    pub async fn subscribe_balance(&self) -> Result<()> {
        let msg = json!({
            "id": "subscribe_balance",
            "topic": "balance",
            "event": "subscribe"
        });
        self.subscribe(msg).await
    }

    pub async fn unsubscribe_balance(&self) -> Result<()> {
        let msg = json!({
            "id": "unsubscribe_balance",
            "topic": "balance",
            "event": "unsubscribe"
        });
        self.unsubscribe(msg).await
    }

    // --- Stop Method ---
    pub async fn stop(&self) {
        info!("Stopping WebSocket client...");
        self.manager_handle.abort();
        let guard = self.shared_tx.lock().await;
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(Message::Close(None)).await;
        }
        info!("Stop signal sent.");
    }
}
