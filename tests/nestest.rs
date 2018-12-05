extern crate nesemu;
use std::io::Read;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

        }

        if token.contains("A:") {

        }

        if token.contains("X:") {

        }

        if token.contains("Y:") {

        }

        if token.contains("P:") {

        }

        if token.contains("SP:") {

        }

        if token.contains("CYC") {

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
    assert_eq!(1, 1);
}
