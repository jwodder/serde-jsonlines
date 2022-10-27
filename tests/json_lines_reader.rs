use assert_fs::fixture::FileTouch;
use assert_fs::NamedTempFile;
use jsonlines::JsonLinesReader;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Result};
use std::path::Path;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Structure {
    name: String,
    size: i32,
    on: bool,
}

static DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");

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
    assert!(items.next().is_none())
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
