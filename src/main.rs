use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Html,
    routing::{get, post},
    Json, Router, Server,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod payload;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

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

    let port = std::env::var("PORT")
        .ok()
        .map(|p| p.parse::<u16>().expect("Unable to parse PORT"))
        .unwrap_or(3000);
    let addr = ([0, 0, 0, 0], port).into();
    tracing::info!("Listening on http://{addr}");
    if let Err(e) = Server::bind(&addr).serve(app.into_make_service()).await {
        eprintln!("Error running server: {:?}", e);
    }
}

async fn root() -> Html<&'static str> {
    Html("<h2>Hello from bors 🤖</h2>")
}

async fn health() -> Html<&'static str> {
    Html("<h4>OK ✅</h4>")
}

async fn handle_payload(
    req: Request<Body>,
) -> Result<Json<IssueEvent>, (StatusCode, &'static str)> {
    let (head, body) = req.into_parts();
    // todo Wrap this with RequestBodyLimiter; limit is not applied when consuming body with Body::data
    // See: https://github.com/tokio-rs/axum/pull/1346 and
    // https://docs.rs/axum/latest/axum/extract/struct.DefaultBodyLimit.html
    // perhaps extract signature headers before consuming body since we know remote is trusted?
    let payload = hyper::body::to_bytes(body).await.unwrap();

    let signature = match head.headers.get("X-Hub-Signature-256") {
        Some(sig) => sig.to_str().unwrap(),
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "X-Hub-Signature-256 header not set",
            ))
        }
    };

    // todo read this into a Context struct to easily share across app
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").expect("GITHUB_WEBHOOK_SECRET not found");

    payload::verify_payload(&secret, signature, &payload)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Signature's do not match"))?;

    let _event = head
        .headers
        .get("X-GitHub-Event")
        .ok_or((StatusCode::BAD_REQUEST, "X-GitHub-Event header not set"))?;

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
