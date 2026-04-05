use smartocr_pro::error::AppError;
use smartocr_pro::services::ai_service::{structure_text_with_deepseek, DeepSeekRequest};

#[tokio::test]
#[ignore = "manual probe: validates request flow with a non-empty key"]
async fn deepseek_request_flow_reaches_remote_or_returns_non_key_error() {
    std::env::set_var("SMARTOCR_PRO_DEEPSEEK_API_KEY", "sk-invalid-smoke-test");

    let request = DeepSeekRequest::new("输出一句话。", "联调验证");
    let result = structure_text_with_deepseek(request).await;

    match result {
        Ok(response) => {
            println!("deepseek_flow_probe_ok_len={}", response.markdown.len());
            assert!(
                !response.markdown.trim().is_empty(),
                "markdown should not be empty when request succeeds"
            );
        }
        Err(error) => {
            println!("deepseek_flow_probe_err={error}");
            assert!(
                !matches!(error, AppError::DeepSeekApiKeyUnavailable),
                "key should be available during probe"
            );
        }
    }
}
