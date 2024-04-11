use std::fs;
use std::fs::DirEntry;

pub(crate) const SOURCE_DIR: &'static str = "file-source";
pub(crate) const STATUS_DIR: &'static str = "file-source/file-status";
pub(crate) const SIMULATION_DIR: &'static str = "src/simulation";

#[derive(Debug)]
pub struct FileEntry {
    /// file's name without the extension string.
    pub file_name: String,

    /// The part after the first "."
    pub file_extension: String,

    /// The full path of the file.
    pub full_path: String,
}

impl FileEntry {
    pub fn new(entry: &DirEntry) -> Self {
        let full_path = entry.path().into_os_string().into_string().unwrap();
        let (_, extension) = full_path.split_once(".").unwrap();
        let file_name: &str = full_path.split("/").last().unwrap();
        let file_name = file_name.split(".").take(1).collect::<String>();
        FileEntry {
            file_name,
            file_extension: String::from(extension),
            full_path
        }
    }
}

pub fn get_ready_files() -> Result<Vec<FileEntry>, std::io::Error> {
    let files = walk_directory(STATUS_DIR, &|x: &str| x.ends_with(".ready"))?;

    Ok(files)
}

pub fn get_done_files() -> Result<Vec<FileEntry>, std::io::Error>  {
    let files = walk_directory(SOURCE_DIR, &|x: &str| x.ends_with(".done"))?;

    Ok(files)
}

pub fn walk_directory(path: &str, fn_filter: impl Fn(&str) -> bool) -> Result<Vec<FileEntry>, std::io::Error> {
    let entries = fs::read_dir(&path)?;

    let mut files: Vec<FileEntry> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            let file_name = entry.file_name();
            if fn_filter(file_name.to_str().unwrap()) {
                files.push(FileEntry::new(&entry));
            }
        }
    }

    Ok(files)
}