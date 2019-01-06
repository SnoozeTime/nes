//
//
use std::time::{Duration, Instant};
use crate::cpu::cpu::Cpu;
use crate::graphic::{EmulatorInput, Graphics};
use crate::cpu::memory::Memory;
use crate::ppu::Ppu; 
use crate::rom;

use std::error::Error;
use std::fs::{OpenOptions, File};
use std::io::{Read, Write};
use serde_derive::{Serialize, Deserialize};



#[derive(Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu, 
    ppu: Ppu,
    memory: Memory,

    #[serde(skip)]
    #[serde(default = "new_graphics")]
    ui: Graphics,
}

fn new_graphics() -> Graphics {
    Graphics::new(3).expect("Could not create window")
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

    // Load from json file.
    pub fn load_state(path: String) -> Result<Nes, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)?;
        let n: Nes = serde_json::from_str(&json_str)?;
        Ok(n)
    }

    // main loop
    pub fn run(&mut self) -> Result<(), &'static str> {
        let mut is_pause = false;
        let mut is_debug = false;

        let mut input_samples: Vec<u32> = Vec::new();
        let mut cpu_samples = Vec::new();
        let mut ppu_samples = Vec::new();
        let mut graph_samples = Vec::new();

        input_samples.reserve(100000);
        cpu_samples.reserve(100000);
        ppu_samples.reserve(100000);
        graph_samples.reserve(100000);

        // Fixed time stamp for input polling.
        let fixed_time_stamp = Duration::new(0, 16666667); 
        let mut previous_clock = Instant::now();
        let mut accumulator = Duration::new(0, 0);

        'should_run: loop {
            accumulator += Instant::now() - previous_clock;
            previous_clock = Instant::now();

            // handle events.
            if accumulator > fixed_time_stamp {
                accumulator -= fixed_time_stamp;
                let now_input = Instant::now();
                match self.ui.handle_events(&mut self.memory, is_pause) {
                    Some(EmulatorInput::QUIT) => break 'should_run,
                    Some(EmulatorInput::PAUSE) => is_pause = !is_pause,
                    Some(EmulatorInput::DEBUG) => is_debug = !is_debug,
                    Some(EmulatorInput::SAVE) => {
                        match self.save_state() {
                            Err(err) => println!("Error while saving state: {}", err),
                            Ok(_) => println!("Successfully saved to saves/saved_state.json"),
                        }
                    },
                    None => {},
                }
                input_samples.push(now_input.elapsed().subsec_micros());
            }

            // Update CPU and PPU (and later APU)
            if !is_pause {
                let now_cpu = Instant::now();
                let cpu_cycles = self.cpu.next(&mut self.memory)?;
                cpu_samples.push(now_cpu.elapsed().subsec_micros());

                let now_ppu =Instant::now();
                self.ppu.next(3*cpu_cycles, &mut self.memory, is_debug)?;
                ppu_samples.push(now_ppu.elapsed().subsec_micros());
            }
            // render
            let now_graph = Instant::now();
            self.ui.display(&mut self.memory, &mut self.ppu);
            graph_samples.push(now_graph.elapsed().subsec_micros());

            // If pause, let's wait a bit to avoid taking all the CPU
            if is_pause {
                self.ui.draw_debug(&self.memory);
                let ten_millis = std::time::Duration::from_millis(10);
                std::thread::sleep(ten_millis);
            }
        }

        let mut avg_input = 0;
        for sample in &input_samples {
            avg_input += *sample;
        }
        //avg_input /= input_samples.len() as u32;

        let mut avg_cpu = 0;
        for sample in &cpu_samples {
            avg_cpu += *sample;
        }
        //avg_cpu /= cpu_samples.len() as u32;

        let mut avg_ppu = 0;
        for sample in &ppu_samples {
            avg_ppu += *sample;
        }
        //avg_ppu /= ppu_samples.len() as u32;

        let mut avg_graph = 0;
        for sample in &graph_samples {
            avg_graph += *sample;
        }
        //avg_graph /= graph_samples.len() as u32;

        println!("INPUT: {}", avg_input);
        println!("CPU: {}", avg_cpu);
        println!("PPU: {}", avg_ppu);
        println!("GRAPH: {}", avg_graph);
        Ok(())
    }

    pub fn decompile(&mut self) {
        loop {
            self.cpu.decompile(&mut self.memory);
        }
    }

    fn save_state(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("saves/saved_state.json")
            .map_err(|err| err.to_string())?;
        let state = serde_json::to_string(&self).map_err(|err| err.to_string())?;
        write!(file, "{}", state).map_err(|err| err.to_string())?;

        Ok(())
    }

}
