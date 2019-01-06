// Read the ROM.
//
use std::fs::File;
use std::io::prelude::*;
use crate::mapper::Mirroring;

fn load(filename: String) -> Result<Vec<u8>, String> {
    File::open(filename)
        .map_err(|err| err.to_string())
        .and_then(|mut file| {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|err| err.to_string())
                .map(|_| contents)
        })
}

pub fn read(filename: String) -> Result<INesFile, String> {
    let bytes = load(filename)?;

    // Check the header is big enough. Expecting 16 bytes.
    if bytes.len() < 16 {
        return Err(format!("ROM size is to short. Expected 16 bytes, got {}", bytes.len()));
    }

    // First 4 bytes are "NES" + MS-DOS EOF
    let expected_bytes = [0x4E, 0x45, 0x53, 0x1A];
    if &expected_bytes[..] != &bytes[0..4] {
        return Err(String::from("ROM 4 first bytes are not $4E $45 $53 $1A"));
    }

    let prg_rom_size = bytes[4] as usize;
    let chr_rom_size = bytes[5] as usize;
    let flags_6 = bytes[6];
    let flags_7 = bytes[7];
    let prg_ram_size = bytes[8] as usize;
    let flags_9 = bytes[9];
    let flags_10 = bytes[10];
    
    for b in &bytes[11..16] {
        if *b != 0x0 {
            return Err(String::from("Expected 5 bytes of 0x0 padding at the end of header"));
        }
    }

    // Trainer if present (check flag 6).
    let mut offset = 16;
    let mut trainer = [0; 512];
    if (flags_6 >> 2) & 1 == 1 {
            
        for i in offset..offset+512 {
            trainer[i-offset] = bytes[i];
        }
        offset += 512;
    }

    // then read the prg rom.
    let mut prg_rom = Vec::new();
    for i in offset..offset+(prg_rom_size*16384) {
        prg_rom.push(bytes[i]);
    }
    offset += prg_rom_size*16384;

    // read the chr_rom
    let mut chr_rom = Vec::new();
    for i in offset..offset+(chr_rom_size*8192) {
        chr_rom.push(bytes[i]);
    }

    Ok(INesFile {
        prg_rom,
        prg_rom_pages: prg_rom_size,
        chr_rom,
        chr_rom_size,
        prg_ram_size,
        flags_6,
        flags_7,
        flags_9,
        flags_10,
    })
}

#[derive(Debug)]
pub struct INesFile {
    // Headers
    prg_rom: Vec<u8>, // in 16kb units
    prg_rom_pages: usize, // pages
    chr_rom: Vec<u8>,
    chr_rom_size: usize, // in 8kb units (value 0 means the board uses CHR RAM)
    flags_6: u8,
    flags_7: u8,
    prg_ram_size: usize, // in 8kb units (value 0 infers 8KB for compatibility)
    flags_9: u8,
    flags_10: u8, // unofficial
}

impl INesFile {

    pub fn new(prg_rom: Vec<u8>,
               prg_rom_pages: usize,
               chr_rom: Vec<u8>,
               chr_rom_size: usize,
               prg_ram_size: usize,
               flags_6: u8,
               flags_7: u8,
               flags_9: u8,
               flags_10: u8) -> INesFile {

        INesFile {
            prg_rom,
            prg_rom_pages,
            chr_rom,
            chr_rom_size,
            prg_ram_size,
            flags_6,
            flags_7,
            flags_9,
            flags_10,
        }
    }

    pub fn get_mapper_id(&self) -> u16 {
        let lower_nib = u16::from((self.flags_6 >> 7) & 1); 
        let upper_nib = u16::from((self.flags_7 >> 7) & 1);

        lower_nib | (upper_nib << 1)
    }

    pub fn get_prg_rom_pages(&self) -> usize {
        self.prg_rom_pages
    }

    pub fn get_prg_rom(&self, page_nb: usize) -> Result<&[u8], String> {
        if page_nb > self.prg_rom_pages {
            return Err(format!("Tried to access page {}, but only have {} pages",
                               page_nb,
                               self.prg_rom_pages));
        }

        Ok(&self.prg_rom[(page_nb-1)*16*1024..page_nb*16*1024])
    }

    pub fn get_chr_rom_pages(&self) -> usize {
        self.chr_rom_size
    }

    pub fn get_chr_rom(&self, page_nb: usize) -> Result<&[u8], String> {
        if page_nb > self.chr_rom_size {
            return Err(format!("Tried to access page {}, but only have {} pages",
                               page_nb,
                               self.chr_rom_size));
        }

        Ok(&self.chr_rom[(page_nb-1)*8*1024..page_nb*8*1024])
    }

    pub fn get_mirroring(&self) -> Mirroring {
        if self.flags_6 & 1 == 1 {
            Mirroring::VERTICAL
        } else {
            Mirroring::HORIZONTAL
        }
    }


}
#[cfg(test)]
mod tests {

    #[test]
    fn load_normal_rom() {

    }
}
