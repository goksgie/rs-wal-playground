use std::{fs::OpenOptions, io::{self, Read, Write}};
use serde_json;
use serde::{Serialize, Deserialize};

use crate::utilities;

#[derive(Serialize, Deserialize)]
pub enum WalAction {
    /// Signifies the number of times that uploading this file will fail.
    /// When it is 0, it is expected to be succeeded.
    Fail { count: u8 },

    /// Upload is successful.
    Success,
}

/// Represents the WAL file format.
#[derive(Serialize, Deserialize)]
pub struct WalFile {
    /// The type of action to be performed by the processor.
    pub(crate) action: WalAction,

    /// The duration where each action will take. The unit is
    /// milliseconds.
    pub(crate) duration: u64, 

    /// The file name to be stored to take action on it.
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) file_name: String,
}

impl WalFile {
    /// Reads the provided WAL file and constructs the WAL file format.
    pub fn read(f_name: &str) -> Self {
        // need to make this work with bytes...
        let mut file_contents = String::new();

        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .open(f_name)
            .expect("File does not exist");
        let read_bytes = f.read_to_string(&mut file_contents).expect("Reading from file did not end up as expected");
        println!("File size was: {}", read_bytes); 

        // let mut buffer = vec![0; 1024];
        // let (zeros, content) = buffer.split_at(1024-read_bytes);
        // println!("content: {:?}", content);

        // file_contents = String::from_utf8(buffer).expect("Failed to convert the read bytes into string");
        let mut wal_file: WalFile = serde_json::from_str(&file_contents).expect("The WAL has incorrect formatting");
        wal_file.file_name = f_name.to_string();
        wal_file
    }

    /// When WAL file is simulating a failure case, it would include
    /// the number of attempts it would fail. When the count reaches 0,
    /// it would alter the action to become "success".
    /// If the action is already "Success", then this is a no-op.
    pub fn decrement_failure_count(&mut self) -> std::io::Result<()> {
        match self.action {
            WalAction::Fail { count } => {
                if count <= 1 {
                    self.action = WalAction::Success;
                } else {
                    self.action = WalAction::Fail { count: count - 1 };
                }
            },

            _ => {},
        }

        Ok(())
    }
    
    /// Each character of a WAL file name is in 16 Base, thus can reach "f".
    /// given the number, construct the WAL file name and generate a WalFile object.
    pub fn generate_wal_file(num: u64, action: WalAction, work_duration: u64) -> WalFile {
        let hex_name = format!("{:X}", num);
        WalFile {
            action: action,
            duration: work_duration,
            file_name: format!("{}/00000001{:0>16}.ready", utilities::STATUS_DIR, hex_name)
        }
    }

    pub fn flush_to_file(&self) -> io::Result<()> {
       if self.file_name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "file name is empty")); 
       } 

       let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_name)
            .expect(&format!("could not open/create a WAL file with name: {:?}", self.file_name));
       let buffer = serde_json::to_string(&self)
            .expect("failed to deserialize the WalFile");
       f.write_all(buffer.as_bytes())
    }

    /// Renames the .ready WAL file as .done
    pub fn mark_done(&self) -> io::Result<()> {
        let file_name = &self.file_name;
        let file_name = file_name.split('.').take(1).collect::<String>();
        let done_file_name = format!("{}.done", file_name);
        
        std::fs::rename(&self.file_name, &done_file_name)
    }

    /// Generates a new .done WAL file under file-source folder.
    pub fn generate_done_file(&self) -> io::Result<()> {
        let file_name = &self.file_name;
        let file_name = file_name.split('.').take(1).collect::<String>();
        let file_name = file_name.split('/').last().unwrap();
        let done_file_name = format!("{}/{}.done", utilities::SOURCE_DIR, file_name);
        
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(done_file_name).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_ignore_file_name() {
        let x = WalFile { action: WalAction::Success, duration: 10, file_name: "test".to_string() };
        let y: WalFile = serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
        assert!(y.file_name.is_empty());

        let x = WalFile { action: WalAction::Fail { count: 100 }, duration: 10, file_name: "test".to_string() };
        let y: WalFile = serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
        assert!(y.file_name.is_empty());
    }

    #[test]
    fn serialization_format() {
        let x = WalFile { action: WalAction::Success, duration: 10, file_name: "test".to_string() };
        assert_eq!("{\"action\":\"Success\",\"duration\":10}", serde_json::to_string(&x).unwrap());
        
        let x = WalFile { action: WalAction::Fail { count: 10 }, duration: 100, file_name: "test".to_string() };
        assert_eq!("{\"action\":{\"Fail\":{\"count\":10}},\"duration\":100}", serde_json::to_string(&x).unwrap());
    }

    #[test]
    fn wal_file_number() {
        let w = WalFile::generate_wal_file(1, WalAction::Success, 10);
        let expected_w = format!("{}/000000010000000000000001.ready", utilities::STATUS_DIR);

        assert_eq!(expected_w, w.file_name);

        let w = WalFile::generate_wal_file(255, WalAction::Success, 10);
        let expected_w = format!("{}/0000000100000000000000FF.ready", utilities::STATUS_DIR);

        assert_eq!(expected_w, w.file_name);
    }
}