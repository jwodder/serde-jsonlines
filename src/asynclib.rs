#![cfg(feature = "async")]
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::io::Result;
use std::marker::PhantomData;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, Lines};
use tokio_stream::Stream;

pin_project! {
    #[derive(Debug)]
    pub struct AsyncJsonLinesReader<R> {
        #[pin]
        inner: R,
    }
}

impl<R> AsyncJsonLinesReader<R> {
    /// Construct a new `AsyncJsonLinesReader` from a
    /// [`tokio::io::AsyncBufRead`] instance
    pub fn new(reader: R) -> Self {
        AsyncJsonLinesReader { inner: reader }
    }

    /// Consume the `AsyncJsonLinesReader` and return the underlying reader
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Get a reference to the underlying reader
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Get a mutable reference to the underlying reader
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Get a pinned mutable reference to the underlying reader
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
        self.project().inner
    }
}

impl<R: AsyncBufRead> AsyncJsonLinesReader<R> {
    pub async fn read<T>(&mut self) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        R: Unpin,
    {
        let mut s = String::new();
        let r = self.inner.read_line(&mut s).await?;
        if r == 0 {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_str::<T>(&s)?))
        }
    }

    pub fn read_all<T>(self) -> JsonLinesStream<R, T> {
        JsonLinesStream {
            inner: self.inner.lines(),
            _output: PhantomData,
        }
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct JsonLinesStream<R, T> {
        #[pin]
        inner: Lines<R>,
        _output: PhantomData<T>,
    }
}

impl<R: AsyncBufRead, T> Stream for JsonLinesStream<R, T>
where
    T: DeserializeOwned,
{
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_next_line(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(Some(line))) => {
                Some(serde_json::from_str::<T>(&line).map_err(Into::into)).into()
            }
            Poll::Ready(Ok(None)) => None.into(),
            Poll::Ready(Err(e)) => Some(Err(e)).into(),
        }
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct AsyncJsonLinesWriter<W> {
        #[pin]
        inner: W,
    }
}

impl<W> AsyncJsonLinesWriter<W> {
    /// Construct a new `AsyncJsonLinesWriter` from a
    /// [`tokio::io::AsyncWrite`] instance
    pub fn new(writer: W) -> Self {
        AsyncJsonLinesWriter { inner: writer }
    }

    /// Consume the `AsyncJsonLinesWriter` and return the underlying writer
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Get a reference to the underlying writer
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Get a mutable reference to the underlying writer
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Get a pinned mutable reference to the underlying writer
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut W> {
        self.project().inner
    }
}

impl<W: AsyncWrite> AsyncJsonLinesWriter<W> {
    pub async fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
        W: Unpin,
    {
        let mut buf = serde_json::to_vec(value)?;
        buf.push(b'\n');
        self.inner.write_all(&buf).await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()>
    where
        W: Unpin,
    {
        self.inner.flush().await
    }
}
