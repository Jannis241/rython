use crate::read_file;
use rython_to_ir;

pub fn run(file_name: &str) {
    let content = read_file::read_file(file_name);
}
