use serde::{de::DeserializeOwned, Serialize};
use std::io::{BufRead, Result, Write};
use std::iter::FusedIterator;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct JsonLinesWriter<W> {
    inner: W,
}

impl<W> JsonLinesWriter<W> {
    pub fn new(writer: W) -> Self {
        JsonLinesWriter { inner: writer }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> JsonLinesWriter<W> {
    pub fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde_json::to_writer(&mut self.inner, value)?;
        self.inner.write_all(b"\n")?;
        Ok(())
    }

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

    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug)]
pub struct JsonLinesReader<R> {
    inner: R,
}

impl<R> JsonLinesReader<R> {
    pub fn new(reader: R) -> Self {
        JsonLinesReader { inner: reader }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: BufRead> JsonLinesReader<R> {
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

    pub fn iter<T>(self) -> Iter<R, T> {
        Iter {
            reader: self,
            _output: PhantomData,
        }
    }
}

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

impl<R, T> FusedIterator for Iter<R, T>
where
    T: DeserializeOwned,
    R: BufRead,
{
}
