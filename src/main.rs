use std::{ffi, fs};
use std::collections::HashMap;
use std::thread;

mod utilities;
mod services;
mod wal;

fn main() {
    let ready_files = utilities::get_ready_files().unwrap();
    let done_files = utilities::get_done_files().unwrap();

    println!("{:?}", ready_files);
    println!("{:?}", done_files);
}
