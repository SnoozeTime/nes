
enum Instruction {
    LDA_immediate(u16, u8)

}

impl Instruction {

    // return debug string
    fn repr(&self) -> String {

    }

    // How long it takes to execute
    fn time(&self) -> u8 {
        match *self {
            LDA_immediate => 2,
            _ => 0
        }
    }
}
