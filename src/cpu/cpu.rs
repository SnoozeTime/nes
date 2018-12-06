use super::instructions::Instruction;
use rom;
use super::memory::*;

#[allow(non_snake_case)] // PC, SP ... are names in the specs.
pub struct Cpu {

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
    V: u8, // Overflow
    N: u8, // negative

    cycles: u64, // current number of cycles executed by the cpu.
}

impl std::fmt::Debug for Cpu {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let p = self.flags_to_u8_debug();
        write!(f, "A:{:x} X:{:x} Y:{:x} P:{:x} SP:{:X} CYC:{}", self.A, self.X, self.Y, p, self.SP, (3*self.cycles) % 341) // 3*cycles to get the APU cycles
    }
}

impl Cpu {
   
    // will create a new NES with the given code.
    pub fn new(code: Vec<u8>) -> Cpu {
        Cpu {
            memory: Memory::new(code),
            PC: 0x8000,// TODO set the correct
            SP: 0xFD, 
            A: 0,
            X: 0,
            Y: 0,
            C: 0,
            Z: 0,
            I: 0,
            D: 0,
            V: 0,
            N: 0,
            cycles: 0,
        }
    }

    pub fn create(ines: &rom::INesFile) -> Cpu {
        let memory = Memory::create(ines).expect("Problem with INES");
        Cpu {
            memory,
            PC: 0x8000,
            SP: 0xFD,
            A: 0,
            X: 0,
            Y: 0,
            C: 0,
            Z: 0,
            I: 0,
            D: 0,
            V: 0,
            N: 0,
            cycles: 0,
        }
    }

    pub fn get_acc(&self) -> u8 {
        self.A
    }

    pub fn get_regx(&self) -> u8 {
        self.X
    }

    pub fn get_regy(&self) -> u8 {
        self.Y
    }

    pub fn get_pc(&self) -> u16 {
        self.PC
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.PC = pc;
    }

    fn push(&mut self, value: u8) {
        let addr = 0x0100 + (self.SP as u16);
        self.memory.set(addr as usize, value);
        self.SP -= 1;
    }

    fn pull(&mut self) -> u8 {
        self.SP += 1;
        let addr = 0x0100 + (self.SP as u16);
        self.memory.get(addr as usize)
    }

    // used to push flags to the stacks.
    fn flags_to_u8(&self) -> u8 {
        // http://wiki.nesdev.com/w/index.php/Status_flags
        let b = ((self.N as u8) << 7)
            + ((self.V as u8) << 6)
            + (1 << 5) + (1 << 4) // always. ignored when pulling
            + ((self.D as u8) << 3)
            + ((self.I as u8) << 2)
            + ((self.Z as u8) << 1)
            + (self.C as u8);
        b
    }

    pub fn flags_to_u8_debug(&self) -> u8 {

        // http://wiki.nesdev.com/w/index.php/Status_flags
        let b = ((self.N as u8) << 7)
            + ((self.V as u8) << 6)
            + (1 << 5) // this is to match with nestest log
            + ((self.D as u8) << 3)
            + ((self.I as u8) << 2)
            + ((self.Z as u8) << 1)
            + (self.C as u8);
        b
    }


    fn u8_to_flags(&mut self, b: u8) {
        self.N = (b >> 7) & 0x1 as u8;
        self.V = (b >> 6) & 0x1 as u8;
        self.D = (b >> 3) & 0x1 as u8;
        self.I = (b >> 2) & 0x1 as u8;
        self.Z = (b >> 1) & 0x1 as u8;
        self.C = (b >> 0) & 0x1 as u8;
    }

    pub fn next(&mut self) -> Result<u8, &'static str> {

        let instruction = Instruction::decode(self);
        println!("{:?}\t{: <100?}", instruction, &self);

        let mut again_extra_cycles: u8 = 0;
        match &instruction {
            Instruction::ADC(_, addressing, _length) => {
                // http://www.6502.org/tutorials/vflag.html
                // A,Z,C,N,V = A+M+C
                // ADC can be used both with signed and unsigned numbers.
                // 
                let rhs = addressing.fetch(&self.memory);
                self.adc(rhs);
            },
            Instruction::SBC(_, addressing, _) => {
                let rhs = addressing.fetch(&self.memory);
                self.adc(!rhs);
            },
            Instruction::CMP(_, addressing, _) => {
                let m = addressing.fetch(&self.memory);
                let (result, overflow) = self.A.overflowing_sub(m);
                if overflow {
                    self.C = 0;
                } else {
                    self.C = 1;
                }
                self.set_result_flags(result);
            },
            Instruction::CPX(_, addressing, _) => {
                let m = addressing.fetch(&self.memory);
                let (result, overflow) = self.X.overflowing_sub(m);
                 if overflow {
                    self.C = 0;
                } else {
                    self.C = 1;
                }
                self.set_result_flags(result);
            },
            Instruction::CPY(_, addressing, _) => {
                let m = addressing.fetch(&self.memory);
                let (result, overflow) = self.Y.overflowing_sub(m);
                if overflow {
                    self.C = 0;
                } else {
                    self.C = 1;
                }
                self.set_result_flags(result);
            },
            Instruction::AND(_, addressing, _length) => {
                let result = self.A & addressing.fetch(&self.memory);
                self.set_result_flags(result);
                self.A = result;
            },
            Instruction::ASL(_, addressing, _length) => {
                let shifted: u16 = (addressing.fetch(&self.memory) as u16) << 1;
                let result = (shifted & 0xFF) as u8;
                self.C = (shifted >> 8) as u8;

                match &addressing.mode_type() {
                    AddressingModeType::Accumulator => self.A = result,
                    _ => addressing.set(&mut self.memory, result),
                }
                self.set_result_flags(result);
            },
            Instruction::LSR(_, addressing, _length) => {
                let operand = addressing.fetch(&self.memory);
                self.C = operand & 1;
                let result = operand >> 1;
                match &addressing.mode_type() {
                    AddressingModeType::Accumulator => self.A = result,
                    _ => addressing.set(&mut self.memory, result),
                }
                self.set_result_flags(result);
            },
            Instruction::ROL(_, addressing, _) => {
                let shifted: u16 = (addressing.fetch(&self.memory) as u16) << 1;
                let result = (shifted & 0xFF) as u8 | (self.C & 1);
                self.C = (shifted >> 8) as u8;

                match &addressing.mode_type() {
                    AddressingModeType::Accumulator => self.A = result,
                    _ => addressing.set(&mut self.memory, result),
                }
                self.set_result_flags(result);

            },
            Instruction::ROR(_, addressing, _) => {
                let operand = addressing.fetch(&self.memory);
                let result = operand >> 1 | (self.C << 7);
                self.C = operand & 1;
                match &addressing.mode_type() {
                    AddressingModeType::Accumulator => self.A = result,
                    _ => addressing.set(&mut self.memory, result),
                }
                self.set_result_flags(result);

            },
            // -------------------------------------
            // Jumps
            // ----------------------------------
            Instruction::JMP(_, addressing, _length) => {
                self.PC = addressing.fetch16(&self.memory);
            },
            Instruction::JSR(_, addressing, _) => {
                let return_addr = self.PC - 1;
                self.push(((return_addr & 0xFF00) >> 8) as u8);
                self.push((return_addr & 0xFF) as u8);
                self.PC = addressing.fetch16(&self.memory);
            },
            Instruction::RTS(_, _, _) => {
                let lsb = self.pull();
                let msb = self.pull();
                self.PC = lsb as u16 + ((msb as u16) << 8) + 1;
            },

            // ----------------------------------------
            // branches
            // ----------------------------------------
            Instruction::BCC(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.C == 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    // Carry clear let's take the branch.
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    // we do one byte offset (0xFF max) so if the upper
                    // bytes are not the same it means we crossed a page.
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;
                }
            },
            Instruction::BCS(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.C != 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;

                }
            },

            Instruction::BEQ(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.Z != 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;

                }
            },
            Instruction::BIT(_, addressing, _length) => {
                let to_test = addressing.fetch(&self.memory);
                let result = to_test & self.A;
                // set Z if to_test & A == 0
                if (result) == 0 {
                    self.Z = 1;
                } else {
                    self.Z = 0;
                }

                self.V = (to_test >> 6) & 0x1;
                self.N = (to_test >> 7) & 0x1;

//                println!("BIT {:x} & {:x} = {:x}", self.A, to_test, result);
                
            },
            Instruction::EOR(_, addressing, _length) => {
                let operand = addressing.fetch(&self.memory);
                let result = self.A ^ operand;
                self.set_result_flags(result);
                self.A = result;
            },
            Instruction::ORA(_, addressing, _length) => {
                let result = self.A | addressing.fetch(&self.memory);
                self.set_result_flags(result);
                self.A = result;
            },

            // INCREMENTS AND DECREMENTS
            Instruction::INC(_, addressing, _cycles) => {
                let result = addressing.fetch(&self.memory).wrapping_add(1);
                self.set_result_flags(result);
                addressing.set(&mut self.memory, result);
            },
            Instruction::INX(_, _addressing, _cycles) => {
                // TODO Wrapping add?
                let result = self.X.wrapping_add(1);
                self.set_result_flags(result);
                self.X = result;
            },
            Instruction::INY(_, _addressing, _cycles) => {
                let result = self.Y.wrapping_add(1);
                self.set_result_flags(result);
                self.Y = result;
            },
            Instruction::DEC(_, addressing, _cycles) => {
                let result = addressing.fetch(&self.memory).wrapping_sub(1);
                self.set_result_flags(result);
                addressing.set(&mut self.memory, result);
            },
            Instruction::DEX(_, _addressing, _cycles) => {
                let result = self.X.wrapping_sub(1);
                self.set_result_flags(result);
                self.X = result;
            },
            Instruction::DEY(_, _addressing, _cycles) => {
                let result = self.Y.wrapping_sub(1);
                self.set_result_flags(result);
                self.Y = result;
            },           
            Instruction::BMI(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.N != 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;
                }
            },
            Instruction::BNE(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.Z == 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;

                }
            },
            Instruction::BPL(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.N == 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;

                }
            },
            Instruction::BVC(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.V == 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;

                }
            },
            Instruction::BVS(_, addressing, _lenght) => {
                let offset = addressing.fetch(&self.memory);
                if self.V != 0 { 
                    let mut cycles = 1;
                    let original_pc = self.PC;
                    if (offset & 0x80) == 0x80 {
                        // negative.
                        self.PC -= 0x100 - offset as u16;
                    } else {
                        self.PC += offset as u16;
                    }
                    if (original_pc >> 8) != (self.PC >> 8) {
                        cycles += 1;
                    }
                    again_extra_cycles += cycles;
                }

            },

            Instruction::CLC(_, _, _length) => {
                self.C = 0;
            },
            Instruction::CLD(_, _, _length) => {
                self.D = 0;
            },
            Instruction::CLI(_, _, _length) => {
                self.I = 0;
            },
            Instruction::CLV(_, _, _length) => {
                self.V = 0;
            },
            Instruction::SEC(_, _, _) => {
                self.C = 1;
            },
            Instruction::SED(_,_,_) => {
                self.D = 1;
            },
            Instruction::SEI(_,_,_) => {
                self.I = 1;
            },
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
            Instruction::STA(_, addressing, _length) => {
                addressing.set(&mut self.memory, self.A);
            },
            Instruction::STX(_, addressing, _length) => {
                addressing.set(&mut self.memory, self.X);
            },
            Instruction::STY(_, addressing, _length) => {
                addressing.set(&mut self.memory, self.Y);
            },
            // transfer instructions
            Instruction::TAX(_, _, _length) => {
                let result = self.A;
                self.X = result;
                self.set_result_flags(result);
            },
            Instruction::TAY(_, _, _length) => {
                let result = self.A;
                self.Y = result;
                self.set_result_flags(result);
            },
            Instruction::TXA(_, _, _length) => {
                let result = self.X;
                self.A = result;
                self.set_result_flags(result);
            },
            Instruction::TYA(_, _, _length) => {
                let result = self.Y;
                self.A = result;
                self.set_result_flags(result);

            },

            // stack instructions
            Instruction::TSX(_, _, _length) => {
                let result = self.SP;
                self.X = result;
                self.set_result_flags(result);
            },
            Instruction::TXS(_, _, _length) => {
                self.SP = self.X;
            },
            Instruction::PHA(_, _, _length) => {
                let to_push = self.A;
                self.push(to_push);
            },
            Instruction::PLA(_, _, _length) => {
                let result = self.pull();
                self.A = result;
                self.set_result_flags(result);
            },
            Instruction::PHP(_, _, _length) => {
                let to_push = self.flags_to_u8();
                self.push(to_push);
            },
            Instruction::PLP(_, _, _length) => {
                let result = self.pull();
                self.u8_to_flags(result);
            },
            Instruction::BRK(_,_,_) => {
                // IRQ interrupt vector is at $FFFE/F
                
                // push PC and Status flag
                let pc = self.PC;
                self.push(((pc & 0xFF00) >> 8) as u8);
                self.push((pc & 0xFF) as u8);
                let flags = self.flags_to_u8();
                self.push(flags);

                let lsb = self.memory.get(0xFFFE-1 as usize) as u16;
                let msb = self.memory.get(0xFFFF-1 as usize) as u16;
                self.PC = lsb + (msb << 8);
            },
            Instruction::RTI(_,_,_) => {
                let flags = self.pull();
                self.u8_to_flags(flags);
                let lsb = self.pull();
                let msb = self.pull();
                self.PC = lsb as u16 + ((msb as u16) << 8);// + 1;
            },
            Instruction::NOP(_,_,_) 
                | Instruction::DOP(_,_,_) 
                | Instruction::TOP(_,_,_) => {
                // nothing to see here.
            },

            // ----------------------------------------------
            // Unofficial opcodes
            // ---------------------------------------------
            Instruction::ANC(_, addressing, _) => {
                let result = self.A & addressing.fetch(&self.memory);
                self.set_result_flags(result);
                self.C = self.N;
            },
            Instruction::ARR(_, addressing, _) => {
                let operand = addressing.fetch(&self.memory);

                let and_result = operand & self.A;
                let result = and_result >> 1 | (self.C << 7);
                self.C = and_result & 1;

                let sixth_bit = result >> 6 & 1; 
                let fifth_bit = result >> 5 & 1;
                match (sixth_bit, fifth_bit) {
                    (1, 1) => {
                        self.C = 1;
                        self.V = 0;
                    },
                    (0, 0) => {
                        self.C = 0;
                        self.V = 0;
                    },
                    (0, 1) => {
                        self.V = 1;
                        self.C = 0;
                    },
                    (1, 0) => {
                        self.V = 1;
                        self.C = 1;
                    }
                    (_, _) => {//uh
                    }
                }
                self.set_result_flags(result);
                self.A = result;
            },
            Instruction::ALR(_, addressing, _) => {
                let operand = addressing.fetch(&self.memory);
                let before_shift = self.A & operand;
                self.C = before_shift & 1;
                let result = before_shift >> 1;
                self.A = result;
                self.set_result_flags(result);

            },
            Instruction::LAX(_, addressing, _) => {
                let operand = addressing.fetch(&self.memory);
                self.X = operand;
                self.A = operand;
                self.set_result_flags(operand);
            },
            Instruction::SAX(_, addressing, _) => {
                let result = self.A & self.X;
                // http://www.ffd2.com/fridge/docs/6502-NMOS.extra.opcodes
                // self.set_result_flags(result);
                addressing.set(&mut self.memory, result);
            },
            Instruction::DCP(_, addressing, _) => {
                let operand = addressing.fetch(&self.memory);
                let result = operand.wrapping_sub(1);
                addressing.set(&mut self.memory, result);
                let (test_result, overflow) = self.A.overflowing_sub(result);
                if overflow {
                    self.C = 0;
                } else {
                    self.C = 1;
                }
                self.set_result_flags(test_result);

            },
            Instruction::ISC(_, addressing, _) => {
                // INC
                let result = addressing.fetch(&self.memory).wrapping_add(1);
                self.set_result_flags(result);
                addressing.set(&mut self.memory, result);

                // SBC
                self.adc(!result);
            },
            Instruction::RLA(_, addressing, _) => {

                let shifted: u16 = (addressing.fetch(&self.memory) as u16) << 1;
                let result = (shifted & 0xFF) as u8 | (self.C & 1);
                self.C = (shifted >> 8) as u8;
                addressing.set(&mut self.memory, result);

                let and_result = self.A & result;
                self.set_result_flags(and_result);
                self.A = and_result;
            },
            Instruction::RRA(_, addressing, _) => {
                // ROR then ADC.
                let operand = addressing.fetch(&self.memory);
                let result = operand >> 1 | (self.C << 7);
                self.C = operand & 1;
                addressing.set(&mut self.memory, result);
                self.set_result_flags(result);
                
                // max value is 0x1FF. There is carry if > 0xFF.
                self.adc(result);
            },
            Instruction::SLO(_, addressing, _) => {
                // shift left one bit in memory
                let shifted: u16 = (addressing.fetch(&self.memory) as u16) << 1;
                let result = (shifted & 0xFF) as u8;
                self.C = (shifted >> 8) as u8;
                addressing.set(&mut self.memory, result);

                // OR With A.
                let or_result = self.A | result;
                self.A = or_result;
                self.set_result_flags(or_result);
            },
            Instruction::SRE(_, addressing, _) => {
                // Shift right.
                let operand = addressing.fetch(&self.memory);
                self.C = operand & 1;
                let result = operand >> 1;
                addressing.set(&mut self.memory, result);
                
                // EOR With A 
                let eor_result = self.A ^ result;
                self.set_result_flags(eor_result);
                self.A = eor_result;
            },
            Instruction::UNKNOWN(_,_) => {}
        };

        let total_cycles = instruction.get_cycles() + again_extra_cycles;
        self.cycles += total_cycles as u64;
        Ok(total_cycles)
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
    
    // Get next instruction and increment PC
    pub fn advance(&mut self) -> u8 {
        let code = self.memory.get(self.PC as usize);
        self.PC += 1;
        code
    }

    fn adc(&mut self, rhs: u8) {
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
        if ((rhs ^ self.A) >> 7 == 0) && ((rhs ^ result) >> 7 == 1) {
            self.V = 1;
        } else {
            self.V = 0;
        }
        self.A = result;
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

        let mut nes = Cpu::new(code);
        assert_eq!(0x8000, nes.PC);
        nes.next().unwrap();

        assert_eq!(0x8002, nes.PC);
        assert_eq!(0x36, nes.A);
    }

    #[test]
    fn test_LDA_zeropage_negative() {
        let code = vec![0xA5, 0x06]; 

        let mut nes = Cpu::new(code);
        nes.memory.set(0x06, 0x84);
        nes.next().unwrap();

        assert_eq!(0x84, nes.A);
        assert_eq!(1, nes.N); 
    }

    #[test]
    fn test_LDA_absolute_processor_zero() {
        let code = vec![0xAD, 0x06, 0xA3]; 

        let mut nes = Cpu::new(code);
        nes.memory.set(0xA306, 0x00);
        nes.next().unwrap();

        assert_eq!(0x00, nes.A);
        assert_eq!(0x01, nes.Z);
    }

    #[test]
    fn test_LDX_indexed_zp() {
        let code = vec![0xB6, 0x04];
        let mut nes = Cpu::new(code);
        nes.Y = 0x02;
        nes.memory.set(0x06, 0x0A);
        nes.next().unwrap();
        assert_eq!(0x0A, nes.X);
    }

    #[test]
    fn test_LDY_indexed_absolute() {
        let code = vec![0xBC, 0x06, 0xA3]; 
    
        let mut nes = Cpu::new(code);
        nes.X = 0x02;
        nes.memory.set(0xA308, 0x11);
        nes.next().unwrap();

        assert_eq!(0x11, nes.Y);
    }

    #[test]
    fn test_adc_without_carry() {
        // now carry, no overflow.
        let code = vec![0xA9, 0x01, 0x69, 0x10]; // A should be 0x11

        let mut nes = Cpu::new(code);
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
        let mut nes = Cpu::new(code);
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
        let mut nes = Cpu::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0xC8, nes.A);
        assert_eq!(0, nes.C);
        assert_eq!(1, nes.V);

    }

    #[test]
    fn test_AND() {
        let code = vec![0xA9, 0x64, 0x29, 0xA0]; 

        let mut nes = Cpu::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0x20, nes.A);
        assert_eq!(0, nes.Z);
        assert_eq!(0, nes.N);


    }

    #[test]
    fn test_ASL_accumulator_nocarry() {
        let code = vec![0xA9, 0x64, 0x0A]; 
        let mut nes = Cpu::new(code);
        nes.next().unwrap();
        nes.next().unwrap();
        assert_eq!(0xc8, nes.A);
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.N);
    }

    #[test]
    fn test_ASL_zeropage_with_carry() {
        let code = vec![0x06, 0x07]; 

        let mut nes = Cpu::new(code);
        nes.memory.set(0x07, 0x84);
        nes.next().unwrap();

        assert_eq!(0x08, nes.memory.get(0x07 as usize));
        assert_eq!(0, nes.N); 
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.C);
    }

    #[test]
    fn test_lsr_acc() {
        let code = vec![0x4A]; 
        let mut nes = Cpu::new(code);
        nes.A = 0x4B;
        nes.next().unwrap();

        assert_eq!(0x25, nes.A);
        assert_eq!(0, nes.N); 
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.C);
    }

    #[test]
    fn test_rol_acc() {
        let code = vec![0x2A]; 
        let mut nes = Cpu::new(code);
        nes.A = 0x4B;
        nes.C = 1;
        nes.next().unwrap();

        assert_eq!(0x97, nes.A);
        assert_eq!(1, nes.N); 
        assert_eq!(0, nes.Z);
        assert_eq!(0, nes.C);
    }

    #[test]
    fn test_ror_mem() {
        let code = vec![0x66, 0x02]; 
        let mut nes = Cpu::new(code);
        nes.memory.set(0x02, 0x4B);
        nes.C = 1;
        nes.next().unwrap();

        assert_eq!(0xa5, nes.memory.get(0x02));
        assert_eq!(1, nes.N); 
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.C);
    }

    #[test]
    fn test_bcc_not_taken() {
        let code = vec![0x90, 0x07]; // offset is +7. 
        let mut nes = Cpu::new(code);
        nes.C = 1; // C not clear so do not take the branch.
        nes.next().unwrap();
        assert_eq!(0x8002, nes.PC);
    }

    #[test]
    fn test_bcc_taken_positive() {
        let code = vec![0x90, 0x07]; // offset is +7. 
        let mut nes = Cpu::new(code);
        nes.C = 0; 
        nes.next().unwrap();
        assert_eq!(0x8009, nes.PC);
    }

    #[test]
    fn test_bcc_taken_negative() {
        let code = vec![0x90, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.C = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bcs_not_taken() {
        let code = vec![0xB0, 0x07]; // offset is +7. 
        let mut nes = Cpu::new(code);
        nes.C = 0; // C clear so do not take the branch.
        nes.next().unwrap();
        assert_eq!(0x8002, nes.PC);
    }

    #[test]
    fn test_bcs_taken_positive() {
        let code = vec![0xB0, 0x07]; // offset is +7. 
        let mut nes = Cpu::new(code);
        nes.C = 1; 
        nes.next().unwrap();
        assert_eq!(0x8009, nes.PC);
    }

    #[test]
    fn test_bcs_taken_negative() {
        let code = vec![0xB0, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.C = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_beq() {
        let code = vec![0xF0, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.Z = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bnq() {
        let code = vec![0xD0, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.Z = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }
    
    #[test]
    fn test_bmi() {
        let code = vec![0x30, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.N = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bpl() {
        let code = vec![0x10, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.N = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }
    
    #[test]
    fn test_bvc() {
        let code = vec![0x50, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.V = 0; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bvs() {
        let code = vec![0x70, 0xF9]; // offset is -7. 
        let mut nes = Cpu::new(code);
        nes.V = 1; 
        nes.next().unwrap();
        assert_eq!(0x7FFB, nes.PC);
    }

    #[test]
    fn test_bit_test_zeroflag() {
       let code = vec![0x24, 0x02]; // Bit test for zero page location
       let mut nes = Cpu::new(code);

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
       let mut nes = Cpu::new(code);

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
       let mut nes = Cpu::new(code);
       nes.C = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.C);
    }
    
    #[test]
    fn test_clear_decimal() {
       let code = vec![0xD8];
       let mut nes = Cpu::new(code);
       nes.D = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.D);
    }

    #[test]
    fn test_clear_interrupt() {
       let code = vec![0x58];
       let mut nes = Cpu::new(code);
       nes.I = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.I);
    }

    #[test]
    fn test_clear_overflow() {
       let code = vec![0xB8];
       let mut nes = Cpu::new(code);
       nes.V = 0x1;
       nes.next().unwrap();
       assert_eq!(0, nes.V);
    }

    #[test]
    fn test_store_A() {
        let code = vec![0x85, 0x04];
        let mut nes = Cpu::new(code);
        nes.A = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.memory.get(0x04));
    }

    #[test]
    fn test_store_X() {
        let code = vec![0x86, 0x04];
        let mut nes = Cpu::new(code);
        nes.X = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.memory.get(0x04));
    }

    #[test]
    fn test_store_Y() {
        let code = vec![0x84, 0x04];
        let mut nes = Cpu::new(code);
        nes.Y = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.memory.get(0x04));
    }


    #[test]
    fn test_transfer_A_to_X() {
        //TAX
        let code = vec![0xAA];
        let mut nes = Cpu::new(code);
        nes.A = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.X);
    }

    #[test]
    fn test_transfer_A_to_Y() {
        //TAY
        let code = vec![0xA8];
        let mut nes = Cpu::new(code);
        nes.A = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.Y);
    }

    #[test]
    fn test_transfer_X_to_A() {
        //TXA
        let code = vec![0x8A];
        let mut nes = Cpu::new(code);
        nes.X = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.A);
    }

    #[test]
    fn test_transfer_Y_to_A() {
        //TYA
        let code = vec![0x98];
        let mut nes = Cpu::new(code);
        nes.Y = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.Y);
    }

    #[test]
    fn test_transfer_X_to_SP() {
        //TXA
        let code = vec![0x9A];
        let mut nes = Cpu::new(code);
        nes.X = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.SP);
    }

    #[test]
    fn test_transfer_SP_to_X() {
        //TYA
        let code = vec![0xBA];
        let mut nes = Cpu::new(code);
        nes.SP = 0xF1;
        nes.next().unwrap();
        assert_eq!(0xF1, nes.X);
        assert_eq!(0x0, nes.Z);
        assert_eq!(0x1, nes.N);
    }

    #[test]
    fn test_stack_accumulator() {
        let code = vec![0x48, 0x68];//push pull
        let mut nes = Cpu::new(code);
        nes.A = 0x44;

        nes.next().unwrap();
        nes.A = 0x00;
        assert_eq!(0xFC, nes.SP);
        assert_eq!(0x44, nes.memory.get(0x01FD));

        nes.next().unwrap();
        assert_eq!(0xFD, nes.SP);
        assert_eq!(0x44, nes.A);
    }

    #[test]
    fn test_stack_processor_flag() {
        let code = vec![0x08, 0x28];//push pull
        let mut nes = Cpu::new(code);
        
        nes.C = 1;
        nes.Z = 1;
        nes.V = 1;
        nes.N = 0;
        nes.I = 0;

        nes.next().unwrap();
         
        nes.C = 0;
        nes.Z = 0;
        nes.V = 0;
        nes.N = 0;
        nes.I = 0;

        assert_eq!(0xFC, nes.SP);
        //assert_eq!(0x44, nes.memory.get(0x01FF));

        nes.next().unwrap();
        assert_eq!(0xFD, nes.SP);
        assert_eq!(1, nes.C);
        assert_eq!(1, nes.Z);
        assert_eq!(1, nes.V);
        assert_eq!(0, nes.N);
        assert_eq!(0, nes.I);
    }

    #[test]
    fn test_exclusive_eor() {
        let code = vec![0x49, 0x3];//push pull
        let mut nes = Cpu::new(code);
        nes.A = 0x6;
        nes.next().unwrap();
        assert_eq!(0x5, nes.A);
    }

    #[test]
    fn test_exclusive_ora() {
        let code = vec![0x09, 0x03];//push pull
        let mut nes = Cpu::new(code);
        nes.A = 0x06;
        nes.next().unwrap();
        assert_eq!(0x7, nes.A);
    }

    #[test]
    fn test_inc_dec_mem() {
        let code = vec![0xE6, 0x02, 0xC6, 0x02];
        let mut nes = Cpu::new(code);
        nes.next().unwrap();
        assert_eq!(1, nes.memory.get(0x02 as usize));
        nes.next().unwrap();
        
        assert_eq!(0, nes.memory.get(0x02 as usize));
    }


    #[test]
    fn test_inx_dex() {
        let code = vec![0xE8, 0xCA];
        let mut nes = Cpu::new(code);
        nes.next().unwrap();
        assert_eq!(1, nes.X);
        nes.next().unwrap();
        assert_eq!(0, nes.X);
    }

    #[test]
    fn test_iny_dey() {
        let code = vec![0xC8, 0x88];
        let mut nes = Cpu::new(code);
        nes.next().unwrap();
        assert_eq!(1, nes.Y);
        nes.next().unwrap();
        assert_eq!(0, nes.Y);
    }

    #[test]
    fn test_cmp_a_gt_m() {
        let code = vec![0xC9, 0x2];
        let mut nes = Cpu::new(code);
        nes.A = 0x05;
        nes.next().unwrap();

        assert_eq!(1, nes.C);
        assert_eq!(0, nes.Z);
    }

    #[test]
    fn test_cmp_a_eq_m() {
        let code = vec![0xC9, 0x2];
        let mut nes = Cpu::new(code);
        nes.A = 0x02;
        nes.next().unwrap();

        assert_eq!(1, nes.C);
        assert_eq!(1, nes.Z);

    }

    #[test]
    fn test_cmp_a_lt_m() {
        let code = vec![0xC9, 0x7];
        let mut nes = Cpu::new(code);
        nes.A = 0x05;
        nes.next().unwrap();

        assert_eq!(0, nes.C);
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.N);
    }
    
    #[test]
    fn test_cmp_x_gt_m() {
        let code = vec![0xE0, 0x2];
        let mut nes = Cpu::new(code);
        nes.X = 0x05;
        nes.next().unwrap();

        assert_eq!(1, nes.C);
        assert_eq!(0, nes.Z);
    }

    #[test]
    fn test_cmp_x_eq_m() {
        let code = vec![0xE0, 0x2];
        let mut nes = Cpu::new(code);
        nes.X = 0x02;
        nes.next().unwrap();

        assert_eq!(1, nes.C);
        assert_eq!(1, nes.Z);

    }

    #[test]
    fn test_cmp_x_lt_m() {
        let code = vec![0xE0, 0x7];
        let mut nes = Cpu::new(code);
        nes.X = 0x05;
        nes.next().unwrap();

        assert_eq!(0, nes.C);
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.N);
    }
    
    #[test]
    fn test_cmp_y_gt_m() {
        let code = vec![0xC0, 0x2];
        let mut nes = Cpu::new(code);
        nes.Y = 0x05;
        nes.next().unwrap();

        assert_eq!(1, nes.C);
        assert_eq!(0, nes.Z);
    }

    #[test]
    fn test_cmp_y_eq_m() {
        let code = vec![0xC0, 0x2];
        let mut nes = Cpu::new(code);
        nes.Y = 0x02;
        nes.next().unwrap();

        assert_eq!(1, nes.C);
        assert_eq!(1, nes.Z);

    }

    #[test]
    fn test_cmp_y_lt_m() {
        let code = vec![0xC0, 0x7];
        let mut nes = Cpu::new(code);
        nes.Y = 0x05;
        nes.next().unwrap();

        assert_eq!(0, nes.C);
        assert_eq!(0, nes.Z);
        assert_eq!(1, nes.N);
    }
    // -----------------------------------------------
    // Quick testing of unofficial opcodes.
    
    // Does AND #i, setting N and Z flags based on the result. Then it copies N (bit 7) to C
    #[test]
    fn test_anc() {
        let code = vec![0x0B, 0xFF];
        let mut nes = Cpu::new(code);
        nes.A = 0xC2; // negatif

        nes.next().unwrap();
        assert_eq!(1, nes.C);
        assert_eq!(1, nes.N);
        assert_eq!(0, nes.Z);
    }

    // AND X register with accumulator and store result in X
    #[test]
    fn test_axs() {

        let code = vec![0x87, 0x01];
        let mut nes = Cpu::new(code);
        nes.X = 0x12;
        nes.A = 0x46;
        nes.next().unwrap();
        assert_eq!(0x02, nes.memory.get(0x01));
        assert_eq!(0, nes.N);
        assert_eq!(0, nes.Z);
    }

    // AND byte with accumulator, then rotate one bit right in accu-mulator and
    // check bit 5 and 6:
    // If both bits are 1: set C, clear V.
    // If both bits are 0: clear C and V.
    // If only bit 5 is 1: set V, clear C.
    // If only bit 6 is 1: set C and V.
    // Status flags: N,V,Z,C
    #[test]
    fn test_arr() {
        let code = vec![0x6B, 0xD1];
        let mut nes = Cpu::new(code);
        nes.A = 0xFF;
        
        nes.next().unwrap();
        assert_eq!(1, nes.C);
        assert_eq!(0, nes.V);
        assert_eq!(0, nes.N);
        assert_eq!(0, nes.Z);
        assert_eq!(0x68, nes.A);
    }

    // ALR
    // This opcode ANDs the contents of the A register with an immediate value and 
    // then LSRs the result.
    #[test]
    fn test_alr() {
        let code = vec![0x4B, 0xD1];
        let mut nes = Cpu::new(code);
        nes.A = 0xc4;
        // AND is 0b11000000
        // Shift right -> 0b01100000 and C = 0
        nes.next().unwrap();
        assert_eq!(0, nes.C);
        assert_eq!(0, nes.N);
        assert_eq!(0, nes.Z);
        assert_eq!(0x60, nes.A);
    }

    #[test]
    fn test_lax() {
        let code = vec![0xA7, 0xD1];
        let mut nes = Cpu::new(code);
        nes.memory.set(0xD1, 0x54);

        nes.next().unwrap();
        
        assert_eq!(0x54, nes.A);
        assert_eq!(0x54, nes.X);
        assert_eq!(0, nes.N);
        assert_eq!(0, nes.Z);
    }

    #[test]
    fn test_sax() {
        let code = vec![0x87, 0xD1];
        let mut nes = Cpu::new(code);
        nes.X = 0x53;
        nes.A = 0x62;
        nes.next().unwrap();
        assert_eq!(0x42, nes.memory.get(0xD1));
    }

}


