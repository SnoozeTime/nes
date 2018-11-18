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

    pub fn A(&self) -> u8 {
        self.A
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
            },
            Instruction::AND(_, addressing, _length) => {
                let result = self.A & addressing.fetch(&self.memory);
                self.set_result_flags(result);
                self.A = result;
            },
            Instruction::ASL(_, addressing, _length) => {
                // This is a funny one.
                let shifted: u16 = (addressing.fetch(&self.memory) as u16) << 1;
                let result = (shifted & 0xFF) as u8;
                self.C = (shifted >> 8) as u8;

                match &addressing.mode_type() {
                    AddressingModeType::Accumulator => self.A = result,
                    _ => addressing.set(&mut self.memory, result),
                }
                self.set_result_flags(result);
            },
            Instruction::BCC(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.C == 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },
            Instruction::BCS(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.C != 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },

            Instruction::BEQ(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.Z != 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },
            Instruction::BIT(_, addressing, _length) => {
                let to_test = addressing.fetch(&self.memory);
                // set Z if to_test & A == 0
                if (to_test & self.A) == 0 {
                    self.Z = 1;
                } else {
                    self.Z = 0;
                }

                self.V = (to_test >> 6) & 0x1;
                self.N = (to_test >> 7) & 0x1;
            },
            Instruction::BMI(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.N != 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },
            Instruction::BNE(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.Z == 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },
            Instruction::BPL(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.N == 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },
            Instruction::BVC(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.V == 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },
            Instruction::BVS(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.V != 0 { 
                   // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                }
            },

            Instruction::CLC(_, _length) => self.C = 0,
            Instruction::CLD(_, _length) => self.D = 0,
            Instruction::CLI(_, _length) => self.I = 0,
            Instruction::CLV(_, _length) => self.V = 0,

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

        // -----------------------------------
        // AND
        // ------------------------------------
        0x29 => {
            let operand = self.advance();
            Instruction::AND(line,
                             ImmediateAddressing::new(operand),
                             2)
        },
        0x25 => {
            let operand = self.advance();
            Instruction::AND(line,
                             ZeroPageAddressing::new(operand),
                             3)
        },
        0x35 => {
            let operand = self.advance();
            Instruction::AND(line,
                             IndexedZeroPageAddressing::new(operand, self.X),
                             4)
        },
        0x2D => {
            let operand = self.advance();
            let operand1 = self.advance();
            Instruction::AND(line,
                             AbsoluteAddressing::new(operand, operand1),
                             4)
        },
        0x3D => {
            let operand = self.advance();
            let operand1 = self.advance();
            Instruction::AND(line,
                             IndexedAbsoluteAddressing::new(operand, operand1, self.X),
                             4)
        },
        0x39 => {
            let operand = self.advance();
            let operand1 = self.advance();
            Instruction::AND(line,
                             IndexedAbsoluteAddressing::new(operand, operand1, self.Y),
                             4)
        },
        0x21 => {
            let operand = self.advance();
            Instruction::AND(line,
                             PreIndexedIndirectAddressing::new(operand, self.X),
                             6)
        },
        0x31 => {
            let operand = self.advance();
            Instruction::AND(line,
                             PostIndexedIndirectAddressing::new(operand, self.Y),
                             5)
        },
        // -----------------------------------
        // ASL arithmetic shift left
        // -----------------------------------
        0x0A => {
            Instruction::ASL(line,
                             AccumulatorAddressing::new(&self),
                             2)
        },
        0x06 => {
            let operand = self.advance();
            Instruction::ASL(line,
                             ZeroPageAddressing::new(operand),
                             5)
        }
        0x16 => {
            let operand = self.advance();
            Instruction::ASL(line,
                             IndexedZeroPageAddressing::new(operand, self.X),
                             6)

        },
        0x0E => {
            let operand = self.advance();
            let operand1 = self.advance();
            Instruction::ASL(line,
                             AbsoluteAddressing::new(operand, operand1),
                             6)
        },
        0x1E => {
            let operand = self.advance();
            let operand1 = self.advance();
            Instruction::ASL(line,
                             IndexedAbsoluteAddressing::new(operand, operand1, self.X),
                             7)
        },
        // --------------------------------
        // BCC Branch if Carry Clear
        // --------------------------------
        0x90 => {
            let operand = self.advance();
            Instruction::BCC(line,
                             RelativeAddressing::new(operand),
                             2)
        },
        // ----------------------------------
        // BCS Branch if Carry Set
        // -------------------------------------
        0xB0 => {
            let operand = self.advance();
            Instruction::BCS(line,
                             RelativeAddressing::new(operand),
                             2)
        },
        // ----------------------------
        // BEQ - Branch if Equal
        // -----------------------------
        0xF0 => {
            let operand = self.advance();
            Instruction::BEQ(line, RelativeAddressing::new(operand), 2)
        },
        // ----------------------------
        // BIT - Bit test
        // ---------------------------
        0x24 => {
            let operand = self.advance();
            Instruction::BIT(line, ZeroPageAddressing::new(operand), 3)
        },
        0x2C => {
            let operand = self.advance();
            let operand1 = self.advance();
            Instruction::BIT(line, AbsoluteAddressing::new(operand, operand1), 4)
        },
        // ----------------------------
        // BMI - Branch if Minus
        // -----------------------------
        0x30 => {
            let operand = self.advance();
            Instruction::BMI(line, RelativeAddressing::new(operand), 2)
        },

        // ----------------------------
        // BNE - Branch if Not Equal
        // -----------------------------
        0xD0 => {
            let operand = self.advance();
            Instruction::BNE(line, RelativeAddressing::new(operand), 2)
        },
        // ----------------------------
        // BPL - Branch if Positive
        // -----------------------------
        0x10 => {
            let operand = self.advance();
            Instruction::BPL(line, RelativeAddressing::new(operand), 2)
        },
        // ----------------------------
        // BVC - Branch if overflow clear
        // -----------------------------
        0x50 => {
            let operand = self.advance();
            Instruction::BVC(line, RelativeAddressing::new(operand), 2)
        },
        // ----------------------------
        // BVS - Branch if overflow set
        // -----------------------------
        0x70 => {
            let operand = self.advance();
            Instruction::BVS(line, RelativeAddressing::new(operand), 2)
        },
        
        // -------------------------------
        // Clear flag instructions
        // -------------------------------
        0x18 => Instruction::CLC(line, 2),
        0xD8 => Instruction::CLD(line, 2),
        0x58 => Instruction::CLI(line, 2),
        0xB8 => Instruction::CLV(line, 2),
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

#[allow(non_snake_case)]
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
        nes.next().unwrap();

        assert_eq!(0x8002, nes.PC);
        assert_eq!(0x36, nes.A);
    }

    #[test]
    fn test_LDA_zeropage_negative() {
        let code = vec![0xA5, 0x06]; 

        let mut nes = Nes::new(code);
        nes.memory.set(0x06, 0x84);
        assert_eq!(0x8000, nes.PC);
        nes.next().unwrap();

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
        nes.next().unwrap();

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
        nes.next().unwrap();
        assert_eq!(0x0A, nes.X);
    }

    #[test]
    fn test_LDY_indexed_absolute() {
        let code = vec![0xBC, 0x06, 0xA3]; 
    
        let mut nes = Nes::new(code);
        nes.X = 0x02;
        nes.memory.set(0xA308, 0x11);
        assert_eq!(0x8000, nes.PC);
        nes.next().unwrap();

        assert_eq!(0x8003, nes.PC);
        assert_eq!(0x11, nes.Y);
    }

    #[test]
    fn test_adc_without_carry() {
        // now carry, no overflow.
        let code = vec![0xA9, 0x01, 0x69, 0x10]; // A should be 0x11

        let mut nes = Nes::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
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
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0x0A, nes.A);
        assert_eq!(1, nes.C);
        assert_eq!(0, nes.V);
    }

    #[test]
    fn test_ADC_positive_overflow() {
        let code = vec![0xA9, 0x64, 0x69, 0x64]; 

        // if signed 0x64 (>0) + 0x64 (>0) = 0xc8 (< 0)
        let mut nes = Nes::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0xC8, nes.A);
        assert_eq!(0, nes.C);
        assert_eq!(1, nes.V);

    }

    #[test]
    fn test_AND() {
        let code = vec![0xA9, 0x64, 0x29, 0xA0]; 

        let mut nes = Nes::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0x20, nes.A);
        assert_eq!(0, nes.Z);
        assert_eq!(0, nes.N);


    }

    #[test]
    fn test_ASL_accumulator_nocarry() {
        let code = vec![0xA9, 0x64, 0x0A]; 
        let mut nes = Nes::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0xc8, nes.A);
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.N);
    }

    #[test]
    fn test_ASL_zeropage_with_carry() {
        let code = vec![0x06, 0x07]; 

        let mut nes = Nes::new(code);
        nes.memory.set(0x07, 0x84);
        assert_eq!(0x8000, nes.PC);
        nes.next().unwrap();

        assert_eq!(0x08, nes.memory.get(0x07 as usize));
        assert_eq!(0, nes.N); 
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.C);
    }

    #[test]
    fn test_bcc_not_taken() {
        let code = vec![0x90, 0x07]; // offset is +7. 
        let mut nes = Nes::new(code);
        nes.C = 1; // C not clear so do not take the branch.
        nes.next().unwrap();
        assert_eq!(0x8002, nes.PC);
    }

    #[test]
    fn test_bcc_taken_positive() {
        let code = vec![0x90, 0x07]; // offset is +7. 
        let mut nes = Nes::new(code);
        nes.C = 0; 
        nes.next().unwrap();
        assert_eq!(0x8009, nes.PC);
    }

    #[test]
    fn test_bcc_taken_negative() {
        let code = vec![0x90, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.C = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bcs_not_taken() {
        let code = vec![0xB0, 0x07]; // offset is +7. 
        let mut nes = Nes::new(code);
        nes.C = 0; // C clear so do not take the branch.
        nes.next().unwrap();
        assert_eq!(0x8002, nes.PC);
    }

    #[test]
    fn test_bcs_taken_positive() {
        let code = vec![0xB0, 0x07]; // offset is +7. 
        let mut nes = Nes::new(code);
        nes.C = 1; 
        nes.next().unwrap();
        assert_eq!(0x8009, nes.PC);
    }

    #[test]
    fn test_bcs_taken_negative() {
        let code = vec![0xB0, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.C = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_beq() {
        let code = vec![0xF0, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.Z = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bnq() {
        let code = vec![0xD0, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.Z = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }
    
    #[test]
    fn test_bmi() {
        let code = vec![0x30, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.N = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bpl() {
        let code = vec![0x10, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.N = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }
    
    #[test]
    fn test_bvc() {
        let code = vec![0x50, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.V = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bvs() {
        let code = vec![0x70, 0xF9]; // offset is -7. 
        let mut nes = Nes::new(code);
        nes.V = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bit_test_zeroflag() {
       let code = vec![0x24, 0x02]; // Bit test for zero page location
       let mut nes = Nes::new(code);

       // this should set the overflow, negative and zero flag.
       nes.memory.set(0x02, 0xF4); // '0b11110101'
       nes.A = 0x02;

       nes.next().unwrap();
       assert_eq!(1, nes.Z);
       assert_eq!(1, nes.N);
       assert_eq!(1, nes.V);
    }

    #[test]
    fn test_bit_test_notneg() {
       let code = vec![0x24, 0x02]; // Bit test for zero page location
       let mut nes = Nes::new(code);

       // this should set the overflow, negative and zero flag.
       nes.memory.set(0x02, 0x75); // '0b01110101'
       nes.A = 0x04;

       nes.next().unwrap();
       assert_eq!(0, nes.Z);
       assert_eq!(0, nes.N);
       assert_eq!(1, nes.V);
    }

    #[test]
    fn test_clear_carry() {
       let code = vec![0x18];
       let mut nes = Nes::new(code);
       nes.C = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.C);
    }
    
    #[test]
    fn test_clear_decimal() {
       let code = vec![0xD8];
       let mut nes = Nes::new(code);
       nes.D = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.D);
    }

    #[test]
    fn test_clear_interrupt() {
       let code = vec![0x58];
       let mut nes = Nes::new(code);
       nes.I = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.I);
    }

    #[test]
    fn test_clear_overflow() {
       let code = vec![0xB8];
       let mut nes = Nes::new(code);
       nes.V = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.V);
    }

}


