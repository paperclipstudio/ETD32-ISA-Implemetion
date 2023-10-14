#![allow(dead_code)]

struct CPU {
    general_purpose: [u8;29],
    stack_pointer: u8,
    program_counter: u8,
    flag_register: [u8; 99]
}

impl CPU {
    /// Creates a new cpu with random values all values
    pub fn new() -> CPU {
        use rand::Fill;
        let mut rng = rand::thread_rng();
        let mut cpu = CPU {
            general_purpose: [0;29],
            stack_pointer: 0,
            program_counter: 0,
            flag_register: [0;99],
        };
        cpu.general_purpose.try_fill(&mut rng)
            .expect("Failed to create random values on CPU creation");
        cpu.flag_register.try_fill(&mut rng)
            .expect("Failed to create random values on CPU creation");


        return cpu
    }
    /// Creates a new zero'd cpu
    fn new_blank() -> CPU {
        CPU {
            general_purpose: [0;29],
            stack_pointer: 0,
            program_counter: 0,
            flag_register: [0;99],
        }
    }

    pub fn read(&self, addr: u32) -> u8 {
        return 1
    }

    pub fn write(&mut self, addr: u32, value: u8) {
        return ()
    }

    /// Simulates a rising edge on the clock 
    pub fn clock(mut self) -> Self {
        self.program_counter += 1;
        return self
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_program_counter_inc() {
        let mut cpu = CPU::new_blank();
        let pc = cpu.program_counter;
        cpu = cpu.clock();
        assert_eq!(pc + 1, cpu.program_counter)
    }
    /// any writes to this register have no effect and when read it always
    /// yields zero
    #[test]
    fn test_black_hole_register() {
        let mut cpu = CPU::new_blank();
        for _ in 0..1000 { 
            let mut rng = rand::thread_rng();
            assert_eq!(cpu.read(0), 0);
            cpu.write(0, rng.gen());
        }
    }
}   
