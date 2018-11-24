// Hello
//
use cpu::cpu::Cpu;
use rom;

pub struct Nes {
   cpu: Cpu, 
}

impl Nes {

    pub fn new(ines: rom::INesFile) -> Nes {

        panic!("nooooo");
    }


    // main loop
    pub fn run(&mut self) {

    }
}
