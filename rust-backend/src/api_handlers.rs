use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};

use crate::database::{ProjectRecord, StructuredLogRecord, TicketRecord, UserRecord, PlanEdit, PlanApproval};
use crate::jwt::{self, JwtConfig, Claims};
use crate::AppState;

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub directory_path: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub directory_path: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub title: String,
    pub description: String,
    pub status: String,
    pub code_context: Option<String>,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_required_approvals")]
    pub required_approvals: i32,
}

fn default_mode() -> String {
    "ask".to_string()
}

fn default_required_approvals() -> i32 {
    2
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct LogsQueryParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedLogsResponse {
    pub logs: Vec<StructuredLogRecord>,
    pub total: u64,
    pub has_more: bool,
}

// GET /api/projects
pub async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<ProjectRecord>>, StatusCode> {
    match state.database.list_projects().await {
        Ok(projects) => Ok(Json(projects)),
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GET /api/projects/:id
pub async fn get_project(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProjectRecord>, StatusCode> {
    match state.database.get_project(&id).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// POST /api/projects
pub async fn create_project(
    State(state): State<AppState>,
    Json(data): Json<CreateProjectRequest>,
) -> Result<Json<ProjectRecord>, StatusCode> {
    let project = ProjectRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name: data.name,
        description: data.description,
        directory_path: data.directory_path,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };

    match state.database.create_project(&project).await {
        Ok(_) => Ok(Json(project)),
        Err(e) => {
            tracing::error!("Failed to create project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// PUT /api/projects/:id
pub async fn update_project(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(data): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectRecord>, StatusCode> {
    // Get existing project first
    let existing = match state.database.get_project(&id).await {
        Ok(Some(project)) => project,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let updated = ProjectRecord {
        id: existing.id.clone(),
        name: data.name,
        description: data.description,
        directory_path: data.directory_path,
        created_at: existing.created_at,
        updated_at: Utc::now().to_rfc3339(),
    };

    match state.database.update_project(&updated).await {
        Ok(_) => Ok(Json(updated)),
        Err(e) => {
            tracing::error!("Failed to update project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// DELETE /api/projects/:id
pub async fn delete_project(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    match state.database.delete_project(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GET /api/projects/:project_id/tickets
pub async fn list_tickets(
    Path(project_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<TicketRecord>>, StatusCode> {
    match state.database.list_tickets_by_project(&project_id).await {
        Ok(tickets) => Ok(Json(tickets)),
        Err(e) => {
            tracing::error!("Failed to list tickets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// POST /api/projects/:project_id/tickets
pub async fn create_ticket(
    Path(project_id): Path<String>,
    State(state): State<AppState>,
    Json(data): Json<CreateTicketRequest>,
) -> Result<Json<TicketRecord>, StatusCode> {
    let ticket = TicketRecord {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project_id.clone(),
        title: data.title,
        description: data.description,
        status: data.status,
        code_context: data.code_context,
        analysis_result: None,
        is_analyzing: false,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
        mode: data.mode,
        plan_content: None,
        plan_created_at: None,
        required_approvals: data.required_approvals,
    };

    match state.database.create_ticket(&ticket).await {
        Ok(_) => Ok(Json(ticket)),
        Err(e) => {
            tracing::error!("Failed to create ticket: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// PUT /api/tickets/:id/status
pub async fn update_ticket_status(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(data): Json<UpdateStatusRequest>,
) -> Result<StatusCode, StatusCode> {
    match state.database.update_ticket_status(&id, &data.status).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to update ticket status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GET /api/tickets/:id/logs
pub async fn get_ticket_logs(
    Path(id): Path<String>,
    Query(params): Query<LogsQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedLogsResponse>, StatusCode> {
    // Validate and log pagination parameters
    let limit = params.limit;
    let offset = params.offset;
    
    tracing::debug!(
        "API get_ticket_logs: ticket_id={}, limit={:?}, offset={:?}",
        id,
        limit,
        offset
    );

    // Validate limit if provided
    if let Some(lim) = limit {
        if lim == 0 {
            tracing::warn!("Invalid limit=0 requested, will use default");
        } else if lim > 1000 {
            tracing::warn!("Limit {} exceeds maximum 1000, will be capped", lim);
        }
    }

    // Get total count
    let total = match state.database.count_logs_for_ticket(&id).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count ticket logs: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get paginated logs
    let logs = match state.database.get_logs_for_ticket(&id, limit, offset).await {
        Ok(logs) => logs,
        Err(e) => {
            tracing::error!("Failed to get ticket logs: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!(
        "API get_ticket_logs: returned {} logs out of {} total",
        logs.len(),
        total
    );

    // Calculate has_more
    let offset_val = offset.unwrap_or(0);
    let has_more = (offset_val + logs.len() as u64) < total;

    Ok(Json(PaginatedLogsResponse {
        logs,
        total,
        has_more,
    }))
}

// POST /api/tickets/:id/stop-analysis
pub async fn stop_analysis(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("⛔ Stop analysis requested for ticket: {}", id);

    // Check if ticket exists
    let ticket = match state.database.get_ticket(&id).await {
        Ok(Some(ticket)) => ticket,
        Ok(None) => {
            error!("Ticket {} not found", id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Failed to get ticket {}: {}", id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if ticket is currently analyzing
    if !ticket.is_analyzing {
        warn!("Ticket {} is not currently being analyzed", id);
        return Ok(Json(json!({
            "success": false,
            "message": "Ticket is not being analyzed"
        })));
    }

    // Lookup and abort the running task
    let handle = {
        let mut tasks = state.running_tasks.lock().await;
        tasks.remove(&id)
    };

    if let Some(handle) = handle {
        handle.abort();
        info!("⛔ Aborted analysis task for ticket {}", id);
    } else {
        warn!("No running task found for ticket {} (may have already completed)", id);
    }

    // Update database: set is_analyzing = false
    if let Err(e) = state.database.update_ticket_analyzing(&id, false).await {
        error!("Failed to update ticket {} analyzing status: {}", id, e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Find active session and cancel it
    if let Ok(Some(session)) = state.database.get_active_session_by_ticket(&id).await {
        if let Err(e) = state.database.cancel_session(&session.id, "Cancelled by user").await {
            error!("Failed to cancel session {}: {}", session.id, e);
        }
    }

    // Create and broadcast stop log
    let log_entry = crate::log_normalizer::LogNormalizer::new().normalize(
        "⛔ Đã dừng phân tích theo yêu cầu".to_string(),
        id.clone(),
    );
    state.msg_store.push(log_entry).await;

    // Broadcast stop event to all connected clients
    let _ = state.broadcast_tx.send(crate::BroadcastMessage {
        ticket_id: id.clone(),
        message_type: "analysis-stopped".to_string(),
        content: "Analysis stopped by user".to_string(),
        timestamp: chrono::Utc::now(),
    });

    info!("✅ Successfully stopped analysis for ticket {}", id);
    Ok(Json(json!({
        "success": true,
        "message": "Analysis stopped successfully"
    })))
}

// Authentication endpoints
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    info!("📝 Registration attempt for username: {}", payload.username);

    // Check if username already exists
    match state.database.get_user_by_username(&payload.username).await {
        Ok(Some(_)) => {
            warn!("⚠️ Username already exists: {}", payload.username);
            return Err((
                StatusCode::CONFLICT,
                Json(json!({ "error": "Username already exists" })),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            error!("❌ Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ));
        }
    }

    // Hash password
    let password_hash = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("❌ Password hashing error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ));
        }
    };

    // Create user record
    let user_id = uuid::Uuid::new_v4().to_string();
    let user = UserRecord {
        id: user_id.clone(),
        username: payload.username.clone(),
        password_hash,
        created_at: Utc::now().to_rfc3339(),
    };

    // Save to database
    if let Err(e) = state.database.create_user(&user).await {
        error!("❌ Failed to create user: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to create user" })),
        ));
    }

    // Generate JWT token
    let jwt_config = JwtConfig::default();
    let token = match jwt::generate_token(&user_id, &payload.username, &jwt_config) {
        Ok(token) => token,
        Err(e) => {
            error!("❌ Failed to generate token: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate token" })),
            ));
        }
    };

    info!("✅ User registered successfully: {}", payload.username);

    Ok(Json(AuthResponse {
        token,
        user: UserInfo {
            id: user_id,
            username: payload.username,
        },
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    info!("🔐 Login attempt for username: {}", payload.username);

    // Get user from database
    let user = match state.database.get_user_by_username(&payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("⚠️ User not found: {}", payload.username);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid credentials" })),
            ));
        }
        Err(e) => {
            error!("❌ Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ));
        }
    };

    // Verify password
    let password_valid = match bcrypt::verify(&payload.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            error!("❌ Password verification error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ));
        }
    };

    if !password_valid {
        warn!("⚠️ Invalid password for user: {}", payload.username);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid credentials" })),
        ));
    }

    // Generate JWT token
    let jwt_config = JwtConfig::default();
    let token = match jwt::generate_token(&user.id, &user.username, &jwt_config) {
        Ok(token) => token,
        Err(e) => {
            error!("❌ Failed to generate token: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate token" })),
            ));
        }
    };

    info!("✅ User logged in successfully: {}", payload.username);

    Ok(Json(AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            username: user.username,
        },
    }))
}

pub async fn get_me(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<UserInfo>, (StatusCode, Json<Value>)> {
    // Get user from database to ensure they still exist
    match state.database.get_user_by_id(&claims.sub).await {
        Ok(Some(user)) => Ok(Json(UserInfo {
            id: user.id,
            username: user.username,
        })),
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "User not found" })),
        )),
        Err(e) => {
            error!("❌ Database error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

// Plan collaboration endpoints
#[derive(Debug, Deserialize)]
pub struct UpdatePlanRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovePlanRequest {
    pub status: String, // "approved" or "rejected"
}

pub async fn update_plan(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePlanRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    info!("📝 User {} updating plan for ticket {}", claims.username, id);

    match state.database.update_plan_content(&id, &claims.sub, &payload.content).await {
        Ok(_) => {
            // Broadcast plan update
            let _ = state.broadcast_tx.send(crate::BroadcastMessage {
                ticket_id: id.clone(),
                message_type: "plan-updated".to_string(),
                content: payload.content,
                timestamp: Utc::now(),
            });

            Ok(Json(json!({ "success": true })))
        }
        Err(e) => {
            error!("❌ Failed to update plan: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update plan" })),
            ))
        }
    }
}

pub async fn get_plan_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<PlanEdit>>, (StatusCode, Json<Value>)> {
    match state.database.get_plan_edits(&id).await {
        Ok(edits) => Ok(Json(edits)),
        Err(e) => {
            error!("❌ Failed to get plan history: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get plan history" })),
            ))
        }
    }
}

pub async fn approve_plan(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ApprovePlanRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    info!("👍 User {} {} plan for ticket {}", claims.username, payload.status, id);

    // Save approval
    if let Err(e) = state.database.approve_plan(&id, &claims.sub, &payload.status).await {
        error!("❌ Failed to approve plan: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to approve plan" })),
        ));
    }

    // Check if we have enough approvals to auto-trigger
    if payload.status == "approved" {
        match state.database.get_ticket(&id).await {
            Ok(Some(ticket)) => {
                let approval_count = state.database.count_plan_approvals(&id).await.unwrap_or(0);
                
                if approval_count >= ticket.required_approvals as i64 {
                    info!("🚀 Auto-triggering implementation for ticket {}", id);
                    
                    // Broadcast auto-implement event
                    let _ = state.broadcast_tx.send(crate::BroadcastMessage {
                        ticket_id: id.clone(),
                        message_type: "auto-implement-started".to_string(),
                        content: format!("Plan approved by {} users, starting implementation", approval_count),
                        timestamp: Utc::now(),
                    });
                }
            }
            _ => {}
        }
    }

    // Broadcast approval
    let _ = state.broadcast_tx.send(crate::BroadcastMessage {
        ticket_id: id.clone(),
        message_type: "plan-approved".to_string(),
        content: serde_json::to_string(&payload).unwrap_or_default(),
        timestamp: Utc::now(),
    });

    Ok(Json(json!({ "success": true })))
}

pub async fn get_plan_approvals(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<PlanApproval>>, (StatusCode, Json<Value>)> {
    match state.database.get_plan_approvals(&id).await {
        Ok(approvals) => Ok(Json(approvals)),
        Err(e) => {
            error!("❌ Failed to get plan approvals: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to get plan approvals" })),
            ))
        }
    }
}
