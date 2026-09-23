use std::sync::Arc;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{sse::Event, Json, Sse},
    routing::{get, post},
    Router,
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use crate::security::auth::validate_bearer_token;
use crate::server::dispatcher::McpDispatcher;
use crate::server::protocol::{JsonRpcRequest, JsonRpcResponse};

#[derive(Clone)]
pub struct AppState {
    pub dispatcher: Arc<McpDispatcher>,
    pub auth_token: String,
}

pub async fn run_network_transport(
    interface: &str,
    port: u16,
    auth_token: &str,
    dispatcher: Arc<McpDispatcher>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        dispatcher,
        auth_token: auth_token.to_string(),
    };

    let app = create_network_router(state);

    let addr: SocketAddr = format!("{}:{}", interface, port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("desktop-mcp-daemon network transport listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

pub fn create_network_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/messages", post(messages_handler))
        .route("/sse", get(sse_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_handler() -> &'static str {
    "desktop-mcp-daemon OK"
}

async fn messages_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    // 1. Verify Bearer token
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    if !validate_bearer_token(auth_header, &state.auth_token) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 2. Dispatch MCP request
    let response = state.dispatcher.dispatch(request).await;
    Ok(Json(response))
}

async fn sse_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Verify Bearer token
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    if !validate_bearer_token(auth_header, &state.auth_token) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Initial endpoint announcement event as per MCP SSE spec
    let initial_event = Event::default()
        .event("endpoint")
        .data("/messages");

    let stream = stream::once(async { Ok(initial_event) });
    Ok(Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15))))
}
