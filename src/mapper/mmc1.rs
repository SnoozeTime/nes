use serde_derive::{Serialize, Deserialize};
use super::Mapper;
use crate::rom;

// MMC1 is mapper 1. Banks are switcheable. Writing to addresses
// $8000-$FFFF will actually change the cardridge registers.
#[derive(Debug, Serialize, Deserialize)]
pub struct mmc1 {

    // Banks.
    prg_rom_banks: Vec<Vec<u8>>,
    chr_rom_banks: Vec<Vec<u8>>,

    // Registers


}
