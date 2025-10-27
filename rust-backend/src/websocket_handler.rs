use crate::{AppState, BroadcastMessage, CodeAnalysisRequest};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json;
use tracing::{error, info};
use uuid::Uuid;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    let client_id = Uuid::new_v4().to_string();

    info!("Client mới kết nối: {}", client_id);

    // Xử lý messages từ client và gửi responses
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_client_message(&text, &state, &client_id, &mut sender).await {
                    error!("Lỗi xử lý message từ client {}: {}", client_id, e);
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} đã đóng kết nối", client_id);
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Ignore pong messages
            }
            Ok(Message::Binary(_)) => {
                // Ignore binary messages
            }
            Err(e) => {
                error!("Lỗi WebSocket với client {}: {}", client_id, e);
                break;
            }
        }

        // Kiểm tra broadcast messages
        if let Ok(msg) = broadcast_rx.try_recv() {
            let json_msg = serde_json::to_string(&msg).unwrap_or_else(|_| "{}".to_string());
            if sender.send(Message::Text(json_msg)).await.is_err() {
                break;
            }
        }
    }

    info!("Client {} đã ngắt kết nối", client_id);
}

async fn handle_client_message(
    text: &str,
    state: &AppState,
    client_id: &str,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse message từ client
    let message: serde_json::Value = serde_json::from_str(text)?;
    let message_type = message["type"].as_str().unwrap_or("unknown");

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
                question: message["question"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            };

            info!(
                "Bắt đầu phân tích code cho ticket {} từ client {}",
                request.ticket_id, client_id
            );

            // Gửi log bắt đầu phân tích
            let start_log = BroadcastMessage {
                ticket_id: request.ticket_id.clone(),
                message_type: "cursor-agent-log".to_string(),
                content: "🔄 Bắt đầu phân tích code với Cursor Agent...".to_string(),
                timestamp: chrono::Utc::now(),
            };

            let _ = state.broadcast_tx.send(start_log);

            // Thực hiện phân tích code
            let result = state.cursor_agent.analyze_code(request).await?;

            // Gửi kết quả phân tích
            let analysis_result = BroadcastMessage {
                ticket_id: result.ticket_id,
                message_type: "code-analysis-complete".to_string(),
                content: result.result,
                timestamp: chrono::Utc::now(),
            };

            let _ = state.broadcast_tx.send(analysis_result);
        }
        "ping" => {
            // Respond với pong
            let pong = serde_json::json!({
                "type": "pong",
                "timestamp": chrono::Utc::now()
            });
            let json_msg = serde_json::to_string(&pong).unwrap_or_else(|_| "{}".to_string());
            let _ = sender.send(Message::Text(json_msg)).await;
        }
        _ => {
            info!("Unknown message type từ client {}: {}", client_id, message_type);
        }
    }

    Ok(())
}