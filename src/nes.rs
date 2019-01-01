// Hello
//
use cpu::cpu::Cpu;
use graphic::{EmulatorInput, Graphics};
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


        let ui = Graphics::new(3)?;
        Ok(Nes { cpu, ppu, memory, ui })
    }


    // main loop
    pub fn run(&mut self) -> Result<(), &'static str> {
        let mut is_pause = false;
        let mut is_debug = false;
        'should_run: loop {
            // handle events.
            match self.ui.handle_events(&mut self.memory, is_pause) {
                Some(EmulatorInput::QUIT) => break 'should_run,
                Some(EmulatorInput::PAUSE) => is_pause = !is_pause,
                Some(EmulatorInput::DEBUG) => is_debug = !is_debug,
                None => {},
            }

            // Update CPU and PPU (and later APU)
            if !is_pause {
                let cpu_cycles = self.cpu.next(&mut self.memory)?;
                self.ppu.next(3*cpu_cycles, &mut self.memory, is_debug)?;
            }
            // render
            self.ui.display(&mut self.memory, &mut self.ppu);

            // If pause, let's wait a bit to avoid taking all the CPU
            if is_pause {
		self.ui.draw_debug(&self.memory);
                let ten_millis = std::time::Duration::from_millis(10);
                std::thread::sleep(ten_millis);
            }

            if is_debug {
                
                self.memory.dump();
                panic!("BIM");
            }
        }

        Ok(())
    }

    pub fn decompile(&mut self) {
        loop {
            self.cpu.decompile(&mut self.memory);
        }
    }
}
