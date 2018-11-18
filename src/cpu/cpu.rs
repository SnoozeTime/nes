use super::instructions::Instruction;
use super::memory::*;


#[allow(non_snake_case)] // PC, SP ... are names in the specs.
pub struct Nes {

    memory: Memory,

    // Program counter. Hold the address of the next instruction to be executed
    PC: u16,

    // stack pointer. from top to bottom.
    SP: u8, 

    // Accumulator. 8 bits register used for arithmetic and logic operation
    A: u8,

    // Index registers
    X: u8,
    Y: u8,

    // Actually, we have memory to spare so let's just use
    // one byte for each flag.
    C: u8, // Carry
    Z: u8, // Zero
    I: u8, // Interrupt disable
    D: u8, // Decimal mode
    B: u8, // Break
    V: u8, // Overflow
    N: u8, // negative
}

impl Nes {
   
    // will create a new NES with the given code.
    pub fn new(code: Vec<u8>) -> Nes {
        Nes {
            memory: Memory::new(code),
            PC: 0x8000,
            SP: 0xFF, 
            A: 0,
            X: 0,
            Y: 0,
            C: 0,
            Z: 0,
            I: 0,
            D: 0,
            B: 0,
            V: 0,
            N: 0,
        }
    }

    pub fn next(&mut self) -> Result<(), Box<std::error::Error>> {

        let instruction = self.decode();
        println!("{}", instruction.repr());

        match instruction {
            Instruction::ADC(_, addressing, _length) => {
                // http://www.6502.org/tutorials/vflag.html
                // A,Z,C,N,V = A+M+C
                // ADC can be used both with signed and unsigned numbers.
                // 
                let rhs = addressing.fetch(&self.memory);
                // max value is 0x1FF. There is carry if > 0xFF.
                let sum: u16 = (self.A as u16)
                    + (rhs as u16) + (self.C as u16);
                let result = (sum & 0xFF) as u8; 
                self.C = (sum >> 8) as u8;

                self.set_result_flags(result);

                // now the overflow.
                // if addition of two negative numbers yield a positive result, set
                // V to 1.
                // if addition of two positive numbers yield a negative result, set V 
                // to 1.
                // TODO Do that better
                if (rhs ^ self.A) >> 7 == 0 {
                    // same sign
                    if (rhs ^ result) >> 7 == 1 {
                        self.V = 1;
                    } else {
                        self.V = 0;
                    }
                } else {
                    self.V = 0;
                }

                self.A = result;
            }
            Instruction::LDA(_, addressing, _length) => {
                // Affect N and Z flags
                let result = addressing.fetch(&self.memory);
                self.A = result;
                self.set_result_flags(result);    
            },
            Instruction::LDX(_, addressing, _length) => {
                let result = addressing.fetch(&self.memory);
                self.X = result;
                self.set_result_flags(result);
            },
            Instruction::LDY(_, addressing, _length) => {
                let result = addressing.fetch(&self.memory);
                self.Y = result;
                self.set_result_flags(result);
            },

            _ => {}
        }

        Ok(())
    }

    // set negative or zero flag depending on result of operation.
    fn set_result_flags(&mut self, result: u8) {
        //  Z flag set if A = 0
        if result == 0 {
            self.Z = 1;
        } else {
            self.Z = 0;
        }

        // negative if bit at 7th position is set.
        self.N = result >> 7;
    }
    
    // Decode the next instruction
    fn decode(&mut self) -> Instruction {

     
        let line = self.PC;
        let opcode = self.advance();

        // let instruction_result = match { .... }
        match opcode {

        // -----------------------------------
        // Add with Carry ADC
        // -----------------------------------
        0x69 => {
            let operand = self.advance();
            Instruction::ADC(line,
                             ImmediateAddressing::new(operand),
                             2)
        },
        0x65 => {
            let operand = self.advance();
            Instruction::ADC(line,
                             ZeroPageAddressing::new(operand),
                             3)
        },
        0x75 => {
            let operand = self.advance();
            Instruction::ADC(line,
                             IndexedZeroPageAddressing::new(operand, self.X),
                             4)
        },
        0x6D => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::ADC(line,
                             AbsoluteAddressing::new(operand1, operand2),
                             4)
        },
        0x7D => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::ADC(line,
                             IndexedAbsoluteAddressing::new(operand1, operand2, self.X),
                             4)
        },
        0x79 => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::ADC(line,
                             IndexedAbsoluteAddressing::new(operand1, operand2, self.Y),
                             4)
        },
        0x61 => {
            let operand = self.advance();
            Instruction::ADC(line,
                             PreIndexedIndirectAddressing::new(operand, self.X),
                             6)
        },
        0x71 => {
            let operand = self.advance();
            Instruction::ADC(line,
                             PostIndexedIndirectAddressing::new(operand, self.Y),
                             5)
        },

        // ------------------------------------
        // LoaD Accumulator LDA
        // -----------------------------------
        0xA9 => {
            // LDA #$44
            let operand = self.advance();
            Instruction::LDA(line, ImmediateAddressing::new(operand), 2)
        },
        0xA5 => {
            // LDA $44
            let operand = self.advance();
            Instruction::LDA(line, ZeroPageAddressing::new(operand), 3)
        },
        0xB5 => {
            // LDA $44,X
            let operand = self.advance();
            Instruction::LDA(line, IndexedZeroPageAddressing::new(operand, self.X), 4)
        },
        0xAD => {
            // LDA $4400
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDA(line, AbsoluteAddressing::new(operand1, operand2), 4)
        },
        0xBD => {
            // LDA $4400,X
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDA(line,
                             IndexedAbsoluteAddressing::new(operand1,
                                                            operand2,
                                                            self.X),
                             4)
        },
        0xB9 => {
            // LDA $4400,Y
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDA(line,
                             IndexedAbsoluteAddressing::new(operand1,
                                                            operand2,
                                                            self.Y),
                             4)

        },
        0xA1 => {
            // LDA ($44, X)
            let operand = self.advance();
            Instruction::LDA(line,
                             PreIndexedIndirectAddressing::new(operand, self.X),
                             6)
        },
        0xB1 => {
            // LDA ($44), Y
            let operand = self.advance();
            Instruction::LDA(line,
                             PostIndexedIndirectAddressing::new(operand, self.Y),
                             5)
        },
        // ------------------------------------
        // LDX - Load X
        // ------------------------------------
        0xA2 => {
            let operand = self.advance();
            Instruction::LDX(line, ImmediateAddressing::new(operand), 2)
        },
        0xA6 => {
            let operand = self.advance();
            Instruction::LDX(line, ZeroPageAddressing::new(operand), 3)
        },
        0xB6 => {
            let operand = self.advance();
            Instruction::LDX(line, IndexedZeroPageAddressing::new(operand, self.Y), 4)
        },
        0xAE => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDX(line, AbsoluteAddressing::new(operand1, operand2), 4)
        },
        0xBE => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDX(line,
                             IndexedAbsoluteAddressing::new(operand1, operand2, self.Y),
                             4)
        },
        // ------------------------------------
        // LDY - Load Y
        // ------------------------------------
        0xA0 => {
            let operand = self.advance();
            Instruction::LDY(line, ImmediateAddressing::new(operand), 2)
        },
        0xA4 => {
            let operand = self.advance();
            Instruction::LDY(line, ZeroPageAddressing::new(operand), 3)
        },
        0xB4 => {
            let operand = self.advance();
            Instruction::LDY(line, IndexedZeroPageAddressing::new(operand, self.X), 4)
        },
        0xAC => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDY(line, AbsoluteAddressing::new(operand1, operand2), 4)
        },
        0xBC => {
            let operand1 = self.advance();
            let operand2 = self.advance();
            Instruction::LDY(line,
                             IndexedAbsoluteAddressing::new(operand1, operand2, self.X),
                             4)
        },
        // -------------------------------------
        // STX - Store X
        // Affect flags: None
        // ------------------------------------
//        0x86 => {
//            // Zero page
//            let zerop_loc = self.advance();
//            println!("STX Operand - Store X at 0x{:x}", zerop_loc);
//        },
//        0x96 => {
//            // Indexing zero page.
//            let zerop_loc = self.advance();
//            println!("STX Operand,Y - Store X at 0x{:x}+Y", zerop_loc);
//        },
//        0x8E => {
//            // absolute indexing
//            let lsb = self.advance();
//            let msb = self.advance();
//            let loc = ((msb as u16) << 8) | (lsb as u16);
//            println!("STX Operand - Absolute. Store X at 0x{:x}", loc);
//        },
//
        _ => Instruction::UNKNOWN(line),
        }

    }
    // Get next instruction and increment PC
    fn advance(&mut self) -> u8 {
        let code = self.memory.get(self.PC as usize);
        self.PC += 1;
        code
    }
}

#[cfg(test)]
mod tests {

    // get names from outer scope.
    use super::*;


    #[test]
    fn test_LDA_immediate_no_flag() {
        // Load accumulator. Immediate addressing
        let code = vec![0xA9, 0x36]; 

        let mut nes = Nes::new(code);
        assert_eq!(0x8000, nes.PC);
        nes.next();

        assert_eq!(0x8002, nes.PC);
        assert_eq!(0x36, nes.A);
    }

    #[test]
    fn test_LDA_zeropage_negative() {
        let code = vec![0xA5, 0x06]; 

        let mut nes = Nes::new(code);
        nes.memory.set(0x06, 0x84);
        assert_eq!(0x8000, nes.PC);
        nes.next();

        assert_eq!(0x8002, nes.PC);
        assert_eq!(0x84, nes.A);
        assert_eq!(1, nes.N); 
    }

    #[test]
    fn test_LDA_absolute_processor_zero() {
        let code = vec![0xAD, 0x06, 0xA3]; 

        let mut nes = Nes::new(code);
        nes.memory.set(0xA306, 0x00);
        assert_eq!(0x8000, nes.PC);
        nes.next();

        assert_eq!(0x8003, nes.PC);
        assert_eq!(0x00, nes.A);
        assert_eq!(0x01, nes.Z);
    }

    #[test]
    fn test_LDX_indexed_zp() {
        let code = vec![0xB6, 0x04];
        let mut nes = Nes::new(code);
        nes.Y = 0x02;
        nes.memory.set(0x06, 0x0A);
        nes.next();
        assert_eq!(0x0A, nes.X);
    }

    #[test]
    fn test_LDY_indexed_absolute() {
        let code = vec![0xBC, 0x06, 0xA3]; 
    
        let mut nes = Nes::new(code);
        nes.X = 0x02;
        nes.memory.set(0xA308, 0x11);
        assert_eq!(0x8000, nes.PC);
        nes.next();

        assert_eq!(0x8003, nes.PC);
        assert_eq!(0x11, nes.Y);
    }

    #[test]
    fn test_adc_without_carry() {
        // now carry, no overflow.
        let code = vec![0xA9, 0x01, 0x69, 0x10]; // A should be 0x11

        let mut nes = Nes::new(code);
        nes.next();
        nes.next();
        assert_eq!(0x11, nes.A);
        assert_eq!(0, nes.C);
        assert_eq!(0, nes.V);
    }

    #[test]
    fn test_ADC_with_carry_no_overflow() {
        let code = vec![0xA9, 0xF1, 0x69, 0x19];

        // if unsigned, 0xF1 + 0x19 = (decimal) 266 => 10 and one carry
        // if signed 0xF1 (-15) + 0x19 (25) = 10
        // no overflow as operands are not the same sign.
        //
        let mut nes = Nes::new(code);
        nes.next();
        nes.next();
        assert_eq!(0x0A, nes.A);
        assert_eq!(1, nes.C);
        assert_eq!(0, nes.V);
    }

    #[test]
    fn test_ADC_positive_overflow() {
        let code = vec![0xA9, 0x64, 0x69, 0x64]; 

        // if signed 0x64 (>0) + 0x64 (>0) = 0xc8 (< 0)
        let mut nes = Nes::new(code);
        nes.next();
        nes.next();
        assert_eq!(0xC8, nes.A);
        assert_eq!(0, nes.C);
        assert_eq!(1, nes.V);

    }
}


