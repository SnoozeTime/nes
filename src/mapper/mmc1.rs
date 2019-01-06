use serde_derive::{Serialize, Deserialize};
use super::{Mirroring, Mapper};
use crate::rom;

// MMC1 is mapper 1. Banks are switcheable. Writing to addresses
// $8000-$FFFF will actually change the cardridge registers.
// Good reference here: http://nesdev.com/mmc1.txt
#[derive(Debug, Serialize, Deserialize)]
pub struct Mmc1 {

    // Banks.
    // --------------------

    //  32KB bank number	|    16KB bank numbers
    //	------------------------|------------------------
    //               0          |          0,1
    //               1		|          2,3
    //               2		|          4,5
    //              ...		|          ...
    prg_rom_banks: Vec<Vec<u8>>, // 16kb for each element
    chr_rom_banks: Vec<Vec<u8>>, // 4kb for each element.

    prg_low_area_idx: usize,
    prg_high_area_idx: usize,
    chr_low_area_idx: usize,
    chr_high_area_idx: usize,

    // Loading register. Need to write 5 times to this before loading to
    // internal registers.
    // 0b10000 means empty. Once 1 is shifted in last position, loading_reg is full
    // and at next write, it will be used to write to the internal registers.
    loading_reg: u8,

    // Internal Registers
    // ------------------

    // written via $8000-$9FFF
    // switch between various MMC states
    // bit 0: Toggle mirroring. 0 = vertical, 1 = horizontal
    // bit 1: Toggle between H/V and "one screen mirroring". 0 = one screen
    // bit 2: Toggle between low area and high area for PRG rom switching.
    //        0 = high prgrom switching, 1 = low area.
    //        low area refers to $8000-$BFFF and high are refers to $C000-$FFFF
    // bit 3: toggle between 16kb and 32kb prg bank switching
    //        0 = 32kb, 1 = 16kb
    //        Overrides bit 2
    // bit 4: Sets 8kb or 4kb CHRRom switching (pattern tables)
    //        0 = 8kb, 1 = 4kb
    //
    reg0: u8,

    // written via $A000-$BFFF
    // to switch chrrom pages
    reg1: u8,

    // written via $C000-$DFFF
    // to switch chrrom pages
    reg2: u8,

    // written via $E000-$FFFF
    // to switch prgrom pages
    reg3: u8,
}

impl Mapper for Mmc1 {

    fn read_prg(&self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                self.prg_rom_banks[self.prg_low_area_idx][addr]
            },
            0xC000..=0xFFFF => {
                self.prg_rom_banks[self.prg_high_area_idx][addr]
            },
            _ => 0,
        }
    }

    // Writing to PRG will actually write to the registers.
    fn write_prg(&mut self, addr: usize, value: u8) {

        // TODO reset.

        if self.is_loading_reg_full() {

            let v = (value & 1) << 7;
            self.loading_reg = v | (self.loading_reg >> 1);

            let value_to_load = (self.loading_reg & 0b11111000) >> 3;
            match addr {
                0x8000..=0x9FFF => {
                    self.reg0 = value_to_load;
                },
                0xA000..=0xBFFF => {
                    self.reg1 = value_to_load;
                },
                0xC000..=0xDFFF => {
                    self.reg2 = value_to_load;
                },
                0xE000..=0xFFFF => {
                    self.reg3 = value_to_load;
                },
                _ => {}
            }

            self.reset_loading_reg();
        } else {
            let v = (value & 1) << 7;
            self.loading_reg = v | (self.loading_reg >> 1);
        }

    }

    // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
    fn read_chr(&self, addr: usize) -> u8 {
        match addr {
            0x0000..=0x0FFF => {
                self.chr_rom_banks[self.chr_low_area_idx][addr]
            },
            0x1000..=0x1FFF => {
                self.chr_rom_banks[self.chr_high_area_idx][addr]
            },
            _ => 0,
        }
    }

    fn write_chr(&mut self, addr: usize, value: u8) {
        match addr {
            0x0000..=0x0FFF => {
                self.chr_rom_banks[self.chr_low_area_idx][addr] = value;
            },
            0x1000..=0x1FFF => {
                self.chr_rom_banks[self.chr_high_area_idx][addr] = value;
            },
            _ => {},
        }
    }

    fn get_chr(&self) -> &[u8] {
        &self.chr_rom_banks[0]
    }
    
    fn get_mirroring(&self) -> Mirroring {
        // bit 0: Toggle mirroring. 0 = vertical, 1 = horizontal
        // bit 1: Toggle between H/V and "one screen mirroring". 0 = one screen
        if self.reg0 & 0b10 == 0 {
            Mirroring::ONE_SCREEN
        } else {
            if self.reg0 & 0b1 == 1 {
                Mirroring::HORIZONTAL
            } else {
                Mirroring::VERTICAL
            }
        }
    }

}

impl Mmc1 {

    pub fn new() -> Mmc1 {
        Mmc1 {
            chr_rom_banks: Vec::new(),
            prg_rom_banks: Vec::new(),
            chr_low_area_idx: 0,
            chr_high_area_idx: 0,
            prg_low_area_idx: 0,
            prg_high_area_idx: 0,
            loading_reg: 0b10000000,
            reg0: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
        }
    }

    fn is_loading_reg_full(&self) -> bool {
        self.loading_reg & 0x8 == 0x8
    }

    fn reset_loading_reg(&mut self) {
        self.loading_reg = 0x80;
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    // Non obfuscated means that the addr to write for reg0 is always 0x8000
    #[test]
    fn write_to_register0_non_obfuscated() {

        let mut mmc1 = Mmc1::new();
        let mut a = 0b11010;
        mmc1.write_prg(0x8000, a); 
        assert_eq!(0b01000000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0x8000, a); 
        assert_eq!(0b10100000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0x8000, a); 
        assert_eq!(0b01010000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0x8000, a); 
        assert_eq!(0b10101000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(true, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0x8000, a); 
        assert_eq!(0b10000000, mmc1.loading_reg);
        assert_eq!(0b11010, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());
    }

    // Obfuscated means that only the last addr written to for reg0 is 0x8000
    #[test]
    fn write_to_register0_obfuscated() {


        let mut mmc1 = Mmc1::new();
        let mut a = 0b11010;
        mmc1.write_prg(0xfe12, a); 
        assert_eq!(0b01000000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0xa123, a); 
        assert_eq!(0b10100000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0xC999, a); 
        assert_eq!(0b01010000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0x812A, a); 
        assert_eq!(0b10101000, mmc1.loading_reg);
        assert_eq!(0, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(true, mmc1.is_loading_reg_full());

        a >>= 1;
        mmc1.write_prg(0x8FFF, a); 
        assert_eq!(0b10000000, mmc1.loading_reg);
        assert_eq!(0b11010, mmc1.reg0);
        assert_eq!(0, mmc1.reg1);
        assert_eq!(0, mmc1.reg2);
        assert_eq!(0, mmc1.reg3);
        assert_eq!(false, mmc1.is_loading_reg_full());
    }

    #[test]
    fn test_mirroring() {
        let mut mmc1 = Mmc1::new();
        mmc1.reg0 = 0;
        assert_eq!(Mirroring::ONE_SCREEN, mmc1.get_mirroring());
        mmc1.reg0 = 1;
        assert_eq!(Mirroring::ONE_SCREEN, mmc1.get_mirroring());
        mmc1.reg0 = 2;
        assert_eq!(Mirroring::VERTICAL, mmc1.get_mirroring());
        mmc1.reg0 = 3;
        assert_eq!(Mirroring::HORIZONTAL, mmc1.get_mirroring());
    }

}
