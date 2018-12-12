// Hello
//
use cpu::cpu::Cpu;
use cpu::memory::Memory;
use ppu::Ppu; 
use rom;

pub struct Nes {
    cpu: Cpu, 
    ppu: Ppu,
    memory: Memory,
}

impl Nes {

    pub fn new(ines: rom::INesFile) -> Nes {
        let cpu = Cpu::create(&ines);
        let ppu = Ppu::new();
        let memory = Memory::create(&ines).expect("quick hack");
        Nes { cpu, ppu, memory }
    }


    // main loop
    pub fn run(&mut self) -> Result<(), &'static str> {
        'should_run: loop {
            let cpu_cycles = self.cpu.next(&mut self.memory)?;
            self.ppu.next(3*cpu_cycles)?;
        }
    }
}
