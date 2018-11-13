
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
pub struct Nes {

    code: Vec<u8>, // contains the instructions.
   
    // RAM ! Layout is:
    // 0x0000-0x0100 :zero page random-access memory.
    // 0x0100-0x0200: Stack
    // 0x0200-0x0800: RAM
    // 0x0800-0x2000: Mirrors of previous (TODO Should we implement mirrors?)
    RAM: Vec<u8>,

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
    pub fn new(code: Vec<u8>) -> Nes {
        Nes {
            code,
            RAM: vec![0; 0x800],
            PC: 0,
            SP: 0xFF, 
            A: 0,
            X: 0,
            Y: 0,
            P: 0,
        }
    }
    
    // Will fetch/decode/execute the next operation. This is a BIG switch statement.
    pub fn next(&mut self) -> Result<(), Box<std::error::Error>> {
    
        let opcode = self.code[self.PC as usize];
        self.PC += 1;


        // let instruction_result = match { .... }
        match opcode {

        // -------------------------------------
        // AND operations
        // ------------------------------------
        0x29 => {
            let zerop_loc = self.advance();
            println!("AND #operand - Zero page at 0x{:x}", zerop_loc);
        },
        0x25 => println!("AND operand"),

        // ------------------------------------
        // LoaD Accumulator LDA
        // Affect flags S, Z
        // -----------------------------------
        0xA9 => {
            let operand = self.advance();
            self.A = operand;
        },
        // -------------------------------------
        // STX - Store X
        // Affect flags: None
        // ------------------------------------
        0x86 => {
            // Zero page
            let zerop_loc = self.advance();
            println!("STX Operand - Store X at 0x{:x}", zerop_loc);
        },
        0x96 => {
            // Indexing zero page.
            let zerop_loc = self.advance();
            println!("STX Operand,Y - Store X at 0x{:x}+Y", zerop_loc);
        },
        0x8E => {
            // absolute indexing
            let lsb = self.advance();
            let msb = self.advance();
            let loc = ((msb as u16) << 8) | (lsb as u16);
            println!("STX Operand - Absolute. Store X at 0x{:x}", loc);
        },

        _ => print!(""),
        }
        Ok( () )
    }

    // Get next instruction and increment PC
    fn advance(&mut self) -> u8 {
        let code = self.code[self.PC as usize];
        self.PC += 1;
        code
    }
}

#[cfg(test)]
mod tests {

    // get names from outer scope.
    use super::*;


    #[test]
    fn test_LDA_immediate() {
        // Load accumulator. Immediate addressing
        let code = vec![0xA9, 0x36]; 

        let mut nes = Nes::new(code);
        assert_eq!(0, nes.PC);
        nes.next();

        assert_eq!(2, nes.PC);
        assert_eq!(0x36, nes.A);
    }

}
