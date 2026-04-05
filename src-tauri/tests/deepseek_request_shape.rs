#[test]
fn deepseek_payload_contains_single_user_message() {
    let payload = smartocr_pro::services::ai_service::build_deepseek_payload("prompt", "ocr text");

    assert_eq!(payload["model"], "deepseek-chat");
    assert_eq!(payload["messages"][0]["role"], "user");
}
