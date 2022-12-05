use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use axum::{Extension, Router};
use axum::routing::any;
use dashmap::DashMap;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::server::primus::{handle_primus_request, PrimusSocketState};

pub mod primus;

#[derive(Debug)]
pub struct MyState {
    pub primus_state: SharedPrimusState,
}

pub type SharedBrokerState = Arc<MyState>;

#[derive(Debug)]
pub struct PrimusState {
    pub sockets: DashMap<Uuid, Sender<String>>,
    pub requests: DashMap<u64, Sender<String>>,
    pub primus_poll_requests: DashMap<String, tokio::sync::oneshot::Sender<Option<()>>>,
    pub primus_poll_state: DashMap<String, PrimusSocketState>,
    pub request_count: AtomicU64,
}

pub type SharedPrimusState = Arc<PrimusState>;

pub fn build_router(state: SharedBrokerState) -> Router {
    Router::new()
        .route("/primus/:token/", any(handle_primus_request))
        .layer(Extension(state))
}