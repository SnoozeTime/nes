// Behaviour of PPU register is quite special. For example, when reading PPUSTATUS,
// the vblank flag will be cleared. In order to avoid cluttering the logic in 
// Memory.rs, I'll gather all the ppu register behaviour here.


#[allow(non_snake_case)]
pub enum RegisterType {
    PPUCTRL,
    PPUMASK, 
PPUSTATUS,
    OAMADDR,
    OAMDATA,
    PPUSCROLL,
    PPUADDR, 
    PPUDATA, 
    OADDMA,
}

impl RegisterType {
    pub fn value(&self) -> usize {

    match self {
    RegisterType::PPUCTRL => 0x2000,
    RegisterType::PPUMASK => 0x2001,
    RegisterType::PPUSTATUS => 0x2002,
    RegisterType::OAMADDR => 0x2003,
    RegisterType::OAMDATA => 0x2004,
    RegisterType::PPUSCROLL => 0x2005,
    RegisterType::PPUADDR => 0x2006,
    RegisterType::PPUDATA => 0x2007,
    RegisterType::OADDMA => 0x4014,
    }
    }
}


