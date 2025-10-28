use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

mod api_handlers;
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
    pub project_id: String,
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

    // Clear old data and seed sample projects
    info!("🗑️ Clearing old tickets data...");
    database.clear_all_tickets().await.expect("Failed to clear tickets");
    info!("✅ Old data cleared");

    // Seed sample projects
    info!("🌱 Seeding sample projects...");
    let sample_projects = vec![
        crate::database::ProjectRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name: "E-commerce Platform".to_string(),
            description: Some("Phân tích flow thanh toán và quản lý đơn hàng".to_string()),
            directory_path: "/home/phan.ngoc@sun-asterisk.com/Documents/projects/explain-source/rust-backend".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
        crate::database::ProjectRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Blog CMS".to_string(),
            description: Some("Hệ thống quản lý nội dung và authentication".to_string()),
            directory_path: "/home/phan.ngoc@sun-asterisk.com/Documents/projects/explain-source".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    for project in &sample_projects {
        database.create_project(project).await.expect("Failed to create project");
    }
    info!("✅ Seeded {} sample projects", sample_projects.len());

    // Initialize message store
    let msg_store = Arc::new(MsgStore::new(database.clone()));

    info!("✅ Message store initialized");

    // Initialize broadcast channel for legacy messages
    let (broadcast_tx, _broadcast_rx) = broadcast::channel(1000);

    // Initialize Cursor Agent with config from environment
    let cursor_config = cursor_agent::CursorAgentConfig::from_env();
    info!("🔧 Cursor Agent config:");
    info!("  - Executable: {}", cursor_config.executable_path);
    info!("  - Timeout: {}s", cursor_config.timeout_seconds);
    info!("  - Retries: {}", cursor_config.max_retries);
    info!("  - Output format: {:?}", cursor_config.output_format);
    if cursor_config.api_key.is_some() {
        info!("  - API key: [SET]");
    }
    
    let cursor_agent = Arc::new(CursorAgent::with_config(cursor_config));

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
        .route("/api/projects", get(api_handlers::list_projects).post(api_handlers::create_project))
        .route("/api/projects/:id", get(api_handlers::get_project).put(api_handlers::update_project).delete(api_handlers::delete_project))
        .route("/api/projects/:project_id/tickets", get(api_handlers::list_tickets).post(api_handlers::create_ticket))
        .route("/api/tickets/:id/status", put(api_handlers::update_ticket_status))
        .route("/api/tickets/:id/logs", get(api_handlers::get_ticket_logs))
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
