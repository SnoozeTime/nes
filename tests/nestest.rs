extern crate nesemu;

use nesemu::cpu::cpu::Cpu;
use nesemu::rom;
use std::io::Read;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct LogLine {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cycles: u16,
}
fn parse_ref_line(line: String) -> LogLine {
    let mut pc = 0;
    let mut a = 0;
    let mut x = 0;
    let mut y = 0;
    let mut p = 0;
    let mut sp = 0;
    let mut cycles = 0;

    for (i, token) in line.split_whitespace().enumerate() {

        if i == 0 {
            pc = u16::from_str_radix(token, 16).unwrap();
        }

        if token.contains("A:") {
            let splits: Vec<&str> = token.split(':').collect();
            a = u8::from_str_radix(splits.get(1).unwrap(), 16).unwrap();
        }

        if token.contains("X:") {
            let splits: Vec<&str> = token.split(':').collect();
            x = u8::from_str_radix(splits.get(1).unwrap(), 16).unwrap();
        }

        if token.contains("Y:") {
            let splits: Vec<&str> = token.split(':').collect();
            y = u8::from_str_radix(splits.get(1).unwrap(), 16).unwrap();
        }

        if token.contains("P:") && !token.contains("SP:") {
            let splits: Vec<&str> = token.split(':').collect();
            p = u8::from_str_radix(splits.get(1).unwrap(), 16).unwrap();
        }

        if token.contains("SP:") {
            let splits: Vec<&str> = token.split(':').collect();
            sp = u8::from_str_radix(splits.get(1).unwrap(), 16).unwrap();
        }

        if token.contains("CYC") {
//             let splits: Vec<&str> = token.split(':').collect();
//             a = splits[1].parse().unwrap();
        }
    }

    LogLine{pc, a, x, y, p, sp, cycles}
}

fn parse_ref_log(filename: String) -> Vec<LogLine> {
    let reader = BufReader::new(File::open(filename).expect("Cannot open file.txt"));
    reader.lines().map(|line| parse_ref_line(line.unwrap())).collect()
}

#[test]
fn test() {

    let rom = String::from("roms/nestest.nes");
    let ines = rom::read(rom).expect("Cannot read nestest");
    
    let mut cpu = Cpu::create(&ines);
    cpu.set_pc(0xC000); // Start of automated tests.
    
    let correct_log = parse_ref_log(String::from("tests/correct.log"));
    for log in correct_log {

        println!("At PC {:X}", cpu.get_pc());
        println!("LogLine {:?}", log);
        assert_eq!(log.pc, cpu.get_pc());
        assert_eq!(log.a, cpu.get_acc());
        assert_eq!(log.x, cpu.get_regx());
        assert_eq!(log.y, cpu.get_regy());
        assert_eq!(log.p, cpu.flags_to_u8_debug());

        if let Err(x) = cpu.next() {
            println!("{}", x);
            break;
        }

        // After that nestest is over.
        if cpu.get_pc() == 0xC66E {
            break;
        }
    }
}
