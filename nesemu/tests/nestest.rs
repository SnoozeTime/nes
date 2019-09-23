extern crate nesemu;

use nesemu::cpu::cpu::Cpu;
use nesemu::cpu::memory::Memory;
use nesemu::rom;
use std::fs::File;
use std::io::{BufRead, BufReader};


#[derive(Debug)]
struct LogRecord {
    pc: u16,
    instruction: String,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cyc: u16,
}

fn extract_log(filename: String) -> Result<Vec<LogRecord>, Box<dyn std::error::Error>> {
    let f = File::open(filename)?;
    let reader = BufReader::new(f);

    reader.lines()
        .skip(1)
        .map(|line| {
            let unwrapped = r#try!(line);
            let tokens: Vec<&str> = unwrapped.split(",").collect();

            // A bit ugly? :D
            let pc = u16::from_str_radix(tokens[0], 16)?;
            let instruction = tokens[1].to_string();
            let a = u8::from_str_radix(tokens[2], 16)?;
            let x = u8::from_str_radix(tokens[3], 16)?;
            let y = u8::from_str_radix(tokens[4], 16)?;
            let p = u8::from_str_radix(tokens[5], 16)?;
            let sp = u8::from_str_radix(tokens[6], 16)?;
            let cyc: u16 = tokens[7].parse()?;
            Ok(LogRecord {pc, instruction, a, x, y, p, sp, cyc})
        }).collect()
}

#[test]
fn test() {

    let rom = String::from("roms/nestest.nes");
    let ines = rom::read(rom).expect("Cannot read nestest");
    
    let mut cpu = Cpu::new();
    let mut memory = Memory::new(&ines).expect("Meh");
    cpu.set_pc(0xC000); // Start of automated tests.
    
    let correct_log = extract_log(String::from("tests/nestest.csv")).unwrap();
    for log in correct_log {

        println!("At PC {:X}", cpu.get_pc());
        println!("LogLine {:?}", log);
        assert_eq!(log.pc, cpu.get_pc());
        assert_eq!(log.a, cpu.get_acc());
        assert_eq!(log.x, cpu.get_regx());
        assert_eq!(log.y, cpu.get_regy());
        assert_eq!(log.p, cpu.flags_to_u8_debug());

        if let Err(x) = cpu.next(&mut memory) {
            println!("{}", x);
            break;
        }

        // After that nestest is over.
        if cpu.get_pc() == 0xC66E {
            break;
        }
    }

}
