use std::collections::HashMap;
use std::mem::replace;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use async_trait::async_trait;
use axum::{Extension, Router};
use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::http::Request;
use axum::response::Response;
use axum::routing::any;
use bytes::Bytes;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use hyper::Body;
use tokio::spawn;
use tokio::sync::mpsc::{channel, Sender};
use tracing::{error, info, instrument, trace, warn};
use uuid::Uuid;

pub type MyResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncMyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

#[derive(Debug)]
pub enum PrimusSocketState {
    Polling,
    WebSocket,
}

#[instrument(skip(state))]
pub async fn handle_primus_request(ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>, Extension(state): Extension<SharedBrokerState>, Path(token): Path<Uuid>, request: Request<Body>) -> Response {
    info!("Handling Primus request");
    let q = request.uri().query().unwrap();
    let query_params: HashMap<&str, &str> = q.split("&").map(|s| s.split_once("=").unwrap()).collect();
    let sid = query_params.get("sid").unwrap().to_string();
    return ws.unwrap().on_upgrade(move |socket| handle_primus_ws(socket, state, token, sid));
}

async fn handle_primus_ws(socket: WebSocket, state: SharedBrokerState, token: Uuid, sid: String) {
    let (mut send, mut recv) = socket.split();
    let (tx, mut rx) = channel::<String>(20);
    state.primus_state.sockets.insert(token.clone(), tx.clone());
    {
        let state = state.clone();
        spawn(async move {
            loop {
                if let Some(m) = rx.recv().await {
                    trace!(m, "received message from internal queue, sending over web socket");
                    send.send(Message::Text("3".to_string())).await.unwrap();
                } else {
                    break;
                }
            }
        });
    }

    let tx = tx.clone();
    // let mut ping_start = Some(ping_start_tx);
    loop {
        match recv.next().await {
            None => break,
            Some(opt) => {
                match opt {
                    Ok(message) => {
                        match message {
                            Message::Text(text) => {
                                trace!(text, "received message from web socket, sending response to internal queue");
                                tx.send("2".into());
                            }
                            Message::Close(reason) => {
                                state.primus_state.sockets.remove(&token);
                                state.primus_state.primus_poll_state.remove(&sid);
                                if let Some(r) = reason {
                                    warn!(code = r.code, reason = r.reason.as_ref(), "connection unexpectedly closed");
                                }
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                    Err(err) => {
                        error!(?err, "got error instead of message from WebSocket - likely closed");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::Duration;

    use futures_util::{SinkExt, StreamExt};
    use tokio::spawn;
    use tokio::sync::oneshot::Receiver;
    use tracing::trace;
    use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    use crate::{MyState, PrimusState};

    // #[tokio::main]
    async fn start_server(rx: Receiver<()>) {
        let state = Arc::new(MyState {
            primus_state: Arc::new(PrimusState {
                sockets: Default::default(),
                requests: Default::default(),
                primus_poll_requests: Default::default(),
                primus_poll_state: Default::default(),
                request_count: Default::default()
            })
        });

        // let monitor_root = tokio_metrics::TaskMonitor::new();

        let app = crate::build_router(state);
        trace!("built app");
        let server = axum::Server::bind(&"127.0.0.1:9899".parse::<SocketAddr>().unwrap())
            .serve(app.into_make_service());

        trace!("built server, starting with graceful shutdown");
        server.with_graceful_shutdown(async {
            trace!("awaiting graceful shutdown");
            rx.await.ok();
        }).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn websocket_connects_and_closes() {
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).with(tracing_subscriber::EnvFilter::from_default_env()).init();
        println!("binding");
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        spawn(start_server(rx));

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("connecting first time");
        let mut ws = tokio_tungstenite::connect_async(format!("ws://{}:{}/primus/51013202-d080-4569-a905-021fc474f9db/?sid=abc&transport=websocket", "127.0.0.1", "9899")).await.unwrap();

        println!("sending first time");
        ws.0.send(tokio_tungstenite::tungstenite::protocol::Message::Close(None)).await.unwrap();

        drop(ws);

        println!("connecting second time");
        let mut ws1 = tokio_tungstenite::connect_async(format!("ws://{}:{}/primus/51013202-d080-4569-a905-021fc474f9db/?sid=abc&transport=websocket", "127.0.0.1", "9899")).await.unwrap();

        println!("sending second time");
        ws1.0.send(tokio_tungstenite::tungstenite::protocol::Message::Text("2".to_string())).await.unwrap();

        println!("receiving");
        // let duration = Duration::from_secs(30);
        let n = ws1.0.next().await.unwrap().unwrap();
        assert!(matches!(n, tokio_tungstenite::tungstenite::protocol::Message::Text(t) if t == "3".to_string()));

        tx.send(()).unwrap();
    }
}