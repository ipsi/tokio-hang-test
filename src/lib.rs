use std::mem::replace;
use async_trait::async_trait;
use bytes::Bytes;
use hyper::Body;

pub mod server;

pub type MyResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncMyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;