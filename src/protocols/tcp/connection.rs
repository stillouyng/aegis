use async_trait::async_trait;
use std::io::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct TcpByteStream {
    stream: TcpStream,
}

impl TcpByteStream {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

#[async_trait]
pub trait ByteStream: Send + Sync + Unpin {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    async fn write(&mut self, data: &[u8]) -> std::io::Result<()>;
    async fn close(&mut self) -> ();
}

#[async_trait]
impl ByteStream for TcpByteStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf).await
    }

    async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await
    }

    async fn close(&mut self) -> () {
        let _ = self.stream.shutdown().await;
    }
}
