mod utilities;
mod services;
mod wal;
mod simulation;

use crate::services::{consumer, generator, processor};

fn main() {
    let simulation_config = simulation::lib::SimulationConfig::get_simulation_config();

    let ready_files = utilities::get_ready_files().unwrap();
    let done_files = utilities::get_done_files().unwrap();

    println!("{:?}", ready_files);
    println!("{:?}", done_files);

    let gen_handle = generator::service_startup(&simulation_config);
    gen_handle.join().unwrap();

    let proc_handle = processor::service_startup(&simulation_config);
    proc_handle.join().unwrap();

    let consumer_handle = consumer::service_startup(&simulation_config);
    consumer_handle.join().unwrap();
}
