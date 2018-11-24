use std::env;
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
    nes.set_pc(0xC000);

    loop {
        if let Err(x) = nes.next() {
            println!("{}", x);
            break;
        }
    }   
}


