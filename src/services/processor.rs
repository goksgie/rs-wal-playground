use std::ffi;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::thread::{self, JoinHandle};

use crate::simulation::lib::SimulationConfig;
use crate::utilities::{self, FileEntry};
use crate::wal::{WalAction, WalFile};

/// This metadata is maintained by the main proccessor, and not thread safe.
struct Metadata {
    /// the first file that processing error has occured,
    /// there is no need to process files that are earlier than this file.
    first_error_at: String,
    
    /// Maintains a mapping between the processed file's name,
    /// and the status, which can be "true" if it was succesful,
    /// "false" otherwise.
    processed_files: HashMap<ffi::OsString, bool>,

    /// The maximum number of items that the "processed_files" can hold on to.
    capacity: usize,

    /// This is added to prevent metadata from accessed/shared between threads.
    /// As, doing so would degrade the performance, which is not necessary, as the
    /// main processor can iterate over the results.
    _marker: PhantomData<*const ()>
}

impl Metadata {
    fn new(capacity: usize) -> Self {
        Metadata {
            first_error_at: String::new(),
            processed_files: HashMap::new(),
            capacity: capacity,
            _marker: PhantomData::default()
        }
    }
}

pub fn wal_processor_internal(sim_config: SimulationConfig) {
    let mut iteration_count = 0;
    let mut processed_wals = std::collections::HashSet::new();
    loop {
        let ready_files = utilities::get_ready_files()
            .expect("The API to list ready files did not terminate correctly")
            .into_iter()
            .filter(|w| { !processed_wals.contains(&w.full_path) })
            .collect::<Vec<FileEntry>>();
        if ready_files.len() == 0 {
            println!("Cleared the WAL files with num iterations: [{}]", iteration_count);
            break;
        }

        for ready_file in ready_files {
            let mut w = WalFile::read(&ready_file.full_path);
            thread::sleep(std::time::Duration::from_nanos(w.duration));
            match w.action {
                WalAction::Success => {
                    processed_wals.insert(w.file_name.clone());
                    w.generate_done_file().expect("Failed to mark the file as done.");
                },
                WalAction::Fail { count: _ } => {
                    w.decrement_failure_count().expect("Failed to decrement the failure count");
                    w.flush_to_file().expect("failed to flush after decrementing the failure count");
                }
            }
        }

        iteration_count += 1;
        thread::sleep(std::time::Duration::from_nanos(sim_config.wal_processing_delay));
    }
}

pub fn service_startup(sim_config: &SimulationConfig) -> JoinHandle<()> {
    let s = sim_config.clone();
    let join_handle = thread::spawn(move || {
        wal_processor_internal(s);
    });

    join_handle
}