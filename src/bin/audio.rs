
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

use std::io::BufReader;
use std::thread;
use rodio::Source;
use rodio::source::Empty;
use rodio::source::Zero;
use rodio::Sink;
use rodio::Sample;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct StreamingSource {
     available: Mutex<Vec<(Box<Source<Item = S> + Send>, Option<Sender<()>>)>>,
}

impl Iterator for StreamingSource {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let sample = self.available.pop().or(Some(0.0));
        println!("{:?}", sample);
        return sample; 
    }
}

impl Source for StreamingSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        48000
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() {
    let device = rodio::default_output_device().unwrap();
    let mut sink = Sink::new(&device);

    let mut streaming = StreamingSource { available: Vec::new() };
    sink.set_volume(0.2);
    sink.append(streaming.clone());

    let mut t: u64 = 0;
    loop {
        t = t.wrapping_add(1);
        let value = 2.0 * 3.14159265 * 440.0 * t as f32 / 48000.0;
        let output = value.sin();
        streaming.available.push(output);
        
    thread::sleep(Duration::from_millis(10));
    }
}
