mod utilities;
mod services;
mod wal;
mod simulation;


fn main() {
    let simulation_config = simulation::lib::SimulationConfig::get_simulation_config();

    let ready_files = utilities::get_ready_files().unwrap();
    let done_files = utilities::get_done_files().unwrap();

    println!("{:?}", ready_files);
    println!("{:?}", done_files);
}
