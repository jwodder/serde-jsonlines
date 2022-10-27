use assert_fs::assert::PathAssert;
use assert_fs::NamedTempFile;
use jsonlines::write_json_lines;
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
