use nesemu::nes::Nes;
use nesemu::rom;
use std::env;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage {} <FILENAME>", args[0]);
    }

    let name = args[1].clone();
    let ines = rom::read(name).unwrap();

    let mut nes = Nes::new(ines).unwrap();
    nes.decompile();
}
