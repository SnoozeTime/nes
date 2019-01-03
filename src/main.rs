use clap::{Arg, SubCommand, App};
use nesemu::nes::Nes;
use nesemu::rom;

fn run_rom(path: String) {
    let ines = rom::read(path).unwrap();
    let mut nes = Nes::new(ines).unwrap();
    nes.run().unwrap();
}

fn load_state(path: String) {
    let mut nes = Nes::load_state().unwrap();
    nes.run().unwrap();
}

fn main() {
    let matches = App::new("My Super Program")
        .version("1.0")
        .subcommand(SubCommand::with_name("run")
                    .about("Run emulator with ROM file")
                    .arg(Arg::with_name("input")
                         .short("i")
                         .help("Path of the ROM file")
                         .required(true)
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("load")
                    .about("Load emulator state from file")
                    .arg(Arg::with_name("input")
                         .short("i")
                         .help("Path of the state file")
                         .required(true)
                         .takes_value(true)))
        .get_matches();

    env_logger::init();
    if let Some(matches) = matches.subcommand_matches("run") {
        let rom_path = matches.value_of("input").unwrap();
        run_rom(rom_path.to_string());
    } else if let Some(matches) = matches.subcommand_matches("load") {
        let state_path = matches.value_of("input").unwrap();
        load_state(state_path.to_string());
    } else {
        panic!("Should use run or load subcommand");
    }
}
