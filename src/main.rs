use std::env;
use std::io::prelude::*;
use std::io::Error;
mod nes;
mod cpu;
mod rom;
use cpu::cpu::Cpu;



pub fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage {} <FILENAME>", args[0]);
    }

    let name = args[1].clone();
    let ines = rom::read(name).expect("IIIIINNNNNEEESS");
    let mut nes = Cpu::create(&ines);

    loop {
        if let Err(x) = nes.next() {
            println!("{}", x);
            break;
        }
    }   
}


