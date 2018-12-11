extern crate nesemu;
use std::env;
use nesemu::cpu::cpu::Cpu;
use nesemu::rom;
use std::{thread, time};

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
        println!("PPUCTRL -> $2000: {:08b}",
                  cpu.read_mem(0x2000),
                  );

        println!("PPUMASK -> $2001: {:08b}", cpu.read_mem(0x2001));
        println!("PPUSTATUS -> $2002: {:08b}", cpu.read_mem(0x2002));
        println!("$2003: {:X}", cpu.read_mem(0x2003));
        println!("$2004: {:X}", cpu.read_mem(0x2004));
        println!("$2005: {:X}", cpu.read_mem(0x2005));
        println!("$2006: {:X}", cpu.read_mem(0x2006));
        println!("$2007: {:X}", cpu.read_mem(0x2007));
        let ten_millis = time::Duration::from_millis(1);
        thread::sleep(ten_millis);
//        println!("{}[2J", 27 as char);
        
       cpu.set_mem(0x2002, 0x80); 
    }
}

