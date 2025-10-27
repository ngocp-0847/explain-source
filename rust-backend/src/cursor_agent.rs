use crate::{BroadcastMessage, CodeAnalysisRequest, CodeAnalysisResponse};
use anyhow::Result;
use serde_json::json;
use std::process::Stdio;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::broadcast,
};
use tracing::{error, info};

#[derive(Debug)]
pub struct CursorAgent {
    broadcast_tx: Option<broadcast::Sender<BroadcastMessage>>,
}

impl CursorAgent {
    pub fn new() -> Self {
        Self {
            broadcast_tx: None,
        }
    }

    pub fn set_broadcast_tx(&mut self, tx: broadcast::Sender<BroadcastMessage>) {
        self.broadcast_tx = Some(tx);
    }

    pub async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
    ) -> Result<CodeAnalysisResponse> {
        info!("Bắt đầu phân tích code cho ticket: {}", request.ticket_id);

        let mut logs = Vec::new();
        let mut result = String::new();

        // Tạo prompt cho Cursor Agent
        let prompt = self.create_analysis_prompt(&request);
        
        // Gọi Cursor Agent thông qua command line
        match self.execute_cursor_agent(&request, &prompt).await {
            Ok(output) => {
                result = output;
                logs.push("Cursor Agent đã hoàn thành phân tích".to_string());
                
                // Gửi kết quả qua WebSocket
                if let Some(ref tx) = self.broadcast_tx {
                    let _ = tx.send(BroadcastMessage {
                        ticket_id: request.ticket_id.clone(),
                        message_type: "code-analysis".to_string(),
                        content: result.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
            Err(e) => {
                error!("Lỗi khi thực thi Cursor Agent: {}", e);
                logs.push(format!("Lỗi: {}", e));
                result = "Không thể phân tích code do lỗi hệ thống".to_string();
            }
        }

        Ok(CodeAnalysisResponse {
            ticket_id: request.ticket_id,
            result,
            logs,
            success: true,
        })
    }

    fn create_analysis_prompt(&self, request: &CodeAnalysisRequest) -> String {
        format!(
            r#"
Bạn là một AI assistant chuyên phân tích source code để giúp QA hiểu business flow.

CONTEXT:
- File/Module: {}
- Câu hỏi của QA: {}

YÊU CẦU:
1. Phân tích code trong context trên
2. Giải thích business flow một cách dễ hiểu
3. Chỉ ra các điểm quan trọng QA cần chú ý
4. Đưa ra ví dụ cụ thể nếu có thể
5. Trả lời bằng tiếng Việt

Hãy trả lời chi tiết và hữu ích cho QA.
"#,
            request.code_context, request.question
        )
    }

    async fn execute_cursor_agent(
        &self,
        request: &CodeAnalysisRequest,
        prompt: &str,
    ) -> Result<String> {
        // Trong thực tế, đây sẽ là lệnh gọi Cursor Agent
        // Tạm thời mô phỏng bằng cách tạo response giả
        let simulated_response = self.simulate_cursor_agent_response(request, prompt).await;
        Ok(simulated_response)
    }

    async fn simulate_cursor_agent_response(
        &self,
        request: &CodeAnalysisRequest,
        _prompt: &str,
    ) -> String {
        // Mô phỏng delay của Cursor Agent
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Tạo response mô phỏng dựa trên context
        match request.code_context.as_str() {
            "auth/login.js" => {
                format!(
                    r#"
## Phân tích Business Flow: Đăng nhập User

### 1. Flow chính:
1. **Input Validation**: Kiểm tra email/password format
2. **Authentication**: Gọi API xác thực với backend
3. **Session Management**: Tạo session và lưu token
4. **Redirect**: Chuyển hướng đến dashboard

### 2. Các điểm quan trọng cho QA:
- **Validation Rules**: Email phải đúng format, password tối thiểu 8 ký tự
- **Error Handling**: Xử lý lỗi network, invalid credentials
- **Security**: Token được lưu trong httpOnly cookie
- **Loading States**: Hiển thị spinner trong quá trình đăng nhập

### 3. Test Cases cần kiểm tra:
- Đăng nhập thành công với credentials hợp lệ
- Đăng nhập thất bại với credentials sai
- Xử lý lỗi network timeout
- Validation input không hợp lệ

### 4. Code Flow:
```
LoginForm → validateInput() → callAuthAPI() → saveSession() → redirect()
```

Ticket ID: {}
"#,
                    request.ticket_id
                )
            }
            "api/payment.js" => {
                format!(
                    r#"
## Phân tích Business Flow: Thanh toán

### 1. Flow chính:
1. **Payment Initiation**: Khởi tạo giao dịch thanh toán
2. **Payment Gateway**: Kết nối với cổng thanh toán
3. **Transaction Processing**: Xử lý giao dịch
4. **Confirmation**: Xác nhận kết quả thanh toán

### 2. Các điểm quan trọng cho QA:
- **Payment Methods**: Hỗ trợ credit card, bank transfer, e-wallet
- **Security**: Mã hóa thông tin thẻ, PCI compliance
- **Error Handling**: Xử lý lỗi thanh toán, timeout
- **Audit Trail**: Log tất cả giao dịch

### 3. Test Cases cần kiểm tra:
- Thanh toán thành công
- Thanh toán thất bại (insufficient funds)
- Timeout payment gateway
- Invalid payment method

### 4. Code Flow:
```
PaymentForm → validatePayment() → callGateway() → processTransaction() → confirmPayment()
```

Ticket ID: {}
"#,
                    request.ticket_id
                )
            }
            _ => {
                format!(
                    r#"
## Phân tích Business Flow

### Context: {}
### Câu hỏi: {}

### Phân tích:
Tôi đã phân tích code trong context `{}` và hiểu được câu hỏi của bạn về "{}".

**Business Flow chính:**
1. Input validation và preprocessing
2. Core business logic processing  
3. Data persistence và state management
4. Response formatting và error handling

**Các điểm quan trọng:**
- Kiểm tra input validation rules
- Xử lý error cases và edge cases
- Performance considerations
- Security implications

**Test Cases đề xuất:**
- Happy path scenarios
- Error handling scenarios
- Edge cases và boundary conditions
- Performance và load testing

Ticket ID: {}
"#,
                    request.code_context,
                    request.question,
                    request.code_context,
                    request.question,
                    request.ticket_id
                )
            }
        }
    }
}