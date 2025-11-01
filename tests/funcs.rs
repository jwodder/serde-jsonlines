mod common;
use crate::common::*;
use assert_fs::NamedTempFile;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::FileTouch;
use serde_jsonlines::{append_json_lines, json_lines, write_json_lines};
use std::iter::empty;
use std::path::Path;

#[test]
fn test_write_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    write_json_lines(
        &tmpfile,
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
        ],
    )
    .unwrap();
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
        "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
    ));
}

#[test]
fn test_write_no_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    write_json_lines(&tmpfile, empty::<Structure>()).unwrap();
    tmpfile.assert("");
}

#[test]
fn test_append_json_lines() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    append_json_lines(
        &tmpfile,
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
        ],
    )
    .unwrap();
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
    ));
    append_json_lines(
        &tmpfile,
        [
            Structure {
                name: "Gnusto Cleesh".into(),
                size: 17,
                on: true,
            },
            Structure {
                name: "baz".into(),
                size: 69105,
                on: false,
            },
        ],
    )
    .unwrap();
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
        "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
        "{\"name\":\"baz\",\"size\":69105,\"on\":false}\n",
    ));
}

#[test]
fn test_append_json_lines_some_then_none() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    append_json_lines(
        &tmpfile,
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
        ],
    )
    .unwrap();
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
    ));
    append_json_lines(&tmpfile, empty::<Structure>()).unwrap();
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
    ));
}

#[test]
fn test_append_json_lines_none_then_some() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    append_json_lines(&tmpfile, empty::<Structure>()).unwrap();
    tmpfile.assert("");
    append_json_lines(
        &tmpfile,
        [
            Structure {
                name: "Gnusto Cleesh".into(),
                size: 17,
                on: true,
            },
            Structure {
                name: "baz".into(),
                size: 69105,
                on: false,
            },
        ],
    )
    .unwrap();
    tmpfile.assert(concat!(
        "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
        "{\"name\":\"baz\",\"size\":69105,\"on\":false}\n",
    ));
}

#[test]
fn test_json_lines() {
    let path = Path::new(DATA_DIR).join("sample01.jsonl");
    let mut items = json_lines::<Structure, _>(path).unwrap();
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
    let mut items = json_lines::<Structure, _>(tmpfile).unwrap();
    assert!(items.next().is_none());
    assert!(items.next().is_none());
    assert!(items.next().is_none());
}
