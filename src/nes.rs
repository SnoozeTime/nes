// Hello
//
use cpu::cpu::Cpu;
use ppu::Ppu; 
use rom;

pub struct Nes {
    cpu: Cpu, 
    ppu: Ppu,
}

impl Nes {

    pub fn new(ines: rom::INesFile) -> Nes {
        let cpu = Cpu::create(&ines);
        let ppu = Ppu {};
        Nes { cpu, ppu }
    }


    // main loop
    pub fn run(&mut self) -> Result<(), &'static str> {
        'should_run: loop {
            let cpu_cycles = self.cpu.next()?;
            self.ppu.next(3*cpu_cycles)?;
        }
    }
}
