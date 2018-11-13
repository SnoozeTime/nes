use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;

// memory layout
// ---------------
// Address range    Size    Device
// $0000-$07FF  $0800   2KB internal RAM
// $0800-$0FFF  $0800   Mirrors of $0000-$07FF
// $1000-$17FF  $0800
// $1800-$1FFF  $0800
// $2000-$2007  $0008   NES PPU registers
// $2008-$3FFF  $1FF8   Mirrors of $2000-2007 (repeats every 8 bytes)
// $4000-$4017  $0018   NES APU and I/O registers
// $4018-$401F  $0008   APU and I/O functionality that is normally disabled. See CPU Test Mode.
// $4020-$FFFF  $BFE0   Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note) 
struct Nes {

    code: Vec<u8>, // contains the instructions.
    
    // Program counter. Hold the address of the next instruction to be executed
    PC: u16,

    // stack pointer. from top to bottom.
    SP: u8, 

    // Accumulator. 8 bits register used for arithmetic and logic operation
    A: u8,

    // Index registers
    X: u8,
    Y: u8,

    // Processor status (Flags register)
    P: u8,
}

impl Nes {
   
    // will create a new NES with the given code.
    fn new(code: Vec<u8>) -> Nes {
        Nes {
            code,
            PC: 0,
            SP: 0xFF, 
            A: 0,
            X: 0,
            Y: 0,
            P: 0,
        }
    }
    
    // Will fetch/decode/execute the next operation. This is a BIG switch statement.
    fn next(&mut self) -> Result<(), Box<std::error::Error>> {
    
        let opcode = self.code[self.PC as usize];
        
        // let instruction_result = match { .... }
        match opcode {
        0x29 => println!("AND #operand"),
        0x25 => println!("AND operand"),
        _ => print!(""),
        }

        self.PC += 1;

        Ok( () )
    }

}

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
    let mut nes = Nes::new(bytes);

    loop {
        nes.next();
    }   
    println!("Geez Rick!");
}


