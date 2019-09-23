use serde_derive::{Serialize, Deserialize};
use super::Mirroring;
use crate::rom::{INesFile};

#[derive(Debug, Serialize, Deserialize)]
pub struct Uxrom {

    // Banks.
    // --------------------

    // low area is switcheable, upper area is fixed
    prg_rom_banks: Vec<Vec<u8>>, // 16kb for each element

    // just RAM
    chr_rom_banks: Vec<Vec<u8>>, // 4kb for each element.

    prg_bank_idx: usize,
    mirroring: Mirroring,
}

impl Uxrom {

    pub fn read_prg(&self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                self.prg_rom_banks[self.prg_bank_idx][addr % 0x4000]
            },
            0xC000..=0xFFFF => {
                self.prg_rom_banks.last().unwrap()[addr % 0x4000]
            },
            _ => 0,
        }
    }

    // Writing to PRG will actually write to the registers.
    pub fn write_prg(&mut self, _addr: usize, value: u8) {
        self.prg_bank_idx = (value & 0xF) as usize;
    }

    // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
    pub fn read_chr(&self, addr: usize) -> u8 {
        match addr {
            0x0000..=0x0FFF => {
                self.chr_rom_banks[0][addr % 0x1000]
            },
            0x1000..=0x1FFF => {
                self.chr_rom_banks[1][addr % 0x1000]
            },
            _ => 0,
        }
    }

    pub fn write_chr(&mut self, addr: usize, value: u8) {
        match addr {
            0x0000..=0x0FFF => {
                self.chr_rom_banks[0][addr % 0x1000] = value;
            },
            0x1000..=0x1FFF => {
                self.chr_rom_banks[1][addr % 0x1000] = value;
            },
            _ => {},
        }
    }

    pub fn get_chr(&self, idx: usize) -> &[u8] {
        if idx == 0 {
            &self.chr_rom_banks[0]
        } else {
            &self.chr_rom_banks[1]
        }
    }
    
    pub fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }


    pub fn new() -> Uxrom {
        Uxrom {
            chr_rom_banks: Vec::new(),
            prg_rom_banks: Vec::new(),
            prg_bank_idx: 0,
            mirroring: Mirroring::HORIZONTAL,
        }
    }

    pub fn from(ines: &INesFile) -> Result<Uxrom, String> {
    
        let mut pages = Vec::new();
        for nb in 0..ines.get_prg_rom_pages() {
            let mut prg_page = vec![0; 0x4000];
            let rom_page = ines.get_prg_rom(nb+1)?;
            for (i, b) in rom_page.iter().enumerate() {
                prg_page[i] = *b;
            }
            pages.push(prg_page);
        }

        let mut pattern_table_pages = Vec::new();
        pattern_table_pages.push(vec![0; 0x1000]);
        pattern_table_pages.push(vec![0; 0x1000]);

        let mirroring = ines.get_mirroring();
        let prg_bank_idx = 0;
        Ok(Uxrom {
            chr_rom_banks: pattern_table_pages,
            prg_rom_banks: pages,
            prg_bank_idx,
            mirroring,
        }) 
    }

}
