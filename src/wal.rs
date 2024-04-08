use std::{ffi::OsString, io::{Read, Write}};
use serde_json;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum WalAction {
    /// Signifies the number of times that uploading this file will fail.
    /// When it is 0, it is expected to be succeeded.
    Fail { count: u16 },

    /// Upload is successful.
    Success,
}

/// Represents the WAL file format.
#[derive(Serialize, Deserialize)]
pub struct WalFile {
    /// The type of action to be performed by the processor.
    action: WalAction,

    /// The duration where each action will take. The unit is
    /// milliseconds.
    duration: u64, 

    /// The file name to be stored to take action on it.
    #[serde(skip_serializing, skip_deserializing)]
    file_name: OsString,
}

impl WalFile {
    /// Reads the provided WAL file and constructs the WAL file format.
    pub fn read(f_name: OsString) -> Self {
        let mut file_contents = String::new();
        let mut buffer = vec![0; 1024];

        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .open(&f_name)
            .expect("File does not exist");
        let read_bytes = f.read_to_end(&mut buffer).expect("Reading from file did not end up as expected");
        println!("File size was: {}", read_bytes); 

        file_contents = String::from_utf8(buffer).expect("Failed to convert the read bytes into string");

        let mut wal_file: WalFile = serde_json::from_str(&file_contents).expect("The WAL has incorrect formatting");
        wal_file.file_name = f_name;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_ignore_file_name() {
        let x = WalFile { action: WalAction::Success, duration: 10, file_name: OsString::from("test") };
        let y: WalFile = serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
        assert!(y.file_name.is_empty());

        let x = WalFile { action: WalAction::Fail { count: 100 }, duration: 10, file_name: OsString::from("test") };
        let y: WalFile = serde_json::from_str(&serde_json::to_string(&x).unwrap()).unwrap();
        assert!(y.file_name.is_empty());
    }

    #[test]
    fn serialization_format() {
        let x = WalFile { action: WalAction::Success, duration: 10, file_name: OsString::from("test") };
        assert_eq!("{\"action\":\"Success\",\"duration\":10}", serde_json::to_string(&x).unwrap());
        
        let x = WalFile { action: WalAction::Fail { count: 10 }, duration: 100, file_name: OsString::from("test") };
        assert_eq!("{\"action\":{\"Fail\":{\"count\":10}},\"duration\":100}", serde_json::to_string(&x).unwrap());
    }
}