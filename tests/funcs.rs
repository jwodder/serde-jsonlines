use assert_fs::assert::PathAssert;
use assert_fs::NamedTempFile;
use jsonlines::{append_json_lines, write_json_lines};
mod common;
use common::*;

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
