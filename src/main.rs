#[macro_use]
extern crate log;
extern crate env_logger;

extern crate sdl2;
extern crate nesemu;
use std::env;
use nesemu::nes::Nes;
use nesemu::rom;

pub fn main() {
    env_logger::init();
    // TODO use lib to parse args
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Usage {} ROM|FILE <FILENAME>", args[0]);
    }

    if args[1] == "ROM" {
        let name = args[1].clone();
        let ines = rom::read(name).unwrap();

        let mut nes = Nes::new(ines).unwrap();
        info!("Will start {}", args[1]);
        nes.run().unwrap();
    } else {
        let mut nes = Nes::load_state().unwrap();
        nes.run().unwrap();

    }

}

