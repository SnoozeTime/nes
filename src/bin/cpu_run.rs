extern crate nesemu;
use std::env;
use nesemu::cpu::cpu::Cpu;
use nesemu::rom;

pub fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage {} <FILENAME>", args[0]);
    }

    let name = args[1].clone();
    let ines = rom::read(name).expect("IIIIINNNNNEEESS");


    let mut cpu = Cpu::create(&ines);

    // read reset vctor
    let lsb = cpu.read_mem(0xFFFC) as u16;
    let msb = cpu.read_mem(0xFFFD) as u16;

    let start_pc = (msb << 8) + lsb;
    cpu.set_pc(start_pc);
    loop {
        cpu.next().unwrap();
//        println!("$2000: {:X}", cpu.read_mem(0x2000));

    }
}

