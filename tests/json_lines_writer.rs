mod common;
use crate::common::*;
use assert_fs::NamedTempFile;
use assert_fs::assert::PathAssert;
use serde_jsonlines::JsonLinesWriter;
use std::fs::File;
use std::io::{Seek, Write};
use std::iter::empty;

#[test]
fn test_write_one() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .unwrap();
        writer.flush().unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n");
}

#[test]
fn test_write_two() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .unwrap();
        writer.write(&Point { x: 69, y: 105 }).unwrap();
        writer.flush().unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n{\"x\":69,\"y\":105}\n");
}

#[test]
fn test_write_all() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer
            .write_all([
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
        writer.flush().unwrap();
    }
    tmpfile.assert(concat!(
        "{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n",
        "{\"name\":\"Quux\",\"size\":23,\"on\":false}\n",
        "{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n",
    ));
}

#[test]
fn test_write_one_then_write_inner() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .unwrap();
        writer.flush().unwrap();
        let mut fp: File = writer.into_inner();
        fp.write_all(b"Not JSON\n").unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\nNot JSON\n");
}

#[test]
fn test_write_all_none() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer.write_all(empty::<Structure>()).unwrap();
        writer.flush().unwrap();
    }
    tmpfile.assert("");
}

#[test]
fn test_write_then_back_up_then_write() {
    let tmpfile = NamedTempFile::new("test.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .unwrap();
        writer.flush().unwrap();
        let fp: &mut File = writer.get_mut();
        fp.rewind().unwrap();
        writer
            .write(&Structure {
                name: "Gnusto Cleesh".into(),
                size: 17,
                on: true,
            })
            .unwrap();
        writer.flush().unwrap();
    }
    tmpfile.assert("{\"name\":\"Gnusto Cleesh\",\"size\":17,\"on\":true}\n");
}
