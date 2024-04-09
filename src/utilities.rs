use std::fs;
use std::ffi;
use std::str::FromStr;

pub(crate) const SOURCE_DIR: &'static str = "file-source";
pub(crate) const STATUS_DIR: &'static str = "file-source/file-status";
pub(crate) const SIMULATION_DIR: &'static str = "src/simulation";

pub fn get_ready_files() -> Result<Vec<ffi::OsString>, std::io::Error> {
    let files = walk_directory(STATUS_DIR, &|x: &str| x.ends_with(".ready"))?;


    Ok(files)
}

pub fn get_done_files() -> Result<Vec<ffi::OsString>, std::io::Error>  {
    let files = walk_directory(SOURCE_DIR, &|x: &str| x.ends_with(".done"))?;

    Ok(files)
}

pub fn walk_directory(path: &str, fn_filter: impl Fn(&str) -> bool) -> Result<Vec<ffi::OsString>, std::io::Error> {
    let entries = fs::read_dir(&path)?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            let file_name = entry.file_name();
            if fn_filter(file_name.to_str().unwrap()) {
                let file_name = entry.path().into_os_string(); 
                files.push(file_name);
            }
        }
    }

    Ok(files)
}