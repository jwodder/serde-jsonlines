#![cfg(feature = "async")]
use assert_fs::assert::PathAssert;
use assert_fs::fixture::FileTouch;
use assert_fs::NamedTempFile;
use futures::sink::SinkExt;
use futures::stream::empty;
use serde_jsonlines::{AsyncBufReadJsonLines, AsyncWriteJsonLines};
use std::path::Path;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio_stream::StreamExt;

mod common;
use common::*;

#[tokio::test]
async fn test_json_lines() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample01.jsonl"))
            .await
            .unwrap(),
    );
    let items = fp.json_lines::<Structure>();
    tokio::pin!(items);
    assert_eq!(
        items.next().await.unwrap().unwrap(),
        Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        }
    );
    assert_eq!(
        items.next().await.unwrap().unwrap(),
        Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        }
    );
    assert_eq!(
        items.next().await.unwrap().unwrap(),
        Structure {
            name: "Gnusto Cleesh".into(),
            size: 17,
            on: true,
        }
    );
    assert!(items.next().await.is_none());
}

#[tokio::test]
async fn test_no_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    tmpfile.touch().unwrap();
    let fp = BufReader::new(File::open(&tmpfile).await.unwrap());
    let items = fp.json_lines::<Structure>();
    tokio::pin!(items);
    assert!(items.next().await.is_none());
    assert!(items.next().await.is_none());
    assert!(items.next().await.is_none());
}

#[tokio::test]
async fn test_into_json_lines_sink() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let sink = File::create(&tmpfile).await.unwrap().into_json_lines_sink();
        tokio::pin!(sink);
        for item in [
            Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            },
            Structure {
                name: "Quux".into(),
                size: 23,
                on: false,
            },
            Structure {
                name: "Gnusto Cleesh".into(),
                size: 17,
                on: true,
            },
        ] {
            sink.send(item).await.unwrap()
        }
        sink.close().await.unwrap();
    }
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
        "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
    ));
}

#[tokio::test]
async fn test_no_write_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let sink = File::create(&tmpfile).await.unwrap().into_json_lines_sink();
        tokio::pin!(sink);
        let stream = empty::<std::io::Result<Structure>>();
        tokio::pin!(stream);
        sink.send_all(&mut stream).await.unwrap();
        sink.close().await.unwrap();
    }
    tmpfile.assert("");
}
