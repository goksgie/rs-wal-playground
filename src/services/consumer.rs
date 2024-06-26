use std::time::Duration;
use std::thread::{self, JoinHandle};
use std::fs;

use crate::simulation::lib::SimulationConfig;
use crate::utilities;
use crate::wal::*;

fn wal_consumer_internal(simulation_config: SimulationConfig) {
    loop {
        thread::sleep(Duration::from_nanos(simulation_config.wal_consumer_delay));
        let done_files = utilities::get_done_files()
            .expect("failed to acquire .done files generated by the processor.")
            .into_iter()
            .map(|file: utilities::FileEntry| {
                file.file_name
            })
            .collect::<Vec<String>>(); 

        let filter_fn = |file_name: &str| {
            if !file_name.ends_with(".ready") {
                return false;
            }

            let file_name = file_name.split(".")
                .take(1)
                .collect::<String>();
            done_files.contains(&file_name)
        };

        let files_to_mark_done = utilities::walk_directory(utilities::STATUS_DIR, filter_fn)
            .expect("Failed to acquire WAL files to be marked as done");
        if files_to_mark_done.len() == 0 {
            println!("No work to do for WAL consumer");
            break;
        }

        for wal_file_path in files_to_mark_done {
            let wal_file = WalFile::read(&wal_file_path.full_path);
            match wal_file.action {
                WalAction::Fail { count: _ } => {
                    panic!("The WAL file was listed to mark as done but its contents are not succeeded: {:?}", wal_file_path);
                },

                _ => {
                    wal_file.mark_done().expect("Failed to mark the WAL file as done.");
                    
                    // remove the corresponding marker file from the source directory.
                    fs::remove_file(format!("{}/{}.done", utilities::SOURCE_DIR, wal_file_path.file_name))
                        .expect("Failed to remove a status file");
                }
            }
        }
    }
}

pub fn service_startup(simulation_config: &SimulationConfig) -> JoinHandle<()> {
    let x = simulation_config.clone();
    let join_handle = thread::spawn(move || {
        wal_consumer_internal(x);
    });

    join_handle
}