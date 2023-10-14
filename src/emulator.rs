#![allow(dead_code)]
struct IntruptedCPU {
    pub cpu: CPU
}

impl IntruptedCPU {
    pub fn release(self) -> CPU {
        self.cpu
    }
}

struct Instruction {
    carry: bool,
    less_than_zero: bool,
    equal_to_zero: bool,
    greater_than_zero: bool,
    opcode: u8,
    operands: u32
}

// TODO Move into own class and test
impl Instruction {
    fn from_opcode(opcode: u8, operands: u32) -> Self {
       Instruction {
            carry: false,
            less_than_zero: false,
            equal_to_zero: false,
            greater_than_zero: false,
            opcode,
            operands
       }
    }

    #[allow(arithmetic_overflow)]
    fn encode(&self) -> u32 {
    // TODO Add tests?
    // TODO Add checks for oversized values
    // TODO Must be a nicer way to do this....
        (if self.carry {1<< 31} else{ return 0 }) as u32 |
        (if self.less_than_zero {1<< 30} else{ return 0 }) as u32 |
        (if self.equal_to_zero {1<< 29} else{ return 0 }) as u32|
        (if self.greater_than_zero {1<< 28} else{ return 0 }) as u32 |
        // TODO have way to make sure this doesn't overflow
        (self.opcode            << 22) as u32 |
        self.operands          <<  0
    }

    fn decode(value: u32) -> Self {
        // TODO Add tests?
        Instruction {
            carry:               value >> 31 == 1,
            less_than_zero:      value >> 30 == 1,
            equal_to_zero:       value >> 29 == 1,
            greater_than_zero:   value >> 28 == 1,
            opcode:              (value >> 22 & 0x3F) as u8,
            operands:            value >>  0 & 0x3FFFFF,
        }
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

    fn show(&self) -> String {
        let result = format!("GR: {:?}", self.general_purpose);
        return result
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
        cpu.program_counter = 0;
        let throw_interupt = Instruction::from_opcode(32, 0);
        cpu.write(1,0xf);
        match cpu.clock() {
            UnknownCPU::Intrupted(cpu) => (),
            UnknownCPU::Unintrupted(cpu) => panic!("CPU should be in interupted state {}", cpu.show())
        }
    }
}   
