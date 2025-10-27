use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

mod cursor_agent;
mod database;
mod log_normalizer;
mod message_store;
mod websocket_handler;

use cursor_agent::CursorAgent;
use database::Database;
use message_store::MsgStore;

#[derive(Debug, Clone)]
pub struct AppState {
    pub cursor_agent: Arc<CursorAgent>,
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
    pub database: Arc<Database>,
    pub msg_store: Arc<MsgStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub ticket_id: String,
    pub message_type: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisRequest {
    pub ticket_id: String,
    pub code_context: String,
    pub question: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysisResponse {
    pub ticket_id: String,
    pub result: String,
    pub logs: Vec<String>,
    pub success: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Khởi động QA Chatbot Backend...");

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:qa_chatbot.db".to_string());

    info!("📊 Kết nối database: {}", database_url);

    let database = Arc::new(
        Database::new(&database_url)
            .await
            .expect("Failed to connect to database"),
    );

    // Initialize database schema
    database
        .init_schema()
        .await
        .expect("Failed to initialize database schema");

    info!("✅ Database schema initialized");

    // Initialize message store
    let msg_store = Arc::new(MsgStore::new(database.clone()));

    info!("✅ Message store initialized");

    // Initialize broadcast channel for legacy messages
    let (broadcast_tx, _broadcast_rx) = broadcast::channel(1000);

    // Initialize Cursor Agent
    let cursor_agent = Arc::new(CursorAgent::new());

    info!("✅ Cursor Agent initialized");

    // Create app state
    let app_state = AppState {
        cursor_agent,
        broadcast_tx,
        database,
        msg_store,
    };

    info!("✅ App state initialized");

    // Build router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    info!("🌐 Server đang chạy trên {}", addr);
    info!("📡 WebSocket endpoint: ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("✅ Server khởi động thành công!");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn health_check() -> &'static str {
    "✅ QA Chatbot Backend đang hoạt động!"
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| websocket_handler::handle_websocket(socket, state))
}
