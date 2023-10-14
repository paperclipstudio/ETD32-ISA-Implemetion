#![allow(dead_code)]
mod instruction;
use instruction::Instruction;

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

#[derive(Debug)]
struct CPU {
    general_purpose: [u8;29],
    stack_pointer: u8,
    program_counter: u8,
    flag_register: [u8; 99]
}

impl CPU {

    fn no_op(mut self) -> Self {
        self
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
            program_counter: 1,
            flag_register: [0;99],
        }
    }

    pub fn read(&self, addr: u8) -> u8 {
        match addr {
            0 => 0,
            1..=29 => self.general_purpose[(addr - 1) as usize],
            30.. => 0
        }
    }

    pub fn write(&mut self, addr: u8, value: u8) {
        match addr {
            0 => (),
            1..=29 => self.general_purpose[(addr - 1) as usize] = value,
            30.. => ()
        };
        return ()
    }


    fn current_instruction(&self) -> Instruction {
        // TODO Add tests
        return Instruction::decode (0 |
             self.read(self.program_counter * 4 - 3) as u32        |
            (self.read(self.program_counter * 4 - 2) as u32) <<  8 |
            (self.read(self.program_counter * 4 - 1) as u32) << 16 |
            (self.read(self.program_counter * 4    ) as u32) << 24 )
    }

    fn load_instruction(&mut self, location: u8, instruction: &Instruction) {
        self.write(location + 0, (instruction.encode() & 0xFF) as u8);
        self.write(location + 1, ((instruction.encode() >> 8) & 0xFF) as u8);
        self.write(location + 2, ((instruction.encode() >> 16) & 0xFF) as u8);
        self.write(location + 3, ((instruction.encode() >> 24) & 0xFF) as u8);
    }

    /// Simulates a rising edge on the clock 
    pub fn clock(mut self) -> UnknownCPU {
        let instruction = self.current_instruction();
        print!(">> Inst: {:#?}", instruction);
        match instruction.opcode {
            29 => InstSet::jump_offset(self),
            32 => InstSet::trigger_interupt(self),
            opcode => panic!("Instruction {opcode} not implemented yet\nInstruction {:#?}\n cpu: {:#?}",instruction, self.show())
        }
        //return UnknownCPU::Unintrupted(self)
    }

}

struct InstSet {}
impl InstSet {
    fn trigger_interupt(mut cpu: CPU) -> UnknownCPU {
        cpu.program_counter += 1;
        UnknownCPU::Intrupted(IntruptedCPU{cpu})
    }

    fn jump_offset(mut cpu: CPU) -> UnknownCPU {
        // TODO what happens when jump is too large UB?
        println!("jumping {} steps", cpu.current_instruction().operands);
        cpu.program_counter += cpu
            .current_instruction()
            .operands as u8;
        return UnknownCPU::Unintrupted(cpu);
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
        // Load in jump + 1 commands
        let jump_1 = Instruction::from_opcode(29, 1);
        cpu.load_instruction(1, &jump_1);
        cpu.load_instruction(5, &jump_1);
        cpu.load_instruction(9, &jump_1);
        println!(">>Before: {:#?}", cpu);
        let pc = cpu.program_counter;
        cpu = match cpu.clock() {
            UnknownCPU::Intrupted(cpu) => panic!("Software interupt called"),
            UnknownCPU::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 1, cpu.program_counter);
        cpu = match cpu.clock() {
            UnknownCPU::Intrupted(cpu) => panic!("Software interupt called"),
            UnknownCPU::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 2, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCPU::Intrupted(cpu) => panic!("Software interupt called"),
            UnknownCPU::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 3, cpu.program_counter);
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
        cpu.program_counter = 1;
        let throw_interupt = Instruction::from_opcode(32, 0);
        cpu.load_instruction(1, &throw_interupt);
        match cpu.clock() {
            UnknownCPU::Intrupted(cpu) => (),
            UnknownCPU::Unintrupted(cpu) => panic!("CPU should be in interupted state {}", cpu.show())
        }
    }

    #[test]
    fn test_load_instruction() {
        let mut cpu = CPU::new_blank();
        // Load in jump + 1 commands
        let jump_1 = Instruction::from_opcode(29, 4);
        cpu.load_instruction(1, &jump_1);
        cpu.load_instruction(5, &jump_1);
        cpu.load_instruction(9, &jump_1);
        cpu.program_counter = 1;
        assert_eq!(jump_1, cpu.current_instruction());

    }
}   
