//! Public, session-free procedural art showcase.

use axum::{Router, response::Html, routing::get};

const ART_DEMO_PATH: &str = "/art-demo";

pub(super) fn routes() -> Router {
    Router::new().route(ART_DEMO_PATH, get(page))
}

async fn page() -> Html<&'static str> {
    Html(include_str!("../static/art-demo/index.html"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn anonymous_get_serves_showcase_without_session_or_database() {
        let router = Router::new().route(ART_DEMO_PATH, get(page));
        let response = router
            .oneshot(
                Request::builder()
                    .uri(ART_DEMO_PATH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key("set-cookie"));
        assert!(!response.headers().contains_key("location"));
    }
}
