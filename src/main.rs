use std::io::Read;
use serde::{Serialize, Deserialize};
use serde_json;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

mod utilities;
mod services;
mod wal;

/// Represents the simulation configurations that will
/// be read from the simulation_conf.json file.
#[derive(Serialize, Deserialize)]
struct SimulationConfig {
    /// A fixed seed that will be fed to the randomizer to obtain the same randomization always.
    seed: u64,
    
    /// Specifies the amount of delay to be put between WAL file generation.
    /// Unit is nanoseconds. 
    wal_generation_delay: u64,
    
    /// Specifies the amount of delay to be put between WAL file processing.
    /// Unit is nanoseconds.
    wal_processing_delay: u64,
    
    /// Specifies the minimum amount of time that will be spend on processing a particular
    /// WAL file. The unit is nanoseconds.
    wal_process_duration_min: u64,
    
    /// Specifies the maximum amount of time that will be spend on processing a particular
    /// WAL file. The unit is nanoseconds.
    wal_process_duration_max: u64,

    #[serde(skip_serializing, skip_deserializing)]
    rng: Option<ChaCha8Rng>,
}

impl SimulationConfig {
    fn get_simulation_config() -> Self {
        let mut buffer = vec![0; 1024];
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .create(false)
            .open(format!("{}/simulation_conf.json", utilities::SIMULATION_DIR))
            .expect("The simulation configuration file does not exist.");
        f.read_to_end(&mut buffer).expect("failed to read the file.");

        let mut conf: SimulationConfig = serde_json::from_str(&String::from_utf8(buffer).expect("Failed to generate String from config"))
            .expect("failed to deserialize the simulation config");

        // setup the RNG
        conf.rng = Some(ChaCha8Rng::seed_from_u64(conf.seed));
        conf
    }
}

fn main() {
    let simulation_config = SimulationConfig::get_simulation_config();

    let ready_files = utilities::get_ready_files().unwrap();
    let done_files = utilities::get_done_files().unwrap();

    println!("{:?}", ready_files);
    println!("{:?}", done_files);
}
