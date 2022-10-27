use serde::Serialize;
use std::io::{Result, Write};

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
