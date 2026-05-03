use std::{fs, ops::Add};

pub fn read_file(file_name: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(file_name)
}
