//! # Verifiable Credential Provider
//!
//! This is a simple Verifiable Credential Provider (VCP) that implements the
//! [Verifiable Credential HTTP API](
//! https://identity.foundation/verifiable-credential/spec/#http-api).

mod provider;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_extra::headers::Host;
use axum_extra::TypedHeader;
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use vercre_verifier::{
    CreateRequestRequest, CreateRequestResponse, RequestObjectRequest, RequestObjectResponse,
    ResponseRequest,
};

use crate::provider::Provider;

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any).allow_headers(Any);

    let router = Router::new()
        .route("/create_request", post(create_request))
        .route("/request/:object_id", get(request_object))
        .route("/callback", get(response))
        .route("/post", post(response))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(Provider::new());

    let listener = TcpListener::bind("0.0.0.0:8080").await.expect("should bind");
    tracing::info!("listening on {}", listener.local_addr().expect("local_addr should be set"));
    axum::serve(listener, router).await.expect("should run");
}

// Generate Authorization Request endpoint
#[axum::debug_handler]
async fn create_request(
    State(provider): State<Provider>, TypedHeader(host): TypedHeader<Host>,
    Json(mut request): Json<CreateRequestRequest>,
) -> AxResult<CreateRequestResponse> {
    request.client_id = format!("http://{host}");
    vercre_verifier::create_request(provider, &request).await.into()
}

// Retrieve Authorization Request Object endpoint
#[axum::debug_handler]
async fn request_object(
    State(provider): State<Provider>, TypedHeader(host): TypedHeader<Host>,
    Path(object_id): Path<String>,
) -> AxResult<RequestObjectResponse> {
    let request = RequestObjectRequest {
        client_id: format!("http://{host}"),
        id: object_id,
    };
    vercre_verifier::request_object(provider, &request).await.into()
}

// Wallet Authorization response endpoint
#[axum::debug_handler]
async fn response(
    State(provider): State<Provider>, Form(request): Form<ResponseRequest>,
) -> impl IntoResponse {
    let response = vercre_verifier::response(provider, &request).await;
    AxResult(response)
}

// ----------------------------------------------------------------------------
// Axum Response
// ----------------------------------------------------------------------------

/// Axum response wrapper
pub struct AxResult<T>(vercre_verifier::Result<T>);

impl<T> IntoResponse for AxResult<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self.0 {
            Ok(v) => (StatusCode::OK, Json(json!(v))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_json())),
        }
        .into_response()
    }
}

impl<T> From<vercre_verifier::Result<T>> for AxResult<T> {
    fn from(val: vercre_verifier::Result<T>) -> Self {
        Self(val)
    }
}
