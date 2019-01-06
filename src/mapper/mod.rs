// mappers extended the life of the NES by providing bank switching capacities
// PRG-ROM: mapped to CPU memory
// CHR-ROM: mapped to pattern tables of PPU
//
//
pub mod nrom;
use crate::rom;


pub type MapperPtr = Box<dyn Mapper>;

pub trait Mapper: erased_serde::Serialize {
    // Read ROM from cardridge
    // writing is needed for some mappers that have registers.
    fn read_prg(&self, addr: usize) -> u8;
    fn write_prg(&mut self, addr: usize, value: u8);

    // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
    fn read_chr(&self, addr: usize) -> u8;
    fn write_chr(&mut self, addr: usize, value: u8);
    fn get_chr(&self) -> &[u8];
}

serialize_trait_object!(Mapper);

pub fn create_mapper(rom: &rom::INesFile) -> Result<MapperPtr, String> {

    let mapper_id = rom.get_mapper_id();

    match mapper_id {
        0 => {
            let nrom = nrom::Nrom::from(&rom)?;
            Ok(Box::new(nrom))
        },
        _ => Err(String::from("Not implemented yet"))
    }

}
