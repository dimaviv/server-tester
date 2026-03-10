use axum::http::header;
use axum::response::Html;

pub async fn index_html() -> Html<&'static str> {
    Html(crate::embedded::INDEX_HTML)
}

pub async fn style_css() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    ([(header::CONTENT_TYPE, "text/css")], crate::embedded::STYLE_CSS)
}

pub async fn app_js() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        crate::embedded::APP_JS,
    )
}
