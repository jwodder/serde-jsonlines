//! Read & write JSON Lines documents
//!
//! JSON Lines (a.k.a. newline-delimited JSON) is a simple format for
//! representing sequences of JSON values in which each value is serialized on
//! a single line and terminated by a newline sequence.
//!
//! This crate provides functionality for reading & writing JSON Lines
//! documents, either all at once or line by line.
use serde::{de::DeserializeOwned, Serialize};
use std::io::{BufRead, Result, Write};
use std::marker::PhantomData;

/// A structure for writing JSON values as JSON Lines
///
/// A `JsonLinesWriter` wraps a [`std::io::Write`] instance and writes
/// [`serde::Serialize`] values to it by serializing each one as a single line
/// of JSON and appending a newline.
#[derive(Debug)]
pub struct JsonLinesWriter<W> {
    inner: W,
}

impl<W> JsonLinesWriter<W> {
    /// Construct a new `JsonLinesWriter` from a [`std::io::Write`] instance
    pub fn new(writer: W) -> Self {
        JsonLinesWriter { inner: writer }
    }

    /// Consume the `JsonLinesWriter` and return the underlying writer
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> JsonLinesWriter<W> {
    /// Serialize & write a value to the underlying writer, followed by a
    /// newline.
    ///
    /// Note that each call may write a different type of value.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`serde_json::to_writer()`] and
    /// [`std::io::Write::write_all()`].
    pub fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde_json::to_writer(&mut self.inner, value)?;
        self.inner.write_all(b"\n")?;
        Ok(())
    }

    /// Serialize & write each item in an iterator to the underlying writer,
    /// appending a newline to each one
    ///
    /// All values in a single call to `write_all()` must be the same type, but
    /// separate calls may write different types.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`write()`][JsonLinesWriter::write].
    pub fn write_all<T, I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Serialize,
    {
        for value in items {
            self.write(&value)?;
        }
        Ok(())
    }

    /// Flush the underlying writer.
    ///
    /// Neither [`write()`][JsonLinesWriter::write] nor
    /// [`write_all()`][JsonLinesWriter::write_all] flush the writer, so you
    /// must explicitly call this method if you need output flushed.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::Write::flush()`].
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

/// A structure for reading JSON values from JSON Lines input
///
/// A `JsonLinesReader` wraps a [`std::io::BufRead`] instance and parses each
/// line as a [`serde::de::DeserializeOwned`] value in JSON.
#[derive(Debug)]
pub struct JsonLinesReader<R> {
    inner: R,
}

impl<R> JsonLinesReader<R> {
    /// Construct a new `JsonLinesReader` from a [`std::io::BufRead`] instance
    pub fn new(reader: R) -> Self {
        JsonLinesReader { inner: reader }
    }

    /// Consume the `JsonLinesReader` and return the underlying reader
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: BufRead> JsonLinesReader<R> {
    /// Read & deserialize a line from the underlying reader.
    ///
    /// If end-of-file is reached, this method returns `Ok(None)`.
    ///
    /// Note that each call may read a different type of value.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`std::io::BufRead::read_line()`] and
    /// [`serde_json::from_str()`].  Note that, in the latter case (which can
    /// be identified by the [`std::io::Error`] having a [`serde_json::Error`]
    /// value as its payload), continuing to read from the `JsonLinesReader`
    /// afterwards will pick up on the next line as though the error never
    /// happened, so invalid JSON can be easily ignored if you so wish.
    pub fn read<T>(&mut self) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let mut s = String::new();
        let r = self.inner.read_line(&mut s)?;
        if r == 0 {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_str::<T>(&s)?))
        }
    }

    /// Consume the `JsonLinesReader` and return an iterator over the
    /// deserialized values from each line.
    ///
    /// The returned iterator has an `Item` type of `std::io::Result<T>`.  Each
    /// call to `next()` has the same error conditions as
    /// [`read()`][JsonLinesReader::read].
    ///
    /// Note that all deserialized values will be of the same type.  If you
    /// wish to read lines of varying types, use the
    /// [`read()`][JsonLinesReader::read] method instead.
    pub fn iter<T>(self) -> JsonLinesIter<R, T> {
        JsonLinesIter {
            reader: self,
            _output: PhantomData,
        }
    }
}

/// An iterator over the lines of a [`BufRead`] value `R` that decodes each
/// line as JSON of type `T`.
///
/// This iterator yields items of type `Result<T, std::io::Error>`.  Errors
/// occurr under the same conditions as for [`JsonLinesReader::read()`].
#[derive(Debug)]
pub struct JsonLinesIter<R, T> {
    reader: JsonLinesReader<R>,
    _output: PhantomData<T>,
}

impl<R, T> Iterator for JsonLinesIter<R, T>
where
    T: DeserializeOwned,
    R: BufRead,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        self.reader.read().transpose()
    }
}
