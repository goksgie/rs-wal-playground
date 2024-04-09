use std::thread::{self, JoinHandle};
use std::time::Duration;
use rand::prelude::*;

use crate::simulation::lib::SimulationConfig;
use crate::wal::{WalAction, WalFile};

fn file_generator_internal(simulation_config: SimulationConfig) {
    let mut num_files_generated = 0;
    let mut rng = simulation_config.rng.unwrap().clone();
    while num_files_generated < 1 {
        // decide on generated action:
        let action = if rng.gen_bool(simulation_config.wal_failure_ratio) {
            WalAction::Fail { count: rng.gen_range(simulation_config.wal_failure_attempt_min..simulation_config.wal_failure_attempt_max) }
        } else {
            WalAction::Success
        };

        let work_duration = rng.gen_range(
            simulation_config.wal_process_duration_min..simulation_config.wal_process_duration_max);
        let m = WalFile::generate_wal_file(num_files_generated, action, work_duration);
        m.flush_to_file().expect("Failed to write a WAL file.");
        thread::sleep(Duration::from_nanos(simulation_config.wal_generation_delay));
        num_files_generated += 1;
    }
}

pub fn service_startup(simulation_config: &SimulationConfig) -> JoinHandle<()> {
    let x = simulation_config.clone();
    let join_handle = thread::spawn(move || {
        file_generator_internal(x);
    });

    join_handle
}