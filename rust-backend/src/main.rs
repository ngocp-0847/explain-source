use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::{get, put, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{sync::{broadcast, Mutex}, task::AbortHandle};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod agent_factory;
mod api_handlers;
mod code_agent;
mod cursor_agent;
mod database;
mod gemini_agent;
mod log_normalizer;
mod message_store;
mod websocket_handler;

use code_agent::CodeAgent;
use database::Database;
use message_store::MsgStore;

#[derive(Clone)]
pub struct AppState {
    pub code_agent: Arc<dyn CodeAgent>,
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
    pub database: Arc<Database>,
    pub msg_store: Arc<MsgStore>,
    pub running_tasks: Arc<Mutex<HashMap<String, AbortHandle>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub ticket_id: String,
    pub message_type: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Re-export for backward compatibility
pub use code_agent::{CodeAnalysisRequest, CodeAnalysisResponse};

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    // Try to load from rust-backend/.env first (relative to current directory)
    // Then fallback to .env in current directory
    let env_paths = [
        "rust-backend/.env",
        ".env",
    ];
    
    let mut env_loaded = false;
    for path in &env_paths {
        match dotenv::from_path(path) {
            Ok(_) => {
                // File loaded successfully
                info!("📄 Loaded .env file from: {}", path);
                env_loaded = true;
                break;
            }
            Err(dotenv::Error::Io(_)) => {
                // File doesn't exist, try next path
                continue;
            }
            Err(e) => {
                // Other error (e.g., parsing error)
                warn!("⚠️ Error loading .env from {}: {}, trying next path", path, e);
                continue;
            }
        }
    }
    
    if !env_loaded {
        // Try default dotenv behavior (searches parent directories)
        match dotenv::dotenv() {
            Ok(_) => {
                info!("📄 Loaded .env file from default location");
            }
            Err(_) => {
                info!("⚠️ No .env file found, using environment variables only");
            }
        }
    }

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

    // Run database migrations
    database
        .run_migrations()
        .await
        .expect("Failed to run database migrations");

    info!("✅ Database schema initialized and migrations applied");

    info!("📊 Database persistence enabled - keeping existing data");

    // Initialize message store
    let msg_store = Arc::new(MsgStore::new(database.clone()));

    info!("✅ Message store initialized");

    // Initialize broadcast channel for legacy messages
    let (broadcast_tx, _broadcast_rx) = broadcast::channel(1000);

    // Initialize code analysis agent from environment
    let code_agent = agent_factory::create_agent_from_env();

    info!("✅ Code analysis agent initialized");

    // Create app state
    let app_state = AppState {
        code_agent,
        broadcast_tx,
        database,
        msg_store,
        running_tasks: Arc::new(Mutex::new(HashMap::new())),
    };

    info!("✅ App state initialized");

    // Build router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/ws", get(websocket_handler))
        .route("/api/projects", get(api_handlers::list_projects).post(api_handlers::create_project))
        .route("/api/projects/:id", get(api_handlers::get_project).put(api_handlers::update_project).delete(api_handlers::delete_project))
        .route("/api/projects/:project_id/tickets", get(api_handlers::list_tickets).post(api_handlers::create_ticket))
        .route("/api/tickets/:id/stop-analysis", post(api_handlers::stop_analysis))
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
