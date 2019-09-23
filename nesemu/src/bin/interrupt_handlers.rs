extern crate nesemu;
use std::env;
use nesemu::cpu::memory::Memory;
use nesemu::cpu::cpu::Cpu;
use nesemu::rom;

pub fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage {} <FILENAME>", args[0]);
    }

    let name = args[1].clone();
    let ines = rom::read(name).expect("IIIIINNNNNEEESS");


    let mut memory = Memory::new(&ines).unwrap();
    let mut cpu = Cpu::new();
    
    let lsb = memory.get(0xFFFA) as u16;
    let msb = memory.get(0xFFFB) as u16;
    let start_pc = (msb << 8) + lsb;

    println!("START PC {:X}", start_pc);
    cpu.set_pc(start_pc);
    // Handler should be until RTI. In theory. Some games manipulate the
    // stack and use RTS... (Final fantasy apparently)
    'should_run: loop {
        
        let pc = cpu.get_pc();
        if memory.get(pc as usize) == 0x40 {
            println!("Found RTI");
            break 'should_run;
        }
        cpu.next(&mut memory).unwrap();

    }
}
