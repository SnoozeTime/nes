use super::Mirroring;
use crate::rom::INesFile;
use serde_derive::{Serialize, Deserialize};

/*
 * PRG is using 8KB banks:
 *  CPU $6000-$7FFF: 8 KB PRG RAM bank (optional)
 *  CPU $8000-$9FFF (or $C000-$DFFF): 8 KB switchable PRG ROM bank
 *  CPU $A000-$BFFF: 8 KB switchable PRG ROM bank
 *  CPU $C000-$DFFF (or $8000-$9FFF): 8 KB PRG ROM bank, fixed to the second-last bank
 *  CPU $E000-$FFFF: 8 KB PRG ROM bank, fixed to the last bank
 * Only two first areas can be switched. 0x8000-0x9FFF and 0xA000-0xBFFF
 *
 * CHR:
 *  PPU $0000-$07FF (or $1000-$17FF): 2 KB switchable CHR bank
 *  PPU $0800-$0FFF (or $1800-$1FFF): 2 KB switchable CHR bank
 *  PPU $1000-$13FF (or $0000-$03FF): 1 KB switchable CHR bank
 *  PPU $1400-$17FF (or $0400-$07FF): 1 KB switchable CHR bank
 *  PPU $1800-$1BFF (or $0800-$0BFF): 1 KB switchable CHR bank
 *  PPU $1C00-$1FFF (or $0C00-$0FFF): 1 KB switchable CHR bank
 *
 *
 * */
#[derive(Serialize, Deserialize)]
pub struct Mmc3 {
    prg_rom_banks: Vec<Vec<u8>>, // 8kb for each element
    chr_rom_banks: Vec<Vec<u8>>, // 1kb banks

    // increment by 8kb
    // 8000-9FFF
    prg_index_1: usize,
    // A000-BFFF
    prg_index_2: usize,
    prg_index_3: usize,
    prg_index_4: usize,

    // increment by 1kb
    chr_index_1: usize,
    chr_index_2: usize,
    chr_index_3: usize,
    chr_index_4: usize,
    chr_index_5: usize,
    chr_index_6: usize,
    chr_index_7: usize,
    chr_index_8: usize,

    // first select the bank to switch with reg_bank_select, then select the
    // bank by writing to reg_bank_data
    reg_bank_select: u8,
    reg_bank_data: usize,

    reg_mirroring: u8,
    reg_ram: u8,

    // relevant for interrupts
    
    // IRQ happened.
    pub irq: bool,

    irq_counter: u8,
    reg_irq_latch: u8,
    irq_enabled: bool,
}


impl Mmc3 {


    pub fn read_prg(&self, addr: usize) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.prg_rom_banks[self.prg_index_1][addr % 0x2000],
            0xA000..=0xBFFF => self.prg_rom_banks[self.prg_index_2][addr % 0x2000],
            0xC000..=0xDFFF => self.prg_rom_banks[self.prg_index_3][addr % 0x2000],
            0xE000..=0xFFFF => self.prg_rom_banks[self.prg_index_4][addr % 0x2000],
            _ => 0,
        }
    }

    // 8 registers.
    pub fn write_prg(&mut self, addr: usize, value: u8) {

        match addr {
            0x8000..=0x9FFF => {
                if addr % 2 == 0 {
                    // 0x8000 is the control register
                    self.reg_bank_select = value;
                } else {
                    // 0x8001 is the data register
                    self.reg_bank_data = value as usize;
                    self.select_bank();
                }
            },

            0xA000..=0xBFFF => {
                if addr % 2 == 0 {
                    // 0xA000 mirroring control
                    self.reg_mirroring = value;

                } else {
                    // 0xA001 WRAM enable/disable..
                    self.reg_ram = value;
                }
            },

            0xC000..=0xDFFF => {
                if addr % 2 == 0 {
                    // 0xC000 IRQ counter reload value.
                    self.reg_irq_latch = value;
                } else {
                    // 0xC001 Clear the IRQ counter.
                    self.reload_irq_counter();
                }
            },

            0xE000..=0xFFFF => {
                if addr % 2 == 0 {
                    // 0xE000 IRQ acknowledge
                    self.disable_irq();

                } else {
                    // 0xE001 IRQ enable
                    self.enable_irq();
                }
            },
            _ => {},
        }
    }

    // Read/Write pattern tables. Sometimes, it is RAM instead of ROM
    pub fn read_chr(&self, addr: usize) -> u8 {
        match addr {
            0x0000..=0x03FF => self.chr_rom_banks[self.chr_index_1][addr % 0x400],
            0x0400..=0x07FF => self.chr_rom_banks[self.chr_index_2][addr % 0x400],
            0x0800..=0x0BFF => self.chr_rom_banks[self.chr_index_3][addr % 0x400],
            0x0C00..=0x0FFF => self.chr_rom_banks[self.chr_index_4][addr % 0x400],
            0x1000..=0x13FF => self.chr_rom_banks[self.chr_index_5][addr % 0x400],
            0x1400..=0x17FF => self.chr_rom_banks[self.chr_index_6][addr % 0x400],
            0x1800..=0x1BFF => self.chr_rom_banks[self.chr_index_7][addr % 0x400],
            0x1C00..=0x1FFF => self.chr_rom_banks[self.chr_index_8][addr % 0x400],
            _ => 0,
        }
    }

    pub fn write_chr(&mut self, addr: usize, value: u8) {
        match addr {
            0x0000..=0x03FF => self.chr_rom_banks[self.chr_index_1][addr % 0x400] = value,
            0x0400..=0x07FF => self.chr_rom_banks[self.chr_index_2][addr % 0x400] = value,
            0x0800..=0x0BFF => self.chr_rom_banks[self.chr_index_3][addr % 0x400] = value,
            0x0C00..=0x0FFF => self.chr_rom_banks[self.chr_index_4][addr % 0x400] = value,
            0x1000..=0x13FF => self.chr_rom_banks[self.chr_index_5][addr % 0x400] = value,
            0x1400..=0x17FF => self.chr_rom_banks[self.chr_index_6][addr % 0x400] = value,
            0x1800..=0x1BFF => self.chr_rom_banks[self.chr_index_7][addr % 0x400] = value,
            0x1C00..=0x1FFF => self.chr_rom_banks[self.chr_index_8][addr % 0x400] = value,
            _ => {}
        }   
    }

    pub fn get_chr(&self, idx: usize) -> &[u8] {
        &self.chr_rom_banks[0] 
    }

    pub fn get_mirroring(&self) -> Mirroring {
        if self.reg_mirroring & 1 == 0 {
            Mirroring::VERTICAL
        } else {
            Mirroring::HORIZONTAL
        }
    }

    pub fn from(ines: &INesFile) -> Result<Mmc3, String> {
        // 0x2000 element vector.
        let mut prg_pages = Vec::new();
        for nb in 0..ines.get_prg_rom_pages() {
            let mut prg_page_low = vec![0; 0x2000];
            let mut prg_page_high = vec![0; 0x2000];

            let rom_page = ines.get_prg_rom(nb+1)?;
            for (i, b) in rom_page[0..0x2000].iter().enumerate() {
                prg_page_low[i] = *b;
            }
            for (i, b) in rom_page[0x2000..0x4000].iter().enumerate() {
                prg_page_high[i] = *b;
            }

            prg_pages.push(prg_page_low);
            prg_pages.push(prg_page_high);
        }

        // 0x400 element vector
        let mut pattern_table_pages = Vec::new();
        for nb in 0..ines.get_chr_rom_pages() {
            let chr_page = ines.get_chr_rom(nb+1)?;
            for i in (0..0x2000).step_by(0x400) {
                let mut page = vec![0; 0x400];
                for j in 0..0x400 {
                    page[j] = chr_page[i+j];
                }
                pattern_table_pages.push(page);
            }
        }

        let prg_index_1 = 0;
        let prg_index_2 = 1;
        let prg_index_3 = prg_pages.len() - 2;
        let prg_index_4 = prg_pages.len() - 1;

        println!("{}, {}, {}, {}", prg_index_1, prg_index_2, prg_index_3, prg_index_4);

        Ok(Mmc3 { 
            prg_rom_banks: prg_pages,
            chr_rom_banks: pattern_table_pages,
            prg_index_1,
            prg_index_2,
            prg_index_3,
            prg_index_4,
            chr_index_1: 0,
            chr_index_2: 1,
            chr_index_3: 2,
            chr_index_4: 3,
            chr_index_5: 4,
            chr_index_6: 5,
            chr_index_7: 6,
            chr_index_8: 7,
            reg_bank_select: 0,
            reg_bank_data: 0,
            reg_mirroring: 0,
            reg_ram: 0,
            irq: false,
            irq_counter: 0,
            reg_irq_latch: 0,
            irq_enabled: false,
        })
    }

    fn select_bank(&mut self) {
        //  7  bit  0
        //  ---- ----
        //  CPMx xRRR
        //  |||   |||
        //  |||   +++- Specify which bank register to update on next write to Bank Data register
        //  |||        0: Select 2 KB CHR bank at PPU $0000-$07FF (or $1000-$17FF);
        //  |||        1: Select 2 KB CHR bank at PPU $0800-$0FFF (or $1800-$1FFF);
        //  |||        2: Select 1 KB CHR bank at PPU $1000-$13FF (or $0000-$03FF);
        //  |||        3: Select 1 KB CHR bank at PPU $1400-$17FF (or $0400-$07FF);
        //  |||        4: Select 1 KB CHR bank at PPU $1800-$1BFF (or $0800-$0BFF);
        //  |||        5: Select 1 KB CHR bank at PPU $1C00-$1FFF (or $0C00-$0FFF);
        //  |||        6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF);
        //  |||        7: Select 8 KB PRG ROM bank at $A000-$BFFF
        //  ||+------- Nothing on the MMC3, see MMC6
        //  |+-------- PRG ROM bank mode (0: $8000-$9FFF swappable,
        //  |                                $C000-$DFFF fixed to second-last bank;
        //  |                             1: $C000-$DFFF swappable,
        //  |                                $8000-$9FFF fixed to second-last bank)
        //  +--------- CHR A12 inversion (0: two 2 KB banks at $0000-$0FFF,
        //                                   four 1 KB banks at $1000-$1FFF;
        //                                1: two 2 KB banks at $1000-$1FFF,
        //                                   four 1 KB banks at $0000-$0FFF)
        // TODO implement
        let bank_select = self.reg_bank_select & 0b111;
        let chr_inversion = self.reg_bank_select & 0x80 == 0x80;
        match bank_select {
            0 => {
                if chr_inversion {
                    self.chr_index_5 = self.reg_bank_data;
                    self.chr_index_6 = self.reg_bank_data+1;
                } else {
                    self.chr_index_1 = self.reg_bank_data;
                    self.chr_index_2 = self.reg_bank_data+1;
                }

            },
            1 => {
                if chr_inversion {
                    self.chr_index_7 = self.reg_bank_data;
                    self.chr_index_8 = self.reg_bank_data+1;
                } else {
                    self.chr_index_3 = self.reg_bank_data;
                    self.chr_index_4 = self.reg_bank_data+1;
                }
            },
            2 => {
                if chr_inversion {
                    self.chr_index_1 = self.reg_bank_data;
                } else {
                    self.chr_index_5 = self.reg_bank_data;
                }
            },
            3 => {
                if chr_inversion {
                    self.chr_index_2 = self.reg_bank_data;
                } else {
                    self.chr_index_6 = self.reg_bank_data;
                }

            },
            4 => {
                if chr_inversion {
                    self.chr_index_3 = self.reg_bank_data;
                } else {
                    self.chr_index_7 = self.reg_bank_data;
                }

            },
            5 => {
                if chr_inversion {
                    self.chr_index_4 = self.reg_bank_data;
                } else {
                    self.chr_index_8 = self.reg_bank_data;
                }

            },
            6 => {
                if (self.reg_bank_select >> 6) & 1 == 1 {
                    self.prg_index_3 = self.reg_bank_data;
                    self.prg_index_1 = self.prg_rom_banks.len() - 2;
                } else {
                    self.prg_index_1 = self.reg_bank_data;
                    self.prg_index_3 = self.prg_rom_banks.len() - 2;
                }
            },
            7 => {
                self.prg_index_2 = self.reg_bank_data;
            },
            _ => panic!("Impossibru"),
        }

    }

    fn reload_irq_counter(&mut self) {
        self.irq_counter = 0;
    }

    fn enable_irq(&mut self) {
        self.irq_enabled = true;
    }

    fn disable_irq(&mut self) {
        self.irq_enabled = false;
        self.irq = false;
    }

    pub fn count_12(&mut self) {

        if self.irq_counter == 0 {
            self.irq_counter = self.reg_irq_latch;
        } else {
            self.irq_counter -= 1;
        }
        
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq = true;         
        }
    }
}
