#![allow(dead_code)]
struct IntruptedCPU {
    pub cpu: CPU
}

impl IntruptedCPU {
    pub fn release(self) -> CPU {
        self.cpu
    }
}

enum UnknownCPU {
    Intrupted(IntruptedCPU),
    Unintrupted(CPU)
}

struct CPU {
    general_purpose: [u8;29],
    stack_pointer: u8,
    program_counter: u8,
    flag_register: [u8; 99]
}

impl CPU {
    fn function_map(opcode: u8) {
       match opcode {
            _ => todo!("Opcode {opcode} isn't implemented yet")
       }
    }

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
        match addr {
            0 => 0,
            1..=29 => self.general_purpose[(addr - 1) as usize],
            30.. => 0
        }
    }

    pub fn write(&mut self, addr: u32, value: u8) {
        match addr {
            0 => (),
            1..=29 => self.general_purpose[(addr - 1) as usize] = value,
            30.. => ()
        };
        return ()
    }

    /// Simulates a rising edge on the clock 
    pub fn clock(mut self) -> UnknownCPU {
        self.program_counter += 1;
        return UnknownCPU::Unintrupted(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::Fill;

    #[test]
    fn test_program_counter_inc() {
        let mut cpu = CPU::new_blank();
        let pc = cpu.program_counter;
        cpu = match cpu.clock() {
            UnknownCPU::Intrupted(cpu) => panic!("Software interupt called"),
            UnknownCPU::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 1, cpu.program_counter)
    }

    /// any writes to this register have no effect and when read it always
    /// yields zero
    #[test]
    fn test_black_hole_register() {
        let mut cpu = CPU::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..1000 { 
            assert_eq!(cpu.read(0), 0);
            cpu.write(0, rng.gen());
        }
    }

    #[test]
    fn test_general_register() {
        let mut cpu = CPU::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..100 { 
            let mut values = [0;29];
            values.try_fill(&mut rng).unwrap();
            
            for (value, address) in values.iter().zip(1..) {
                cpu.write(address, *value);
            }

            for (value, address) in values.iter().zip(1..) {
                assert_eq!(*value, cpu.read(address), 
                        "Write/Read from ADDR: {} failed",
                        address
                        );
            }
        }
    }

    #[test]
    fn test_software_interrupt() {
        let mut cpu = CPU::new_blank();
        let mut rng = rand::thread_rng();
    }
}   
