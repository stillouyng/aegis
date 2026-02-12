use async_trait::async_trait;
use std::io::Result;
use std::net::SocketAddr;

use super::connection::{ByteStream, TcpByteStream};

pub struct TcpByteListener {
    inner: tokio::net::TcpListener,
}

#[async_trait]
pub trait Listener: Send + Sync {
    async fn accept(&self) -> Result<(Box<dyn ByteStream>, SocketAddr)>;
}

impl TcpByteListener {
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        Ok(Self { inner: listener })
    }
}

#[async_trait]
impl Listener for TcpByteListener {
    async fn accept(&self) -> Result<(Box<dyn ByteStream>, SocketAddr)> {
        let (stream, addr) = self.inner.accept().await?;

        let byte_stream: Box<dyn ByteStream> = Box::new(TcpByteStream::new(stream));

        Ok((byte_stream, addr))
    }
}
