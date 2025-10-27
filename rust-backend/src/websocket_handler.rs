use crate::{AppState, CodeAnalysisRequest};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut log_receiver = state.msg_store.subscribe();
    let client_id = Uuid::new_v4().to_string();
    let client_id_clone = client_id.clone();

    info!("🔌 Client mới kết nối: {}", client_id);

    // Spawn task to listen for broadcast messages and forward to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(log_entry) = log_receiver.recv().await {
            // Convert StructuredLogEntry to JSON and send to client
            let message = json!({
                "message_type": "structured-log",
                "log": {
                    "id": log_entry.id,
                    "ticket_id": log_entry.ticket_id,
                    "message_type": log_entry.message_type,
                    "content": log_entry.content,
                    "raw_log": log_entry.raw_log,
                    "metadata": log_entry.metadata,
                    "timestamp": log_entry.timestamp.to_rfc3339(),
                }
            });

            let json_msg = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string());

            if sender.send(Message::Text(json_msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = handle_client_message(&text, &state, &client_id_clone).await {
                        error!("Lỗi xử lý message từ client {}: {}", client_id_clone, e);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} đã đóng kết nối", client_id_clone);
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    // Pings are handled automatically by axum
                }
                Ok(Message::Pong(_)) => {
                    // Ignore pong messages
                }
                Ok(Message::Binary(_)) => {
                    // Ignore binary messages
                }
                Err(e) => {
                    error!("Lỗi WebSocket với client {}: {}", client_id_clone, e);
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("Client {} đã ngắt kết nối", client_id);
}

async fn handle_client_message(
    text: &str,
    state: &AppState,
    client_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message: Value = serde_json::from_str(text)?;
    let message_type = message["type"].as_str().unwrap_or("unknown");

    info!("📨 Nhận message từ client {}: {}", client_id, message_type);

    match message_type {
        "start-code-analysis" => {
            let request = CodeAnalysisRequest {
                ticket_id: message["ticketId"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                code_context: message["codeContext"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                question: message["question"].as_str().unwrap_or("").to_string(),
            };

            info!(
                "🚀 Bắt đầu phân tích code cho ticket {} từ client {}",
                request.ticket_id, client_id
            );

            // Spawn analysis in background
            let cursor_agent = state.cursor_agent.clone();
            let msg_store = state.msg_store.clone();
            let database = state.database.clone();
            let broadcast_tx = state.broadcast_tx.clone();

            tokio::spawn(async move {
                match cursor_agent
                    .analyze_code(request.clone(), msg_store.clone(), database.clone())
                    .await
                {
                    Ok(response) => {
                        // Broadcast completion message
                        let _ = broadcast_tx.send(crate::BroadcastMessage {
                            ticket_id: response.ticket_id,
                            message_type: "code-analysis-complete".to_string(),
                            content: response.result,
                            timestamp: chrono::Utc::now(),
                        });

                        info!("✅ Phân tích hoàn tất cho ticket {}", request.ticket_id);
                    }
                    Err(e) => {
                        error!("❌ Lỗi phân tích code: {}", e);

                        // Broadcast error message
                        let _ = broadcast_tx.send(crate::BroadcastMessage {
                            ticket_id: request.ticket_id,
                            message_type: "code-analysis-error".to_string(),
                            content: e.to_string(),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            });
        }

        "get-ticket-logs" => {
            let ticket_id = message["ticketId"].as_str().unwrap_or("");

            info!("📋 Client {} yêu cầu logs cho ticket {}", client_id, ticket_id);

            // This is handled by returning from database
            // Not implemented in this handler but available via msg_store.get_logs()
        }

        "load-tickets" => {
            info!("📂 Client {} yêu cầu tải danh sách tickets", client_id);

            // Load tickets from database
            match state.database.list_tickets().await {
                Ok(tickets) => {
                    info!("✅ Tải được {} tickets từ database", tickets.len());
                    // Tickets will be sent back via broadcast or direct response
                    // For now, just log success
                }
                Err(e) => {
                    error!("❌ Lỗi tải tickets: {}", e);
                }
            }
        }

        "create-ticket" => {
            info!("➕ Client {} tạo ticket mới", client_id);

            let ticket_id = message["id"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            let ticket = crate::database::TicketRecord {
                id: ticket_id,
                title: message["title"].as_str().unwrap_or("").to_string(),
                description: message["description"].as_str().unwrap_or("").to_string(),
                status: message["status"].as_str().unwrap_or("todo").to_string(),
                code_context: message["codeContext"].as_str().map(|s| s.to_string()),
                analysis_result: None,
                is_analyzing: false,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            if let Err(e) = state.database.create_ticket(&ticket).await {
                error!("❌ Lỗi tạo ticket: {}", e);
            } else {
                info!("✅ Tạo ticket thành công: {}", ticket.id);
            }
        }

        "update-ticket-status" => {
            let ticket_id = message["ticketId"].as_str().unwrap_or("");
            let new_status = message["status"].as_str().unwrap_or("");

            info!(
                "🔄 Client {} cập nhật status ticket {} -> {}",
                client_id, ticket_id, new_status
            );

            if let Err(e) = state.database.update_ticket_status(ticket_id, new_status).await {
                error!("❌ Lỗi cập nhật ticket status: {}", e);
            }
        }

        "ping" => {
            info!("🏓 Ping từ client {}", client_id);
            // Pong will be sent automatically
        }

        _ => {
            info!("❓ Unknown message type từ client {}: {}", client_id, message_type);
        }
    }

    Ok(())
}
