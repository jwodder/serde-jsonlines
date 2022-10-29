use assert_fs::fixture::{FileTouch, FileWriteStr};
use assert_fs::NamedTempFile;
use serde_jsonlines::JsonLinesReader;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Result, Seek, SeekFrom, Write};
use std::path::Path;
mod common;
use common::*;

#[test]
fn test_read_empty() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    tmpfile.touch().unwrap();
    let fp = BufReader::new(File::open(&tmpfile).unwrap());
    let mut reader = JsonLinesReader::new(fp);
    assert_eq!(reader.read::<Structure>().unwrap(), None);
    assert_eq!(reader.read::<Structure>().unwrap(), None);
    assert_eq!(reader.read::<Structure>().unwrap(), None);
}

#[test]
fn test_read_one() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample01.jsonl")).unwrap());
    let mut reader = JsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
}

#[test]
fn test_iter() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample01.jsonl")).unwrap());
    let reader = JsonLinesReader::new(fp);
    let mut items = reader.iter::<Structure>();
    assert_eq!(
        items.next().unwrap().unwrap(),
        Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        }
    );
    assert_eq!(
        items.next().unwrap().unwrap(),
        Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        }
    );
    assert_eq!(
        items.next().unwrap().unwrap(),
        Structure {
            name: "Gnusto Cleesh".into(),
            size: 17,
            on: true,
        }
    );
    assert!(items.next().is_none());
}

#[test]
fn test_iter_collect() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample01.jsonl")).unwrap());
    let reader = JsonLinesReader::new(fp);
    let items = reader
        .iter::<Structure>()
        .collect::<Result<Vec<_>>>()
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

#[test]
fn test_read_one_then_read_inner() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample02.txt")).unwrap());
    let mut reader = JsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    let mut fp: BufReader<File> = reader.into_inner();
    let mut s = String::new();
    fp.read_line(&mut s).unwrap();
    assert_eq!(s, "Not JSON.\n");
}

#[test]
fn test_read_two() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample03.jsonl")).unwrap());
    let mut reader = JsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    assert_eq!(
        reader.read::<Point>().unwrap(),
        Some(Point { x: 69, y: 105 })
    );
}

#[test]
fn test_iter_invalid_json() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample04.txt")).unwrap());
    let reader = JsonLinesReader::new(fp);
    let mut items = reader.iter::<Structure>();
    assert_eq!(
        items.next().unwrap().unwrap(),
        Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        }
    );

    let e = items.next().unwrap().unwrap_err();
    assert_eq!(e.kind(), ErrorKind::UnexpectedEof);
    assert!(e
        .into_inner()
        .unwrap()
        .downcast::<serde_json::Error>()
        .is_ok());

    assert_eq!(
        items.next().unwrap().unwrap(),
        Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        }
    );

    let e = items.next().unwrap().unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidData);
    assert!(e
        .into_inner()
        .unwrap()
        .downcast::<serde_json::Error>()
        .is_ok());

    let e = items.next().unwrap().unwrap_err();
    assert_eq!(e.kind(), ErrorKind::InvalidData);
    assert!(e
        .into_inner()
        .unwrap()
        .downcast::<serde_json::Error>()
        .is_ok());

    assert_eq!(
        items.next().unwrap().unwrap(),
        Structure {
            name: "Gnusto Cleesh".into(),
            size: 17,
            on: true,
        }
    );
    assert!(items.next().is_none());
}

#[test]
fn test_read_then_write_then_read() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    tmpfile
        .write_str("{\"name\": \"Foo Bar\", \"on\":true,\"size\": 42 }\n")
        .unwrap();
    let fp = BufReader::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&tmpfile)
            .unwrap(),
    );
    let mut reader = JsonLinesReader::new(fp);
    assert_eq!(
        reader.read::<Structure>().unwrap(),
        Some(Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        })
    );
    assert_eq!(reader.read::<Structure>().unwrap(), None);
    let fp: &mut File = reader.get_mut().get_mut();
    let pos = fp.stream_position().unwrap();
    fp.write_all(b"{ \"name\":\"Quux\", \"on\" : false ,\"size\": 23}\n")
        .unwrap();
    fp.flush().unwrap();
    fp.seek(SeekFrom::Start(pos)).unwrap();
    assert_eq!(
        reader.read::<Structure>().unwrap(),
        Some(Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        })
    );
    assert_eq!(reader.read::<Structure>().unwrap(), None);
}
