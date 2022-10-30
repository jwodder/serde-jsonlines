#![cfg(feature = "async")]
use assert_fs::fixture::FileTouch;
use assert_fs::NamedTempFile;
use serde_jsonlines::AsyncBufReadJsonLines;
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
