use smartocr_pro::services::ai_service::{
    has_deepseek_api_key, structure_text_with_deepseek, DeepSeekRequest,
};

#[tokio::test]
#[ignore = "requires a real configured DeepSeek API key"]
async fn deepseek_live_call_returns_non_empty_markdown() {
    assert!(
        has_deepseek_api_key(),
        "DeepSeek API key not found in keyring/env; cannot run live verification"
    );

    let request = DeepSeekRequest::new(
        "请用一句话总结 OCR 文本内容。",
        "这是 SmartOCR Pro 的大模型在线联调验证文本。",
    );

    let result = structure_text_with_deepseek(request)
        .await
        .expect("live DeepSeek request should succeed");

    assert!(
        !result.markdown.trim().is_empty(),
        "DeepSeek returned empty markdown"
    );

    println!("deepseek_live_smoke_ok_len={}", result.markdown.len());
}
