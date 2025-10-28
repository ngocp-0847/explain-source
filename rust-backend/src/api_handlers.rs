use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::Deserialize;

use crate::database::{ProjectRecord, StructuredLogRecord, TicketRecord};
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
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
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
    State(state): State<AppState>,
) -> Result<Json<Vec<StructuredLogRecord>>, StatusCode> {
    match state.database.get_logs_for_ticket(&id).await {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => {
            tracing::error!("Failed to get ticket logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

