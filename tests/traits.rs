mod common;
use crate::common::*;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::FileTouch;
use assert_fs::NamedTempFile;
use serde_jsonlines::{BufReadExt, WriteExt};
use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::empty;
use std::path::Path;

#[test]
fn test_write_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let mut fp = File::create(&tmpfile).unwrap();
        fp.write_json_lines([
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
        ])
        .unwrap();
        fp.flush().unwrap();
    }
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
        "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
    ));
}

#[test]
fn test_no_write_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let mut fp = File::create(&tmpfile).unwrap();
        fp.write_json_lines(empty::<Structure>()).unwrap();
        fp.flush().unwrap();
    }
    tmpfile.assert("");
}

#[test]
fn test_json_lines() {
    let fp = BufReader::new(File::open(Path::new(DATA_DIR).join("sample01.jsonl")).unwrap());
    let mut items = fp.json_lines::<Structure>();
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
fn test_no_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    tmpfile.touch().unwrap();
    let fp = BufReader::new(File::open(&tmpfile).unwrap());
    let mut items = fp.json_lines::<Structure>();
    assert!(items.next().is_none());
    assert!(items.next().is_none());
    assert!(items.next().is_none());
}
