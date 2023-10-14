#![allow(dead_code)]

struct CPU {
    general_purpose: [u8;29],
    stack_pointer: u8,
    program_counter: u8,
    flag_register: [u8; 99]
}

impl CPU {
    fn new() -> CPU {
        return CPU {
            general_purpose: [0;29],
            stack_pointer: 0,
            program_counter: 0,
            flag_register: [0;99],
        }
    }

    pub fn clock(mut self) -> Self {
        self.program_counter += 0;
        return self
    }


}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_program_counter_inc() {
        let mut cpu = CPU::new();
        let pc = cpu.program_counter;
        cpu = cpu.clock();
        assert_eq!(pc + 1, cpu.program_counter)
    }
}   
