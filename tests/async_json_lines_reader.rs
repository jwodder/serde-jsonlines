#![cfg(feature = "async")]
mod common;
use crate::common::*;
use assert_fs::NamedTempFile;
use assert_fs::fixture::{FileTouch, FileWriteStr};
use futures_util::{StreamExt, TryStreamExt};
use serde_jsonlines::AsyncJsonLinesReader;
use std::io::{ErrorKind, SeekFrom};
use std::path::Path;
use std::pin::Pin;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn test_read_empty() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    tmpfile.touch().unwrap();
    let fp = BufReader::new(File::open(&tmpfile).await.unwrap());
    let mut reader = AsyncJsonLinesReader::new(fp);
    assert_eq!(reader.read::<Structure>().await.unwrap(), None);
    assert_eq!(reader.read::<Structure>().await.unwrap(), None);
    assert_eq!(reader.read::<Structure>().await.unwrap(), None);
}

#[tokio::test]
async fn test_read_one() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample01.jsonl"))
            .await
            .unwrap(),
    );
    let mut reader = AsyncJsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().await.unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
}

#[tokio::test]
async fn test_read_one_then_read_inner() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample02.txt"))
            .await
            .unwrap(),
    );
    let mut reader = AsyncJsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().await.unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    let mut fp: BufReader<File> = reader.into_inner();
    let mut s = String::new();
    fp.read_line(&mut s).await.unwrap();
    assert_eq!(s, "Not JSON.\n");
}

#[tokio::test]
async fn test_read_two() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample03.jsonl"))
            .await
            .unwrap(),
    );
    let mut reader = AsyncJsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().await.unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    assert_eq!(
        reader.read::<Point>().await.unwrap(),
        Some(Point { x: 69, y: 105 })
    );
}

#[tokio::test]
async fn test_read_then_write_then_read() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    tmpfile
        .write_str("{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n")
        .unwrap();
    let fp = BufReader::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&tmpfile)
            .await
            .unwrap(),
    );
    let mut reader = AsyncJsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().await.unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    assert_eq!(reader.read::<Structure>().await.unwrap(), None);
    let fp: &mut File = reader.get_mut().get_mut();
    let pos = fp.stream_position().await.unwrap();
    fp.write_all(b"{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n")
        .await
        .unwrap();
    fp.flush().await.unwrap();
    fp.seek(SeekFrom::Start(pos)).await.unwrap();
    assert_eq!(
        reader.read::<Structure>().await.unwrap(),
        Some(Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        })
    );
    assert_eq!(reader.read::<Structure>().await.unwrap(), None);
}

#[tokio::test]
async fn test_read_one_then_read_pin_mut() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample02.txt"))
            .await
            .unwrap(),
    );
    let mut reader = AsyncJsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().await.unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    tokio::pin!(reader);
    let mut fp: Pin<&mut BufReader<File>> = reader.get_pin_mut();
    let mut s = String::new();
    fp.read_line(&mut s).await.unwrap();
    assert_eq!(s, "Not JSON.\n");
}

#[tokio::test]
async fn test_read_all() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample01.jsonl"))
            .await
            .unwrap(),
    );
    let reader = AsyncJsonLinesReader::new(fp);
    let mut items = reader.read_all::<Structure>();
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
async fn test_read_all_collect() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample01.jsonl"))
            .await
            .unwrap(),
    );
    let reader = AsyncJsonLinesReader::new(fp);
    let items = reader
        .read_all::<Structure>()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(
        items,
        [
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
        ]
    );
}

#[tokio::test]
async fn test_read_all_invalid_json() {
    let fp = BufReader::new(
        File::open(Path::new(DATA_DIR).join("sample04.txt"))
            .await
            .unwrap(),
    );
    let reader = AsyncJsonLinesReader::new(fp);
    let mut items = reader.read_all::<Structure>();
    assert_eq!(
        items.next().await.unwrap().unwrap(),
        Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        }
    );

    let e = items.next().await.unwrap().unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnexpectedEof);
    assert!(e.get_ref().unwrap().is::<serde_json::Error>());

    assert_eq!(
        items.next().await.unwrap().unwrap(),
        Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        }
    );

    let e = items.next().await.unwrap().unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidData);
    assert!(e.get_ref().unwrap().is::<serde_json::Error>());

    let e = items.next().await.unwrap().unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidData);
    assert!(e.get_ref().unwrap().is::<serde_json::Error>());

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
