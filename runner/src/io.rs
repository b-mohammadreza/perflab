use std::{
    fs,
    io::{Read, Write},
};

pub fn read_txt_file(file_path: &String) -> Result<String, std::io::Error> {
    let mut file_text: String = String::new();

    let mut file = fs::File::open(file_path)?;
    file.read_to_string(&mut file_text)?;

    Ok(file_text)
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
