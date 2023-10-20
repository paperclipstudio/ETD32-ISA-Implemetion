#![allow(dead_code)]
mod instruction;
use instruction::Instruction;
use std::fmt;

struct IntruptedCpu {
    pub cpu: Cpu
}

impl IntruptedCpu {
    pub fn release(self) -> Cpu {
        self.cpu
    }
}

enum UnknownCpu {
    Intrupted(IntruptedCpu),
    Unintrupted(Cpu)
}

impl UnknownCpu {
    fn unwrap(self) -> Cpu {
        match self {
            UnknownCpu::Intrupted(cpu) => cpu.release(),
            UnknownCpu::Unintrupted(cpu) => cpu,
        }
    }
}

#[derive(Debug)]
struct Cpu {
    general_purpose: [u8;29],
    stack_pointer: u8,
    program_counter: u8,
    flag_register: [u8; 4]
}

impl fmt::Display for Cpu {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "General {:#?} \n", self.general_purpose).ok();
        write!(fmt, "StackPointer {:#?} \n", self.stack_pointer).ok();
        write!(fmt, "Program Counter {:#?} \n", self.program_counter).ok();
        write!(fmt, "Flag Register {:#?} \n", self.flag_register).ok();

        let mut i = 1;
        while i < self.general_purpose.len() as u8 {
            if i == self.program_counter {
                write!(fmt, ">>>").ok();
            } else {
                write!(fmt, "   ").ok();
            }
            write!(fmt, "{} \n", self.instruction_at(i)).ok();
            i += 4
        }

        return Ok(())
    }
}

impl Cpu {
    fn show(&self) -> String {
        let result = format!("GR: {:?}", self.general_purpose);
        result
    }

    /// Creates a new cpu with random values all values
    pub fn new() -> Cpu {
        use rand::Fill;
        let mut rng = rand::thread_rng();
        let mut cpu = Cpu {
            general_purpose: [0;29],
            stack_pointer: 0,
            program_counter: 0,
            flag_register: [0;4],
        };
        cpu.general_purpose.try_fill(&mut rng)
            .expect("Failed to create random values on Cpu creation");
        cpu.flag_register.try_fill(&mut rng)
            .expect("Failed to create random values on Cpu creation");
        cpu
    }
    /// Creates a new zero'd cpu
    fn new_blank() -> Cpu {
        Cpu {
            general_purpose: [0;29],
            stack_pointer: 0,
            program_counter: 1,
            flag_register: [0;4],
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
    }

    pub fn current_instruction(&self) -> Instruction {
        self.instruction_at(self.program_counter)
    }

    fn instruction_at(&self, i:u8) -> Instruction {
        Instruction::decode (
             self.read(i    ) as u32        |
            (self.read(i + 1) as u32) <<  8 |
            (self.read(i + 2) as u32) << 16 |
            (self.read(i + 3) as u32) << 24 )
    }

    fn load_instruction(&mut self, location: u8, instruction: &Instruction) {
        self.write(location    , (instruction.encode()         & 0xFF) as u8);
        self.write(location + 1, ((instruction.encode() >> 8)  & 0xFF) as u8);
        self.write(location + 2, ((instruction.encode() >> 16) & 0xFF) as u8);
        self.write(location + 3, ((instruction.encode() >> 24) & 0xFF) as u8);
    }

    /// Simulates a rising edge on the clock 
    pub fn clock(self) -> UnknownCpu {
        let instruction = self.current_instruction();
        print!(">> Inst: {:#?}", instruction);
        match instruction.opcode {
            29 => InstSet::jump_offset(self),
            31 => InstSet::jump_to(self),
            32 => InstSet::trigger_interupt(self),
            opcode => panic!("Instruction {opcode} not implemented yet\nInstruction {:#?}\n cpu: {:#?}",instruction, self.show())
        }
        //return UnknownCpu::Unintrupted(self)
    }

}

struct InstSet {}
impl InstSet {
    /// Flow Control
    fn trigger_interupt(mut cpu: Cpu) -> UnknownCpu {
        cpu.program_counter += 1;
        UnknownCpu::Intrupted(IntruptedCpu{cpu})
    }

    fn jump_offset(mut cpu: Cpu) -> UnknownCpu {
        // TODO what happens when jump is too large UB?
        cpu.program_counter += cpu
            .current_instruction()
            .operands as u8;
        UnknownCpu::Unintrupted(cpu)
    }

    fn jump_to(mut cpu: Cpu) -> UnknownCpu {
        let jump_to = cpu.current_instruction().i();
        if jump_to < 0 || jump_to > 29 - 4 {
            UnknownCpu::Intrupted(IntruptedCpu{cpu})
        } else {
            cpu.program_counter = jump_to as u8;
            UnknownCpu::Unintrupted(cpu)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::Fill;

    #[test]
    fn test_program_counter_inc() {
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 1;
        // Load in jump + 1 commands
        let jump_1 = Instruction::from_opcode(29, 4);
        cpu.load_instruction(1, &jump_1);
        cpu.load_instruction(5, &jump_1);
        cpu.load_instruction(9, &jump_1);
        println!(">>Before: {:#?}", cpu);
        let pc = cpu.program_counter;
        cpu = match cpu.clock() {
            UnknownCpu::Intrupted(_) => panic!("Software interupt called"),
            UnknownCpu::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 4, cpu.program_counter);
        println!("SEOND, {:?}", cpu.current_instruction());

        cpu = match cpu.clock() {
            UnknownCpu::Intrupted(_) => panic!("Software interupt called"),
            UnknownCpu::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 8, cpu.program_counter);
        println!("Third, {:?}", cpu.current_instruction());

        cpu = match cpu.clock() {
            UnknownCpu::Intrupted(_) => panic!("Software interupt called"),
            UnknownCpu::Unintrupted(cpu) => cpu,
        };
        assert_eq!(pc + 12, cpu.program_counter);
    }

    /// any writes to this register have no effect and when read it always
    /// yields zero
    #[test]
    fn test_black_hole_register() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..1000 { 
            assert_eq!(cpu.read(0), 0);
            cpu.write(0, rng.gen());
        }
    }

    #[test]
    fn test_general_register() {
        let mut cpu = Cpu::new_blank();
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
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 1;
        let throw_interupt = Instruction::from_opcode(32, 0);
        cpu.load_instruction(1, &throw_interupt);
        match cpu.clock() {
            UnknownCpu::Intrupted(_) => (),
            UnknownCpu::Unintrupted(cpu) => panic!("Cpu should be in interupted state {}", cpu.show())
        }
    }

    #[test]
    fn test_load_instruction() {
        let mut cpu = Cpu::new_blank();
        // Load in jump + 1 commands
        let jump_1 = Instruction::from_opcode(29, 4);
        cpu.load_instruction(1, &jump_1);
        cpu.load_instruction(5, &jump_1);
        cpu.load_instruction(9, &jump_1);
        cpu.program_counter = 1;
        assert_eq!(jump_1, cpu.current_instruction());

    }

    #[test]
    fn test_jump_instruction() {
        let mut cpu = Cpu::new_blank();
        // Fill with Intrupts
        for i in 0..15 {
            let intrupt = Instruction::from_opcode(32, 21);
            cpu.load_instruction(i * 4 + 1, &intrupt);
        }
        let jump_to_1 = Instruction::from_opcode(31,  1);
        let jump_to_21 = Instruction::from_opcode(31, 21);
        let jump_to_9 = Instruction::from_opcode(31, 9);

        cpu.load_instruction( 1, &jump_to_9);
        cpu.load_instruction(9, &jump_to_21);
        cpu.load_instruction(21, &jump_to_1);

        cpu.program_counter = 1;
        cpu = match cpu.clock() {
            UnknownCpu::Intrupted(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Unintrupted(cpu) => cpu,
        };
        assert_eq!(9, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Intrupted(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Unintrupted(cpu) => cpu,
        };
        assert_eq!(21, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Intrupted(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Unintrupted(cpu) => cpu,
        };
        assert_eq!(1, cpu.program_counter);
    }
}   
