use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;

mod cpu;
mod rom;
use cpu::cpu::Cpu;

fn load(filename: String) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    Ok(content)
}

pub fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage {} <FILENAME>", args[0]);
    }

    let filename = args[1].clone();
    let bytes = load(filename).unwrap();

    for (i, b) in bytes.iter().enumerate() {

        println!("0x{:x} - 0x{:x}", 0xC000+i, b);

        if (0xFFFF < (0xC000+i)) {
            break;
        }
    }

    let name = args[1].clone();
    let ines = rom::read(name);
    println!("{:?}", ines);
    //let mut nes = Cpu::new(bytes);

   // loop {
     //   if let Err(x) = nes.next() {
       //     println!("{}", x);
         //   break;
        //}
    //}   
}


