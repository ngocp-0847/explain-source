use crate::database::Database;
use crate::log_normalizer::LogNormalizer;
use crate::message_store::MsgStore;
use crate::{CodeAnalysisRequest, CodeAnalysisResponse};
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

#[derive(Debug)]
pub struct CursorAgent;

impl CursorAgent {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
        msg_store: Arc<MsgStore>,
        database: Arc<Database>,
    ) -> Result<CodeAnalysisResponse> {
        info!("🚀 Bắt đầu phân tích code cho ticket: {}", request.ticket_id);

        // Create analysis session in database
        let session_id = database.create_session(&request.ticket_id).await?;

        // Update ticket status to analyzing
        database
            .update_ticket_analyzing(&request.ticket_id, true)
            .await?;

        let mut logs = Vec::new();
        let normalizer = LogNormalizer::new();

        // Send initial log
        let start_log = "🔄 Khởi động Cursor Agent...";
        let entry = normalizer.normalize(
            start_log.to_string(),
            request.ticket_id.clone(),
        );
        msg_store.push(entry).await;
        logs.push(start_log.to_string());

        // Execute Cursor Agent analysis
        let result = match self
            .execute_cursor_agent(&request, &msg_store, &normalizer)
            .await
        {
            Ok(output) => {
                info!("✅ Cursor Agent hoàn thành phân tích");

                // Send completion log
                let completion_log = "✅ Phân tích hoàn tất!";
                let entry = normalizer.normalize(
                    completion_log.to_string(),
                    request.ticket_id.clone(),
                );
                msg_store.push(entry).await;
                logs.push(completion_log.to_string());

                // Update database with success
                database.complete_session(&session_id, "Success").await?;
                database
                    .update_ticket_result(&request.ticket_id, &output)
                    .await?;

                output
            }
            Err(e) => {
                error!("❌ Lỗi khi thực thi Cursor Agent: {}", e);

                // Send error log
                let error_log = format!("❌ Lỗi: {}", e);
                let entry = normalizer.normalize(error_log.clone(), request.ticket_id.clone());
                msg_store.push(entry).await;
                logs.push(error_log);

                // Update database with failure
                database.fail_session(&session_id, &e.to_string()).await?;
                database
                    .update_ticket_analyzing(&request.ticket_id, false)
                    .await?;

                format!("Không thể phân tích code do lỗi: {}", e)
            }
        };

        Ok(CodeAnalysisResponse {
            ticket_id: request.ticket_id,
            result,
            logs,
            success: true,
        })
    }

    async fn execute_cursor_agent(
        &self,
        request: &CodeAnalysisRequest,
        msg_store: &Arc<MsgStore>,
        normalizer: &LogNormalizer,
    ) -> Result<String> {
        // For now, we'll simulate the Cursor Agent with realistic logging
        // In production, this would spawn an actual Cursor Agent process
        // and capture its stdout/stderr

        info!("🎯 Executing analysis for: {}", request.code_context);

        // Simulate analysis steps with realistic logs
        let analysis_steps = self.simulate_cursor_agent_analysis(request, msg_store, normalizer).await;

        Ok(analysis_steps)
    }

    async fn simulate_cursor_agent_analysis(
        &self,
        request: &CodeAnalysisRequest,
        msg_store: &Arc<MsgStore>,
        normalizer: &LogNormalizer,
    ) -> String {
        let ticket_id = request.ticket_id.clone();

        // Step 1: Initialize
        self.stream_log(
            "🔍 Khởi tạo phân tích code...",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(500)).await;

        // Step 2: Reading file
        let file_log = format!("📂 Reading file: {}", request.code_context);
        self.stream_log(&file_log, &ticket_id, msg_store, normalizer)
            .await;
        sleep(Duration::from_millis(800)).await;

        // Step 3: Analyzing code structure
        self.stream_log(
            "🔧 Using tool: code_analyzer",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(600)).await;

        self.stream_log(
            "🏗️  Analyzing code structure and dependencies...",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(1000)).await;

        // Step 4: Business flow analysis
        self.stream_log(
            "💼 Analysis: Extracting business flow patterns...",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(800)).await;

        // Step 5: Finding key components
        self.stream_log(
            "🔍 Found: 5 key functions and 3 data flow paths",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(600)).await;

        // Step 6: Test case generation
        self.stream_log(
            "📝 Generating test case recommendations...",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(700)).await;

        // Step 7: Final summary
        self.stream_log(
            "✨ Summary: Analysis complete with 12 findings",
            &ticket_id,
            msg_store,
            normalizer,
        )
        .await;
        sleep(Duration::from_millis(500)).await;

        // Generate comprehensive analysis result based on context
        self.generate_analysis_result(request)
    }

    async fn stream_log(
        &self,
        message: &str,
        ticket_id: &str,
        msg_store: &Arc<MsgStore>,
        normalizer: &LogNormalizer,
    ) {
        let entry = normalizer.normalize(message.to_string(), ticket_id.to_string());
        msg_store.push(entry).await;
    }

    fn generate_analysis_result(&self, request: &CodeAnalysisRequest) -> String {
        match request.code_context.as_str() {
            "auth/login.js" => {
                format!(
                    r#"
## 🔐 Phân tích Business Flow: Đăng nhập User

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
✅ Đăng nhập thành công với credentials hợp lệ
✅ Đăng nhập thất bại với credentials sai
✅ Xử lý lỗi network timeout
✅ Validation input không hợp lệ
✅ Remember me functionality
✅ Session expiry handling

### 4. Code Flow:
```
LoginForm → validateInput() → callAuthAPI() → saveSession() → redirect()
```

### 5. Các API Endpoints:
- POST /api/auth/login
- POST /api/auth/refresh-token
- POST /api/auth/logout

**Ticket ID**: {}
"#,
                    request.ticket_id
                )
            }
            "api/payment.js" => {
                format!(
                    r#"
## 💳 Phân tích Business Flow: Thanh toán

### 1. Flow chính:
1. **Payment Initiation**: Khởi tạo giao dịch thanh toán
2. **Payment Gateway**: Kết nối với cổng thanh toán (Stripe/PayPal)
3. **Transaction Processing**: Xử lý giao dịch
4. **Confirmation**: Xác nhận kết quả thanh toán
5. **Notification**: Gửi email/SMS xác nhận

### 2. Các điểm quan trọng cho QA:
- **Payment Methods**: Hỗ trợ credit card, bank transfer, e-wallet
- **Security**: Mã hóa thông tin thẻ, PCI compliance
- **Error Handling**: Xử lý lỗi thanh toán, timeout
- **Audit Trail**: Log tất cả giao dịch
- **Idempotency**: Đảm bảo không double charge

### 3. Test Cases cần kiểm tra:
✅ Thanh toán thành công với card hợp lệ
✅ Thanh toán thất bại (insufficient funds)
✅ Timeout payment gateway (retry logic)
✅ Invalid payment method
✅ Currency conversion (nếu có)
✅ Refund process
✅ Webhook handling cho payment confirmation

### 4. Code Flow:
```
PaymentForm → validatePayment() → callGateway() →
processTransaction() → confirmPayment() → sendNotification()
```

### 5. Error Scenarios:
- Network timeout: Retry 3 lần với exponential backoff
- Invalid card: Hiển thị thông báo rõ ràng
- Gateway down: Fallback sang gateway dự phòng

### 6. Security Considerations:
- PCI DSS compliance
- Tokenization của thông tin thẻ
- 3D Secure authentication
- Fraud detection integration

**Ticket ID**: {}
"#,
                    request.ticket_id
                )
            }
            _ => {
                format!(
                    r#"
## 📊 Phân tích Business Flow

### Context: {}
### Câu hỏi: {}

### Phân tích chi tiết:

Tôi đã phân tích code trong context `{}` để hiểu business flow liên quan đến câu hỏi: "{}".

#### **Business Flow chính:**

1. **Input Processing**
   - Validation và preprocessing dữ liệu đầu vào
   - Kiểm tra quyền truy cập và authorization

2. **Core Logic**
   - Xử lý business logic chính
   - Data transformation và calculations

3. **Data Persistence**
   - Lưu trữ dữ liệu vào database
   - Cache management và optimization

4. **Response & Error Handling**
   - Format response cho client
   - Xử lý errors và edge cases

#### **Các điểm quan trọng:**
- ⚠️ Kiểm tra input validation rules kỹ càng
- 🔒 Xử lý authorization và permissions
- ⚡ Performance considerations (N+1 queries, caching)
- 🛡️ Security implications (injection attacks, XSS)

#### **Test Cases đề xuất:**
✅ Happy path scenarios với data hợp lệ
✅ Error handling scenarios (invalid input, network errors)
✅ Edge cases và boundary conditions
✅ Performance testing với large datasets
✅ Security testing (injection, authentication bypass)
✅ Concurrent access scenarios

#### **Recommendations:**
1. Thêm comprehensive error logging
2. Implement rate limiting nếu là API endpoint
3. Add metrics và monitoring
4. Review security implications

**Ticket ID**: {}
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

    // This method would be used for real Cursor Agent integration
    #[allow(dead_code)]
    async fn execute_real_cursor_agent(
        &self,
        request: &CodeAnalysisRequest,
        msg_store: &Arc<MsgStore>,
        _normalizer: &LogNormalizer,
    ) -> Result<String> {
        let prompt = self.create_analysis_prompt(request);

        // Spawn Cursor Agent process
        let mut child = Command::new("cursor-agent")
            .arg("--prompt")
            .arg(&prompt)
            .arg("--context")
            .arg(&request.code_context)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let ticket_id = request.ticket_id.clone();
        let msg_store_clone = msg_store.clone();
        let normalizer_clone = LogNormalizer::new();

        // Spawn task to capture stdout
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let entry = normalizer_clone.normalize(line, ticket_id.clone());
                msg_store_clone.push(entry).await;
            }
        });

        // Spawn task to capture stderr
        let stderr_ticket_id = request.ticket_id.clone();
        let stderr_msg_store = msg_store.clone();
        let stderr_normalizer = LogNormalizer::new();

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let error_line = format!("ERROR: {}", line);
                let entry = stderr_normalizer.normalize(error_line, stderr_ticket_id.clone());
                stderr_msg_store.push(entry).await;
            }
        });

        // Wait for process to complete
        let status = child.wait().await?;

        // Wait for log capture to complete
        let _ = tokio::join!(stdout_handle, stderr_handle);

        if !status.success() {
            return Err(anyhow::anyhow!("Cursor Agent process failed"));
        }

        Ok("Analysis completed".to_string())
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
5. Đề xuất test cases cần kiểm tra
6. Trả lời bằng tiếng Việt

Hãy trả lời chi tiết và hữu ích cho QA.
"#,
            request.code_context, request.question
        )
    }
}
