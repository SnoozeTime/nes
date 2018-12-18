// Hello
//
use cpu::cpu::Cpu;
use graphic::Graphics;
use cpu::memory::Memory;
use ppu::Ppu; 
use rom;

pub struct Nes {
    cpu: Cpu, 
    ppu: Ppu,
    memory: Memory,
    ui: Graphics,
}

impl Nes {

    pub fn new(ines: rom::INesFile) -> Result<Nes, String> {
        let mut cpu = Cpu::new();
        let ppu = Ppu::new();
        let mut memory = Memory::new(&ines)?;

        // Need to set the correct PC. It is at FFFC-FFFD
        let lsb = memory.get(0xFFFC) as u16;
        let msb = memory.get(0xFFFD) as u16;
        let start_pc = (msb << 8) + lsb;
        cpu.set_pc(start_pc);


        let ui = Graphics::new(2)?;
        Ok(Nes { cpu, ppu, memory, ui })
    }


    // main loop
    pub fn run(&mut self) -> Result<(), &'static str> {
        'should_run: loop {
            // handle events.
            if self.ui.handle_events(&mut self.memory) {
                break 'should_run;
            }

            // Update CPU and PPU (and later APU)
            let cpu_cycles = self.cpu.next(&mut self.memory)?;
            self.ppu.next(3*cpu_cycles, &mut self.memory)?;

            // render
            self.ui.real_display(&mut self.memory, &mut self.ppu);
        }

        Ok(())
    }

    pub fn decompile(&mut self) {
        loop {
            self.cpu.decompile(&mut self.memory);
        }
    }
}
