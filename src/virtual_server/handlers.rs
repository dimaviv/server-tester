use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use std::collections::HashMap;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub async fn up_handler(
    http_status: u16,
    html_title: String,
    response_body: Option<String>,
    custom_headers: HashMap<String, String>,
    delay_ms: u64,
) -> Response {
    if delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }

    let status = StatusCode::from_u16(http_status).unwrap_or(StatusCode::OK);
    let body = response_body.unwrap_or_else(|| {
        let title = escape_html(&html_title);
        format!(
            "<!DOCTYPE html>\n<html><head><title>{title}</title></head>\
             <body><h1>{title}</h1><p>Server is running. Status: {code}</p></body></html>",
            title = title,
            code = http_status
        )
    });

    let mut response = (status, Html(body)).into_response();
    for (key, value) in &custom_headers {
        if let (Ok(name), Ok(val)) = (key.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
            response.headers_mut().insert(name, val);
        }
    }
    response
}

pub async fn down_503_handler(html_title: String) -> Response {
    let title = escape_html(&html_title);
    let body = format!(
        "<!DOCTYPE html>\n<html><head><title>{title}</title></head>\
         <body><h1>503 Service Unavailable</h1><p>{title} is currently down.</p></body></html>",
        title = title
    );
    (StatusCode::SERVICE_UNAVAILABLE, Html(body)).into_response()
}
