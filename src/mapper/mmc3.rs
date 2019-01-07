use super::Mirroring;
use crate::rom::INesFile;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Mmc3 {
    bank: Vec<u8>
}


impl Mmc3 {

    pub fn read_prg(&self, addr: usize) -> u8 {
        0
    }

    // Writing to PRG will actually write to the registers.
    pub fn write_prg(&mut self, addr: usize, value: u8) {
    }

    // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
    pub fn read_chr(&self, addr: usize) -> u8 {
        0
    }

    pub fn write_chr(&mut self, addr: usize, value: u8) {
    }

    pub fn get_chr(&self, idx: usize) -> &[u8] {
       &self.bank 
    }
    
    pub fn get_mirroring(&self) -> Mirroring {
        Mirroring::HORIZONTAL
    }

    pub fn from(ines: &INesFile) -> Result<Mmc3, String> {
        Ok(Mmc3 { bank: Vec::new() })
    }
}
