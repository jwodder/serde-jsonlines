#![cfg(feature = "async")]
#![cfg_attr(docsrs, doc(cfg(feature = "async")))]
use futures::ready;
use futures::sink::Sink;
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::io::Result;
use std::marker::{PhantomData, Unpin};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, Lines};
use tokio_stream::Stream;

pin_project! {
    /// A structure for asynchronously reading JSON values from JSON Lines
    /// input.
    ///
    /// An `AsyncJsonLinesReader` wraps a [`tokio::io::AsyncBufRead`] instance
    /// and parses each line as a [`serde::de::DeserializeOwned`] value in
    /// JSON.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde::Deserialize;
    /// use serde_jsonlines::AsyncJsonLinesReader;
    /// use tokio::fs::{write, File};
    /// use tokio::io::BufReader;
    /// use tokio_stream::StreamExt;
    ///
    /// #[derive(Debug, Deserialize, PartialEq)]
    /// pub struct Structure {
    ///     pub name: String,
    ///     pub size: i32,
    ///     pub on: bool,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     write(
    ///         "example.jsonl",
    ///         concat!(
    ///             "{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n",
    ///             "{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n",
    ///             " {\"name\": \"Gnusto Cleesh\" , \"on\": true, \"size\": 17}\n",
    ///         ),
    ///     )
    ///     .await?;
    ///     let fp = BufReader::new(File::open("example.jsonl").await?);
    ///     let reader = AsyncJsonLinesReader::new(fp);
    ///     let items = reader
    ///         .read_all::<Structure>()
    ///         .collect::<std::io::Result<Vec<_>>>()
    ///         .await?;
    ///     assert_eq!(
    ///         items,
    ///         [
    ///             Structure {
    ///                 name: "Foo Bar".into(),
    ///                 size: 42,
    ///                 on: true,
    ///             },
    ///             Structure {
    ///                 name: "Quux".into(),
    ///                 size: 23,
    ///                 on: false,
    ///             },
    ///             Structure {
    ///                 name: "Gnusto Cleesh".into(),
    ///                 size: 17,
    ///                 on: true,
    ///             },
    ///         ]
    ///     );
    ///     Ok(())
    /// }
    /// ```
    #[derive(Clone, Debug, Eq, PartialEq)]
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
    /// Asynchronously read & deserialize a line of JSON from the underlying
    /// reader.
    ///
    /// If end-of-file is reached, this method returns `Ok(None)`.
    ///
    /// Note that separate calls to this method may read different types of
    /// values.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as
    /// [`tokio::io::AsyncBufReadExt::read_line()`] and
    /// [`serde_json::from_str()`].  Note that, in the latter case (which can
    /// be identified by the [`std::io::Error`] having a [`serde_json::Error`]
    /// value as its payload), continuing to read from the
    /// `AsyncJsonLinesReader` afterwards will pick up on the next line as
    /// though the error never happened, so invalid JSON can be easily ignored
    /// if you so wish.
    #[allow(clippy::future_not_send)] // The Future is Send if R is Send
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

    /// Consume the `AsyncJsonLinesReader` and return an asynchronous stream
    /// over the deserialized JSON values from each line.
    ///
    /// The returned stream has an `Item` type of `std::io::Result<T>`.  Each
    /// call to `next()` has the same error conditions as
    /// [`read()`][AsyncJsonLinesReader::read].
    ///
    /// Note that all deserialized values will be of the same type.  If you
    /// wish to read lines of varying types, use the
    /// [`read()`][AsyncJsonLinesReader::read] method instead.
    pub fn read_all<T>(self) -> JsonLinesStream<R, T> {
        JsonLinesStream {
            inner: self.inner.lines(),
            _output: PhantomData,
        }
    }
}

pin_project! {
    /// An asynchronous stream over the lines of an [`AsyncBufRead`] value `R`
    /// that decodes each line as JSON of type `T`.
    ///
    /// This stream yields items of type `Result<T, std::io::Error>`.  Errors
    /// occurr under the same conditions as for
    /// [`AsyncJsonLinesReader::read()`].
    ///
    /// Streams of this type are returned by
    /// [`AsyncJsonLinesReader::read_all()`] and
    /// [`AsyncBufReadJsonLines::json_lines()`].
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
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
        match ready!(self.project().inner.poll_next_line(cx)) {
            Ok(Some(line)) => Some(serde_json::from_str::<T>(&line).map_err(Into::into)).into(),
            Ok(None) => None.into(),
            Err(e) => Some(Err(e)).into(),
        }
    }
}

pin_project! {
    /// A structure for asynchronously writing JSON values as JSON Lines.
    ///
    /// An `AsyncJsonLinesWriter` wraps a [`tokio::io::AsyncWrite`] instance
    /// and writes [`serde::Serialize`] values to it by serializing each one as
    /// a single line of JSON and appending a newline.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde::Serialize;
    /// use serde_jsonlines::AsyncJsonLinesWriter;
    /// use tokio::fs::{read_to_string, File};
    ///
    /// #[derive(Serialize)]
    /// pub struct Structure {
    ///     pub name: String,
    ///     pub size: i32,
    ///     pub on: bool,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     {
    ///         let fp = File::create("example.jsonl").await?;
    ///         let writer = AsyncJsonLinesWriter::new(fp);
    ///         tokio::pin!(writer);
    ///         writer
    ///             .write(&Structure {
    ///                 name: "Foo Bar".into(),
    ///                 size: 42,
    ///                 on: true,
    ///             })
    ///             .await?;
    ///         writer
    ///             .write(&Structure {
    ///                 name: "Quux".into(),
    ///                 size: 23,
    ///                 on: false,
    ///             })
    ///             .await?;
    ///         writer
    ///             .write(&Structure {
    ///                 name: "Gnusto Cleesh".into(),
    ///                 size: 17,
    ///                 on: true,
    ///             })
    ///             .await?;
    ///         writer.flush().await?;
    ///     }
    ///     // End the block to close the writer
    ///     assert_eq!(
    ///         read_to_string("example.jsonl").await?,
    ///         concat!(
    ///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
    ///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
    ///             "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
    ///         )
    ///     );
    ///     Ok(())
    /// }
    /// ```
    #[derive(Clone, Debug, Eq, PartialEq)]
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

    /// Consume the `AsyncJsonLinesWriter` and return an asynchronous sink
    /// for serializing values as JSON and writing them to the underlying
    /// writer.
    ///
    /// The returned sink consumes `T` values and has an `Error` type of
    /// [`std::io::Error`].  Each call to `send()` has the same error
    /// conditions as [`write()`][AsyncJsonLinesWriter::write].
    ///
    /// Note that all values sent to the sink must be of the same type.  If you
    /// wish to write values of varying types, use the
    /// [`write()`][AsyncJsonLinesWriter::write] method.
    pub fn into_sink<T>(self) -> JsonLinesSink<W, T> {
        JsonLinesSink::new(self.inner)
    }
}

impl<W: AsyncWrite> AsyncJsonLinesWriter<W> {
    /// Serialize a value as a line of JSON and write it asynchronously to the
    /// underlying writer, followed by a newline.
    ///
    /// Note that separate calls to this method may write different types of
    /// values.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`serde_json::to_writer()`] and
    /// [`tokio::io::AsyncWriteExt::write_all()`].
    #[allow(clippy::future_not_send)] // The Future is Send if W is Send
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

    /// Flush the underlying writer.
    ///
    /// [`write()`][AsyncJsonLinesWriter::write] does not flush the writer, so
    /// you must explicitly call this method if you need output flushed.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`tokio::io::AsyncWriteExt::flush()`].
    #[allow(clippy::future_not_send)] // The Future is Send if W is Send
    pub async fn flush(&mut self) -> Result<()>
    where
        W: Unpin,
    {
        self.inner.flush().await
    }
}

pin_project! {
    /// An asynchronous sink that serializes input values of type `T` as JSON
    /// and writes them to the underlying [`AsyncWrite`] value `W`.
    ///
    /// Sinks of this type are returned by
    /// [`AsyncJsonLinesWriter::into_sink()`] and
    /// [`AsyncWriteJsonLines::into_json_lines_sink()`].
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[must_use = "sinks do nothing unless polled"]
    pub struct JsonLinesSink<W, T> {
        #[pin]
        inner: W,
        buffer: Option<Vec<u8>>,
        offset: usize,
        _input: PhantomData<T>,
    }
}

impl<W, T> JsonLinesSink<W, T> {
    fn new(writer: W) -> Self {
        JsonLinesSink {
            inner: writer,
            buffer: None,
            offset: 0,
            _input: PhantomData,
        }
    }

    // Based on the implementation of futures::io::IntoSink
    fn poll_flush_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>>
    where
        W: AsyncWrite,
    {
        let mut this = self.project();
        if let Some(buffer) = this.buffer {
            loop {
                let written = ready!(this.inner.as_mut().poll_write(cx, &buffer[*this.offset..]))?;
                *this.offset += written;
                if *this.offset == buffer.len() {
                    break;
                }
            }
        }
        *this.buffer = None;
        Poll::Ready(Ok(()))
    }
}

impl<W: AsyncWrite, T> Sink<T> for JsonLinesSink<W, T>
where
    T: Serialize,
{
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.poll_flush_buffer(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<()> {
        debug_assert!(
            self.buffer.is_none(),
            "buffer should be None after calling poll_ready()"
        );
        let this = self.project();
        let mut buf = serde_json::to_vec(&item)?;
        buf.push(b'\n');
        *this.buffer = Some(buf);
        *this.offset = 0;
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        ready!(self.as_mut().poll_flush_buffer(cx))?;
        ready!(self.project().inner.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        ready!(self.as_mut().poll_flush_buffer(cx))?;
        ready!(self.project().inner.poll_shutdown(cx))?;
        Poll::Ready(Ok(()))
    }
}

/// An extension trait for the [`tokio::io::AsyncBufRead`] trait that adds a
/// `json_lines()` method
///
/// # Example
///
/// ```no_run
/// use serde::Deserialize;
/// use serde_jsonlines::AsyncBufReadJsonLines;
/// use std::io::Result;
/// use tokio::fs::{write, File};
/// use tokio::io::BufReader;
/// use tokio_stream::StreamExt;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     write(
///         "example.jsonl",
///         concat!(
///             "{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n",
///             "{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n",
///             " {\"name\": \"Gnusto Cleesh\" , \"on\": true, \"size\": 17}\n",
///         ),
///     )
///     .await?;
///     let fp = BufReader::new(File::open("example.jsonl").await?);
///     let items = fp
///         .json_lines::<Structure>()
///         .collect::<Result<Vec<_>>>()
///         .await?;
///     assert_eq!(
///         items,
///         [
///             Structure {
///                 name: "Foo Bar".into(),
///                 size: 42,
///                 on: true,
///             },
///             Structure {
///                 name: "Quux".into(),
///                 size: 23,
///                 on: false,
///             },
///             Structure {
///                 name: "Gnusto Cleesh".into(),
///                 size: 17,
///                 on: true,
///             },
///         ]
///     );
///     Ok(())
/// }
/// ```
pub trait AsyncBufReadJsonLines: AsyncBufRead {
    /// Consume the reader and return an asynchronous stream over the
    /// deserialized JSON values from each line.
    ///
    /// The returned stream has an `Item` type of `std::io::Result<T>`.  Each
    /// call to `next()` has the same error conditions as
    /// [`read()`][AsyncJsonLinesReader::read].
    ///
    /// Note that all deserialized values will be of the same type.
    fn json_lines<T>(self) -> JsonLinesStream<Self, T>
    where
        Self: Sized,
    {
        JsonLinesStream {
            inner: self.lines(),
            _output: PhantomData,
        }
    }
}

impl<R: AsyncBufRead> AsyncBufReadJsonLines for R {}

/// An extension trait for the [`tokio::io::AsyncWrite`] trait that adds an
/// `into_json_lines_sink()` method
///
/// # Example
///
/// ```no_run
/// use futures::sink::SinkExt;
/// use serde::Serialize;
/// use serde_jsonlines::AsyncWriteJsonLines;
/// use tokio::fs::{read_to_string, File};
///
/// #[derive(Serialize)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     {
///         let fp = File::create("example.jsonl").await?;
///         let sink = fp.into_json_lines_sink();
///         tokio::pin!(sink);
///         sink.send(Structure {
///             name: "Foo Bar".into(),
///             size: 42,
///             on: true,
///         })
///         .await?;
///         sink.send(Structure {
///             name: "Quux".into(),
///             size: 23,
///             on: false,
///         })
///         .await?;
///         sink.send(Structure {
///             name: "Gnusto Cleesh".into(),
///             size: 17,
///             on: true,
///         })
///         .await?;
///         sink.close().await?;
///     }
///     assert_eq!(
///         read_to_string("example.jsonl").await?,
///         concat!(
///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
///             "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
///         )
///     );
///     Ok(())
/// }
/// ```
pub trait AsyncWriteJsonLines: AsyncWrite {
    /// Consume the writer and return an asynchronous sink for serializing
    /// values as JSON and writing them to the writer.
    ///
    /// The returned sink consumes `T` values and has an `Error` type of
    /// [`std::io::Error`].  Each call to `send()` has the same error
    /// conditions as [`AsyncJsonLinesWriter::write()`].
    ///
    /// Note that all values sent to the sink must be of the same type.
    fn into_json_lines_sink<T>(self) -> JsonLinesSink<Self, T>
    where
        Self: Sized,
    {
        JsonLinesSink::new(self)
    }
}

impl<W: AsyncWrite> AsyncWriteJsonLines for W {}

#[cfg(test)]
mod tests {
    use super::*;

    fn require_send<T: Send>(_t: T) {}

    #[test]
    fn test_read_is_send_if_r_is_send() {
        let mut ajreader = AsyncJsonLinesReader::new(tokio::io::empty());
        let fut = ajreader.read::<String>();
        require_send(fut);
    }

    #[test]
    fn test_write_is_send_if_w_is_send() {
        let mut ajwriter = AsyncJsonLinesWriter::new(tokio::io::sink());
        let s = String::from("This is test text.");
        let fut = ajwriter.write(&s);
        require_send(fut);
    }

    #[test]
    fn test_flush_is_send_if_w_is_send() {
        let mut ajwriter = AsyncJsonLinesWriter::new(tokio::io::sink());
        let fut = ajwriter.flush();
        require_send(fut);
    }
}
