extern crate core;
extern crate tokio_hang_test;

use std::env;
use std::sync::Arc;
use tokio_hang_test::{MyResult, MyState, PrimusState};


#[tokio::main]
async fn main() -> MyResult<()> {
    let state = Arc::new(MyState {
        primus_state: Arc::new(PrimusState {
            sockets: Default::default(),
            requests: Default::default(),
            primus_poll_requests: Default::default(),
            primus_poll_state: Default::default(),
            request_count: Default::default()
        })
    });

    let app = tokio_hang_test::build_router(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:7341".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}