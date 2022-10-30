#![cfg(feature = "async")]
use assert_fs::assert::PathAssert;
use assert_fs::NamedTempFile;
use serde_jsonlines::AsyncJsonLinesWriter;
use std::io::SeekFrom;
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
mod common;
use common::*;

#[tokio::test]
async fn test_write_one() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).await.unwrap();
        let writer = AsyncJsonLinesWriter::new(fp);
        tokio::pin!(writer);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n");
}

#[tokio::test]
async fn test_write_two() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).await.unwrap();
        let writer = AsyncJsonLinesWriter::new(fp);
        tokio::pin!(writer);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .await
            .unwrap();
        writer.write(&Point { x: 69, y: 105 }).await.unwrap();
        writer.flush().await.unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n{\"x\":69,\"y\":105}\n");
}

#[tokio::test]
async fn test_write_one_then_write_inner() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).await.unwrap();
        let mut writer = Pin::new(Box::new(AsyncJsonLinesWriter::new(fp)));
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();
        let mut fp: File = Pin::into_inner(writer).into_inner();
        fp.write_all(b"Not JSON\n").await.unwrap();
        fp.flush().await.unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\nNot JSON\n");
}

#[tokio::test]
async fn test_write_one_then_write_pin_mut() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).await.unwrap();
        let writer = AsyncJsonLinesWriter::new(fp);
        tokio::pin!(writer);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();
        let mut fp: Pin<&mut File> = writer.get_pin_mut();
        fp.write_all(b"Not JSON\n").await.unwrap();
        fp.flush().await.unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\nNot JSON\n");
}

#[tokio::test]
async fn test_write_then_back_up_then_write() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).await.unwrap();
        let mut writer = Pin::new(Box::new(AsyncJsonLinesWriter::new(fp)));
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();
        let fp: &mut File = writer.get_mut();
        fp.seek(SeekFrom::Start(0)).await.unwrap();
        writer
            .write(&Structure {
                name: "Gnusto Cleesh".into(),
                size: 17,
                on: true,
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();
    }
    tmpfile.assert("{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n");
}
