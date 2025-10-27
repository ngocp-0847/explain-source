use axum::{
    extract::{
        ws::WebSocketUpgrade,
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

mod cursor_agent;
mod websocket_handler;

use cursor_agent::CursorAgent;

#[derive(Debug, Clone)]
pub struct AppState {
    pub cursor_agent: Arc<CursorAgent>,
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub ticket_id: String,
    pub message_type: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    tracing_subscriber::fmt::init();

    let (broadcast_tx, _broadcast_rx) = broadcast::channel(1000);
    let cursor_agent = Arc::new(CursorAgent::new());
    
    let app_state = AppState {
        cursor_agent,
        broadcast_tx,
    };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server đang chạy trên {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "QA Chatbot Backend đang hoạt động!"
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_handler::handle_websocket(socket, state))
}