use std::io::Read;
use serde::{Serialize, Deserialize};
use serde_json;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::utilities;

/// Represents the simulation configurations that will
/// be read from the simulation_conf.json file.
#[derive(Serialize, Deserialize)]
pub struct SimulationConfig {
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
    pub fn get_simulation_config() -> Self {
        let mut buffer = vec![0; 1024];
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .create(false)
            .open(format!("{}/simulation_conf.json", utilities::SIMULATION_DIR))
            .expect("The simulation configuration file does not exist.");
        f.read_to_end(&mut buffer).expect("failed to read the file.");

        let buffer = String::from_utf8(buffer).unwrap();
        let buffer = buffer.replace("\0", "");
        println!("{:?}", buffer);
        let mut conf: SimulationConfig = serde_json::from_str(&buffer)
            .expect("failed to deserialize the simulation config");

        // setup the RNG
        conf.rng = Some(ChaCha8Rng::seed_from_u64(conf.seed));
        conf
    }
}