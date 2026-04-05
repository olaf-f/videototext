use smartocr_pro::commands::imports::validate_image_url_content_type;
use smartocr_pro::services::url_import::{
    extract_display_name_from_url, parse_supported_http_url, validate_image_content_type,
};

#[test]
fn rejects_html_content_type() {
    let error = validate_image_content_type("text/html; charset=utf-8").unwrap_err();

    assert!(
        error.to_string().contains("text/html"),
        "expected error to mention rejected content type, got: {error}"
    );
}

#[test]
fn rejects_non_http_url_for_image_validation() {
    let error = validate_image_url_content_type(
        "file:///tmp/image.png".into(),
        "image/png".into(),
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("unsupported url scheme"),
        "expected unsupported url scheme error, got: {error}"
    );
}

#[test]
fn accepts_png_content_type() {
    validate_image_content_type("image/png").unwrap();
}

#[test]
fn extracts_display_name_from_url_path() {
    let value = extract_display_name_from_url("https://example.com/images/cat-photo.png?x=1#y");

    assert_eq!(value.as_deref(), Some("cat-photo.png"));
}

#[test]
fn falls_back_to_none_when_url_has_no_path_filename() {
    let value = extract_display_name_from_url("https://example.com/images/");

    assert_eq!(value, None);
}

#[test]
fn parse_supported_http_url_rejects_ftp() {
    let result = parse_supported_http_url("ftp://example.com/a.png");

    assert!(result.is_err());
}

#[test]
fn parse_supported_http_url_accepts_https() {
    let result = parse_supported_http_url("https://example.com/a.png");

    assert!(result.is_ok());
}
