# Tokio Hang Test
This demonstrates a test that hangs forever, for no apparent reason.

Run the test with:

`RUST_LOG=trace cargo test -- --nocapture`

The output will look like this:

```text
running 1 test
binding
2022-12-05T16:54:35.037727Z TRACE tokio_hang_test::server::primus::test: built app
2022-12-05T16:54:35.038383Z TRACE mio::poll: registering event source with poller: token=Token(0), interests=READABLE | WRITABLE    
2022-12-05T16:54:35.038483Z TRACE tokio_hang_test::server::primus::test: built server, starting with graceful shutdown
2022-12-05T16:54:35.038503Z TRACE tokio_hang_test::server::primus::test: awaiting graceful shutdown
connecting first time
2022-12-05T16:54:36.041821Z TRACE mio::poll: registering event source with poller: token=Token(1), interests=READABLE | WRITABLE    
2022-12-05T16:54:36.042310Z TRACE tokio_tungstenite::handshake: Setting ctx when starting handshake    
2022-12-05T16:54:36.042367Z TRACE mio::poll: registering event source with poller: token=Token(2), interests=READABLE | WRITABLE    
2022-12-05T16:54:36.042466Z TRACE tungstenite::handshake::client: Request: "GET /primus/51013202-d080-4569-a905-021fc474f9db/?sid=abc&transport=websocket HTTP/1.1\r\nHost: 127.0.0.1:9899\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: rSpvdn+jI1UcomxZ5DMTOw==\r\n\r\n"    
2022-12-05T16:54:36.042550Z TRACE tungstenite::handshake::client: Client handshake initiated.    
2022-12-05T16:54:36.042572Z TRACE tungstenite::handshake::machine: Doing handshake round.    
2022-12-05T16:54:36.042591Z TRACE tokio_tungstenite::compat: /Users/athorburn/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-tungstenite-0.17.2/src/compat.rs:149 Read.read    
2022-12-05T16:54:36.042608Z TRACE tokio_tungstenite::compat: /Users/athorburn/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-tungstenite-0.17.2/src/compat.rs:126 AllowStd.with_context    
2022-12-05T16:54:36.042626Z TRACE tokio_tungstenite::compat: /Users/athorburn/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-tungstenite-0.17.2/src/compat.rs:152 Read.with_context read -> poll_read    
2022-12-05T16:54:36.042652Z TRACE tokio_tungstenite::handshake: Setting context in handshake    
2022-12-05T16:54:36.042670Z TRACE tungstenite::handshake::machine: Doing handshake round.    
2022-12-05T16:54:36.042687Z TRACE tokio_tungstenite::compat: /Users/athorburn/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-tungstenite-0.17.2/src/compat.rs:149 Read.read    
2022-12-05T16:54:36.042703Z TRACE tokio_tungstenite::compat: /Users/athorburn/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-tungstenite-0.17.2/src/compat.rs:126 AllowStd.with_context    
2022-12-05T16:54:36.042720Z TRACE tokio_tungstenite::compat: /Users/athorburn/.cargo/registry/src/github.com-1ecc6299db9ec823/tokio-tungstenite-0.17.2/src/compat.rs:152 Read.with_context read -> poll_read    
2022-12-05T16:54:36.047420Z TRACE hyper::proto::h1::conn: Conn::read_head
2022-12-05T16:54:36.047545Z TRACE hyper::proto::h1::conn: flushed({role=server}): State { reading: Init, writing: Init, keep_alive: Busy }
test server::primus::test::websocket_connects_and_closes has been running for over 60 seconds
```

However, while the test is running, you can make a cURL request against it, like so:

`curl -v -H 'Connection: Upgrade' -H 'Upgrade: websocket' -H 'Sec-WebSocket-Version: 13' -H 'Sec-WebSocket-Key: rSpvdn+jI1UcomxZ5DMTOw==' 'localhost:9899/primus/51013202-d080-4569-a905-021fc474f9db/?sid=abc&transport=websocket'`

Which will work as expected - it will go through the handshake and return a 101 response, along with much more logging information.