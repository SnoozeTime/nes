use serde_derive::{Serialize, Deserialize};
use super::Mapper;
use crate::rom;

// NROM is mapper 0. Banks are not switcheable.
// it has either 1 or 2 16kb PRG ROMs and 1 8kb CHR-ROM (can be RAM...?)
#[derive(Debug, Serialize, Deserialize)]
pub struct Nrom {

    // PRGROM
    nb_page: usize, // either 1 or 2.
    // mapped to CPU $8000-$BFFF
    prg_rom_first: Vec<u8>, // size is 0x4000
    // mapped to CPU $C000-$FFFF
    prg_rom_last: Vec<u8>,

    // PPU pattern tables
    chr_rom: Vec<u8>,
}

impl Nrom {
    // empty NROM
    pub fn new() -> Nrom {
        Nrom {
            nb_page: 1,
            prg_rom_first: vec![0; 0x4000],
            prg_rom_last: vec![0; 0x4000],
            chr_rom: vec![0; 0x2000],
        }
    }

    pub fn from(ines: &rom::INesFile) -> Result<Nrom, String> {
        let page_nb = ines.get_prg_rom_pages();

        // ----------------------------------
        // First copy PRG ROM
        // ----------------------------------
        let mut prg_rom_first = vec![0; 0x4000];
        let mut prg_rom_last = vec![0; 0x4000];

        if page_nb == 1 {

            let page = ines.get_prg_rom(1)?;
            for (i, b) in page.iter().enumerate() {
                prg_rom_first[i] = *b;
                prg_rom_last[i] = *b;
            }
        } else if page_nb == 2 {
            let page = ines.get_prg_rom(1)?;
            for (i, b) in page.iter().enumerate() {
                prg_rom_first[i] = *b;
            }
            let page2 = ines.get_prg_rom(2)?;
            for (i, b) in page2.iter().enumerate() {
                prg_rom_last[i] = *b;
            }
        } else {
            return Err(String::from("NROM expect 1 or 2 PRG ROM pages"));
        }

        // ----------------------------------
        // Then copy pattern table
        // ----------------------------------
        let mut chr_rom = vec![0; 0x2000];
        if ines.get_chr_rom_pages() > 0 {
            let vrom = ines.get_chr_rom(1)?;
            for (i, b) in vrom.iter().enumerate() {
                chr_rom[i] = *b;
            }
        }

        Ok(Nrom {
            nb_page: page_nb,
            prg_rom_first,
            prg_rom_last,
            chr_rom,
        })
    }
}

impl Mapper for Nrom {

    fn read_prg(&self, addr: usize) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                self.prg_rom_first[addr % 0x4000]
            },
            0xC000..=0xFFFF => {
                self.prg_rom_last[addr % 0x4000]
            },
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: usize, value: u8) {
        match addr {
            0x8000..=0xBFFF => {
                self.prg_rom_first[addr % 0x4000] = value;
            },
            0xC000..=0xFFFF => {
                self.prg_rom_last[addr % 0x4000] = value;
            },
            _ => {}
        }
    }

    // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
    fn read_chr(&self, addr: usize) -> u8 {
        self.chr_rom[addr]
    }

    fn write_chr(&mut self, addr: usize, value: u8) {
        self.chr_rom[addr] = value;
    }

}


