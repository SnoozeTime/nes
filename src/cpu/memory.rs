// Will contain memory layout and access methods
//
//
use super::cpu::Nes;

pub fn immediate(operand: u8) -> Box<Fn(&mut Nes) -> u8> {
    Box::new( move |ref mut _x| operand )
}

pub fn zero_page(address: u8) -> Box<Fn(&mut Nes) -> u8> {
    Box::new( move |ref mut x| {
       // address is the address of somewhere in the memory.
       // TODO make it more encapsulated with passing the memory object
       // instead of Nes
       x.RAM[address as usize]
    })

}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_immediate() {
        let mut nes = Nes::new(vec![0;5]);
        let addressing = immediate(8);
        assert_eq!(8, addressing(&mut nes));
    }

    #[test]
    fn test_zero_page() {
        let mut nes = Nes::new(vec![1, 2 ,3 ,4 ,5]);
        nes.RAM[0x02] = 3;
        let addressing = zero_page(0x02);
        assert_eq!(3, addressing(&mut nes));
    }
}
