use async_trait::async_trait;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
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
pub trait ByteStream: AsyncRead + AsyncWrite + Send + Unpin {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    async fn write(&mut self, data: &[u8]) -> std::io::Result<()>;
    async fn close(&mut self) -> ();
}
impl AsyncRead for TcpByteStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpByteStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
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
