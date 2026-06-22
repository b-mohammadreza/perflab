use crate::paths;
use std::{
    fs,
    io::{Read, Write},
};

pub fn read_perf_file(timestamp: &String, rep: u32) -> String {
    let perf_stat_path = paths::get_perf_stat_path(timestamp, rep);
    let mut csv_file_text: String = String::new();

    fs::File::open(perf_stat_path.as_str())
        .unwrap_or_else(|err| {
            panic!("perflab-Cannot open file({perf_stat_path}), error:\n{err}");
        })
        .read_to_string(&mut csv_file_text)
        .unwrap_or_else(|err| {
            panic!("perflab-Failed reading file({perf_stat_path}), error:\n{err}");
        });

    csv_file_text
}

pub fn write_result(timestamp: &String, result_schem_pretty: &String) {
    let file_path = paths::get_result_json_path(timestamp);
    let mut json_file = fs::File::create(&file_path).unwrap_or_else(|err| {
        panic!(
            "perflab-Failed to create file({}), error:\n{err}",
            &file_path
        );
    });

    json_file
        .write_all(result_schem_pretty.as_bytes())
        .unwrap_or_else(|err| {
            panic!("perflab-Failed to write json file, error:\n{err}");
        });
}
