use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router, Server,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bors=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/webhook", post(handle_payload))
        .layer(TraceLayer::new_for_http());

    let addr = "[::]:8080".parse().unwrap();
    tracing::info!("Listening on http://{addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h2>Hello from bors 🤖<h2>")
}

async fn health() -> Html<String> {
    Html("<h4>OK ✅</h4>".to_string())
}

async fn handle_payload(req: Request<Body>) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("request = {:?}", req);
    let (head, body) = req.into_parts();

    let _event = match head.headers.get("x-github-event") {
        Some(e) => e,
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "X-GitHub-Event header not set".to_owned(),
            ));
        }
    };

    let payload = hyper::body::to_bytes(body).await.unwrap();
    let payload =
        serde_json::from_str::<IssueEvent>(std::str::from_utf8(&payload).unwrap()).unwrap();

    Ok(Json(payload))
}

#[derive(Debug, Deserialize, Serialize)]
struct IssueEvent {
    action: IssueAction,
    issue: Issue,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum IssueAction {
    Opened,
    Closed,
}

#[derive(Debug, Deserialize, Serialize)]
struct Issue {
    number: u64,
    title: String,
    body: String,
}
