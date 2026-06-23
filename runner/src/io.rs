use std::{
    fs,
    io::{Read, Write},
};

pub fn read_txt_file(file_path: &String) -> String {
    let mut file_text: String = String::new();

    fs::File::open(file_path)
        .unwrap_or_else(|err| {
            panic!("perflab-Cannot open file({file_path}), error:\n{err}");
        })
        .read_to_string(&mut file_text)
        .unwrap_or_else(|err| {
            panic!("perflab-Failed reading file({file_path}), error:\n{err}");
        });

    file_text
}

pub fn write_result(file_path: &String, txt_data: &String) {
    let mut json_file = fs::File::create(file_path).unwrap_or_else(|err| {
        panic!("perflab-Failed to create file({file_path}), error:\n{err}");
    });

    json_file
        .write_all(txt_data.as_bytes())
        .unwrap_or_else(|err| {
            panic!("perflab-Failed to write json file, error:\n{err}");
        });
}
