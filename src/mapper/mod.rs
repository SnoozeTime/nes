// mappers extended the life of the NES by providing bank switching capacities
// PRG-ROM: mapped to CPU memory
// CHR-ROM: mapped to pattern tables of PPU
//
//
use serde_derive::{Serialize, Deserialize};
pub mod nrom;
pub mod mmc1;
pub mod uxrom;

use crate::rom;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    ONE_SCREEN,
}

macro_rules! mapper_types {
    ($($name:ident: ($id: expr, $mapper:ty)),+) => {
        #[derive(Serialize, Deserialize)]
        pub enum MapperType {
            $(
                $name($mapper)
            ),+
        }

        impl MapperType {

            pub fn read_prg(&self, addr: usize) -> u8 {
                match *self {
                    $(
                        MapperType::$name(ref x) => x.read_prg(addr),
                        )+
                }
            }


            pub fn write_prg(&mut self, addr: usize, value: u8) {
                match *self {
                    $(
                        MapperType::$name(ref mut x) => x.write_prg(addr, value),
                        )+
                }
            }

            // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
            pub fn read_chr(&self, addr: usize) -> u8 {
                match *self {
                    $(
                        MapperType::$name(ref x) => x.read_chr(addr),
                        )+
                }
            }

            pub fn write_chr(&mut self, addr: usize, value: u8) {
                match *self {
                    $(
                        MapperType::$name(ref mut x) => x.write_chr(addr, value),
                        )+
                }
            }

            pub fn get_chr(&self, idx: usize) -> &[u8] {
                match *self {
                    $(
                        MapperType::$name(ref x) => x.get_chr(idx),
                        )+
                }
            }

            pub fn get_mirroring(&self) -> Mirroring {
                match *self {
                    $(
                        MapperType::$name(ref x) => x.get_mirroring(),
                        )+
                }
            }

        }


        pub fn create_mapper(rom: &rom::INesFile) -> Result<MapperType, String> {

            let mapper_id = rom.get_mapper_id();

            println!("MAPPER ID: {}", mapper_id);
            match mapper_id {
                $(
                    $id => {
                        let x = <$mapper>::from(&rom).unwrap();
                        Ok(MapperType::$name(x))
                    },
                    )+
                    _ => Err(String::from("Not implemented yet"))
            }

        }
    }
}

mapper_types!(
    Nrom: (0, nrom::Nrom),
    Mmc1: (1, mmc1::Mmc1),
    Uxrom: (2, uxrom::Uxrom)
);

