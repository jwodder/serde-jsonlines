//! Read & write JSON Lines documents
//!
//! JSON Lines (a.k.a. newline-delimited JSON) is a simple format for storing
//! sequences of JSON values in which each value is serialized on a single line
//! and terminated by a newline sequence.  The `serde-jsonlines` crate provides
//! functionality for reading & writing these documents (whether all at once or
//! line by line) using [`serde`]'s serialization & deserialization features.
//!
//! Basic usage involves simply importing the [`BufReadExt`] or [`WriteExt`]
//! extension trait and then using the [`json_lines()`][BufReadExt::json_lines]
//! or [`write_json_lines()`][WriteExt::write_json_lines] method on a `BufRead`
//! or `Write` value to read or write a sequence of JSON Lines values.
//! Convenience functions are also provided for the common case of reading or
//! writing a JSON Lines file given as a filepath.
//!
//! At a lower level, values can be read or written one at a time (which is
//! useful if, say, different lines are different types) by wrapping a
//! `BufRead` or `Write` value in a [`JsonLinesReader`] or [`JsonLinesWriter`]
//! and then calling the wrapped structure's [`read()`][JsonLinesReader::read]
//! or [`write()`][JsonLinesWriter::write] method, respectively.
//!
//! # Example
//!
//! ```no_run
//! use serde::{Deserialize, Serialize};
//! use serde_jsonlines::{json_lines, write_json_lines};
//! use std::io::Result;
//!
//! #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
//! pub struct Structure {
//!     pub name: String,
//!     pub size: i32,
//!     pub on: bool,
//! }
//!
//! fn main() -> Result<()> {
//!     let values = vec![
//!         Structure {
//!             name: "Foo Bar".into(),
//!             size: 42,
//!             on: true,
//!         },
//!         Structure {
//!             name: "Quux".into(),
//!             size: 23,
//!             on: false,
//!         },
//!         Structure {
//!             name: "Gnusto Cleesh".into(),
//!             size: 17,
//!             on: true,
//!         },
//!     ];
//!     write_json_lines("example.jsonl", &values)?;
//!     let values2 = json_lines("example.jsonl")?.collect::<Result<Vec<Structure>>>()?;
//!     assert_eq!(values, values2);
//!     Ok(())
//! }
//! ```

use serde::{de::DeserializeOwned, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::marker::PhantomData;
use std::path::Path;

/// A type alias for an [`Iter`] on a buffered file object.
///
/// This is the return type of [`json_lines()`].
pub type JsonLinesIter<T> = Iter<BufReader<File>, T>;

/// A structure for writing JSON values as JSON Lines.
///
/// A `JsonLinesWriter` wraps a [`std::io::Write`] instance and writes
/// [`serde::Serialize`] values to it by serializing each one as a single line
/// of JSON and appending a newline.
///
/// # Example
///
/// ```no_run
/// use serde::Serialize;
/// use serde_jsonlines::JsonLinesWriter;
/// use std::fs::{read_to_string, File};
///
/// #[derive(Serialize)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> std::io::Result<()> {
///     {
///         let fp = File::create("example.jsonl")?;
///         let mut writer = JsonLinesWriter::new(fp);
///         writer.write_all([
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
///         ])?;
///         writer.flush()?;
///     }
///     // End the block to close the writer
///     assert_eq!(
///         read_to_string("example.jsonl")?,
///         concat!(
///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
///             "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
///         )
///     );
///     Ok(())
/// }
/// ```
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
    /// Serialize a value as a line of JSON and write it to the underlying
    /// writer, followed by a newline.
    ///
    /// Note that separate calls to this method may write different types of
    /// values.
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

    /// Serialize each item in an iterator as a line of JSON, and write out
    /// each one followed by a newline to the underlying writer.
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

/// A structure for reading JSON values from JSON Lines input.
///
/// A `JsonLinesReader` wraps a [`std::io::BufRead`] instance and parses each
/// line as a [`serde::de::DeserializeOwned`] value in JSON.
///
/// # Example
///
/// ```no_run
/// use serde::Deserialize;
/// use serde_jsonlines::JsonLinesReader;
/// use std::fs::{write, File};
/// use std::io::BufReader;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> std::io::Result<()> {
///     write(
///         "example.jsonl",
///         concat!(
///             "{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n",
///             "{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n",
///             " {\"name\": \"Gnusto Cleesh\" , \"on\": true, \"size\": 17}\n",
///         ),
///     )?;
///     let fp = BufReader::new(File::open("example.jsonl")?);
///     let reader = JsonLinesReader::new(fp);
///     let items = reader
///         .iter::<Structure>()
///         .collect::<std::io::Result<Vec<_>>>()?;
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
    /// Read & deserialize a line of JSON from the underlying reader.
    ///
    /// If end-of-file is reached, this method returns `Ok(None)`.
    ///
    /// Note that separate calls to this method may read different types of
    /// values.
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
    /// deserialized JSON values from each line.
    ///
    /// The returned iterator has an `Item` type of `std::io::Result<T>`.  Each
    /// call to `next()` has the same error conditions as
    /// [`read()`][JsonLinesReader::read].
    ///
    /// Note that all deserialized values will be of the same type.  If you
    /// wish to read lines of varying types, use the
    /// [`read()`][JsonLinesReader::read] method instead.
    pub fn iter<T>(self) -> Iter<R, T> {
        Iter {
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
///
/// Iterators of this type are returned by [`JsonLinesReader::iter()`],
/// [`BufReadExt::json_lines()`], and [`json_lines()`].
#[derive(Debug)]
pub struct Iter<R, T> {
    reader: JsonLinesReader<R>,
    _output: PhantomData<T>,
}

impl<R, T> Iterator for Iter<R, T>
where
    T: DeserializeOwned,
    R: BufRead,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        self.reader.read().transpose()
    }
}

/// An extension trait for the [`std::io::Write`] trait that adds a
/// `write_json_lines()` method
///
/// # Example
///
/// ```no_run
/// use serde::Serialize;
/// use serde_jsonlines::WriteExt;
/// use std::fs::{read_to_string, File};
/// use std::io::Write;
///
/// #[derive(Serialize)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> std::io::Result<()> {
///     {
///         let mut fp = File::create("example.jsonl")?;
///         fp.write_json_lines([
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
///         ])?;
///         fp.flush()?;
///     }
///     // End the block to close the writer
///     assert_eq!(
///         read_to_string("example.jsonl")?,
///         concat!(
///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
///             "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
///         )
///     );
///     Ok(())
/// }
/// ```
pub trait WriteExt: Write {
    /// Serialize each item in an iterator as a line of JSON, and write out
    /// each one followed by a newline.
    ///
    /// All values in a single call to `write_json_lines()` must be the same
    /// type, but separate calls may write different types.
    ///
    /// This method does not flush.
    ///
    /// # Errors
    ///
    /// Has the same error conditions as [`serde_json::to_writer()`] and
    /// [`std::io::Write::write_all()`].
    fn write_json_lines<T, I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Serialize,
    {
        for value in items {
            serde_json::to_writer(&mut *self, &value)?;
            self.write_all(b"\n")?;
        }
        Ok(())
    }
}

impl<W: Write> WriteExt for W {}

/// An extension trait for the [`std::io::BufRead`] trait that adds a
/// `json_lines()` method
///
/// # Example
///
/// ```no_run
/// use serde::Deserialize;
/// use serde_jsonlines::BufReadExt;
/// use std::fs::{write, File};
/// use std::io::{BufReader, Result};
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> Result<()> {
///     write(
///         "example.jsonl",
///         concat!(
///             "{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n",
///             "{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n",
///             " {\"name\": \"Gnusto Cleesh\" , \"on\": true, \"size\": 17}\n",
///         ),
///     )?;
///     let fp = BufReader::new(File::open("example.jsonl")?);
///     let items = fp.json_lines::<Structure>().collect::<Result<Vec<_>>>()?;
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
pub trait BufReadExt: BufRead {
    /// Consume the reader and return an iterator over the deserialized JSON
    /// values from each line.
    ///
    /// The returned iterator has an `Item` type of `std::io::Result<T>`.  Each
    /// call to `next()` has the same error conditions as
    /// [`JsonLinesReader::read()`].
    ///
    /// Note that all deserialized values will be of the same type.
    fn json_lines<T>(self) -> Iter<Self, T>
    where
        Self: Sized,
    {
        JsonLinesReader::new(self).iter()
    }
}

impl<R: BufRead> BufReadExt for R {}

/// Write an iterator of values to the file at `path` as JSON Lines.
///
/// If the file does not already exist, it is created.  If it does exist, any
/// contents are discarded.
///
/// # Errors
///
/// Has the same error conditions as [`File::create()`],
/// [`serde_json::to_writer()`], [`std::io::Write::write_all()`], and
/// [`std::io::Write::flush()`].
///
/// # Example
///
/// ```no_run
/// use serde::Serialize;
/// use serde_jsonlines::write_json_lines;
/// use std::fs::read_to_string;
///
/// #[derive(Serialize)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> std::io::Result<()> {
///     write_json_lines(
///         "example.jsonl",
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
///         ],
///     )?;
///     assert_eq!(
///         read_to_string("example.jsonl")?,
///         concat!(
///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
///             "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
///         )
///     );
///     Ok(())
/// }
/// ```
pub fn write_json_lines<P, I, T>(path: P, items: I) -> Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = T>,
    T: Serialize,
{
    let mut fp = BufWriter::new(File::create(path)?);
    fp.write_json_lines(items)?;
    fp.flush()
}

/// Append an iterator of values to the file at `path` as JSON Lines.
///
/// If the file does not already exist, it is created.  If it does exist, the
/// new lines are added after any lines that are already present.
///
/// # Errors
///
/// Has the same error conditions as [`File::create()`],
/// [`serde_json::to_writer()`], [`std::io::Write::write_all()`], and
/// [`std::io::Write::flush()`].
///
/// # Example
///
/// ```no_run
/// use serde::Serialize;
/// use serde_jsonlines::append_json_lines;
/// use std::fs::read_to_string;
///
/// #[derive(Serialize)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> std::io::Result<()> {
///     append_json_lines(
///         "example.jsonl",
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
///         ],
///     )?;
///     assert_eq!(
///         read_to_string("example.jsonl")?,
///         concat!(
///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
///         )
///     );
///     append_json_lines(
///         "example.jsonl",
///         [
///             Structure {
///                 name: "Gnusto Cleesh".into(),
///                 size: 17,
///                 on: true,
///             },
///             Structure {
///                 name: "baz".into(),
///                 size: 69105,
///                 on: false,
///             },
///         ],
///     )?;
///     assert_eq!(
///         read_to_string("example.jsonl")?,
///         concat!(
///             "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
///             "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
///             "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
///             "{\"name\":\"baz\",\"size\":69105,\"on\":false}\n",
///         )
///     );
///     Ok(())
/// }
/// ```
pub fn append_json_lines<P, I, T>(path: P, items: I) -> Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = T>,
    T: Serialize,
{
    let mut fp = BufWriter::new(OpenOptions::new().append(true).create(true).open(path)?);
    fp.write_json_lines(items)?;
    fp.flush()
}

/// Iterate over JSON Lines values from a file.
///
/// `json_lines(path)` returns an iterator of values deserialized from the JSON
/// Lines in the file at `path`.
///
/// The returned iterator has an `Item` type of `std::io::Result<T>`.  Each
/// call to `next()` has the same error conditions as
/// [`JsonLinesReader::read()`].
///
/// # Errors
///
/// Has the same error conditions as [`File::open()`].
///
/// # Example
///
/// ```no_run
/// use serde::Deserialize;
/// use serde_jsonlines::json_lines;
/// use std::fs::write;
/// use std::io::Result;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// pub struct Structure {
///     pub name: String,
///     pub size: i32,
///     pub on: bool,
/// }
///
/// fn main() -> Result<()> {
///     write(
///         "example.jsonl",
///         concat!(
///             "{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n",
///             "{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n",
///             " {\"name\": \"Gnusto Cleesh\" , \"on\": true, \"size\": 17}\n",
///         ),
///     )?;
///     let items = json_lines::<Structure, _>("example.jsonl")?.collect::<Result<Vec<_>>>()?;
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

pub fn json_lines<T, P: AsRef<Path>>(path: P) -> Result<JsonLinesIter<T>> {
    let fp = BufReader::new(File::open(path)?);
    Ok(fp.json_lines())
}
