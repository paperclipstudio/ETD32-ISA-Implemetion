#![allow(dead_code)]
mod instruction;
mod memory;
use instruction::Instruction;
use std::fmt;
use rand;
use memory::*;


struct InterruptedCpu {
    pub cpu: Cpu
}

impl InterruptedCpu {
    pub fn release(self) -> Cpu {
        self.cpu
    }
}

enum UnknownCpu {
    Inter(InterruptedCpu),
    Ok(Cpu)
}

impl UnknownCpu {
    fn unwrap(self) -> Cpu {
        match self {
            UnknownCpu::Inter(cpu) => cpu.release(),
            UnknownCpu::Ok(cpu) => cpu,
        }
    }
}

struct Cpu {
    general_purpose: [u8;29],
    stack_pointer: u8,
    program_counter: u8,
    flag_register: [u8; 4],
    memory: Box<dyn Memory>,
}

impl fmt::Display for Cpu {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "General {:#?}", self.general_purpose).ok();
        writeln!(fmt, "StackPointer {:#?}", self.stack_pointer).ok();
        writeln!(fmt, "Program Counter {:#?}", self.program_counter).ok();
        writeln!(fmt, "Flag Register {:#?}", self.flag_register).ok();

        let mut i = 1;
        while i < self.general_purpose.len() as u8 {
            if i == self.program_counter {
                write!(fmt, ">>>").ok();
            } else {
                write!(fmt, "   ").ok();
            }
            writeln!(fmt, "{}", self.instruction_at(i)).ok();
            i += 4
        }

        for i in 0..32 {
            if i % 4 == 1 {
                write!(fmt, "\n|{:3}|", i).ok();
            };
            write!(fmt, "{:X},", self.read(i)).ok();
        }

        Ok(())
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
            memory: Box::new(SimpleMemory::new())
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
            memory: Box::new(SimpleMemory::new_blank()),
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
        println!(">> Inst: {}", instruction);
        match instruction.opcode {
            17 => InstSet::load_8_bo(self),
            29 => InstSet::jump_offset(self),
            30 => InstSet::jump_to_rd(self),
            31 => InstSet::jump_to_i(self),
            32 => InstSet::trigger_interupt(self),
            opcode => panic!("Instruction {opcode} not implemented yet\nInstruction {}\n cpu: {}",instruction, self.show())
        }
        //return UnknownCpu::Ok(self)
    }

}

struct InstSet {}
impl InstSet {
    ///Memory
    fn load_8_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let memory_address = instruction.r_base() + instruction.i_offset() as u8;
        let value = match cpu.memory.read(memory_address) {
            Some(value) => value,
            None => return UnknownCpu::Inter(InterruptedCpu{cpu})
        };
        println!("Base {}, Offset {}", instruction.r_base(), instruction.i_offset());
        println!("M{}", memory_address);
        println!("{}", cpu.memory);
        println!("V{}", value);
        cpu.write(instruction.r_dest(), value);
        UnknownCpu::Ok(cpu)
    }

    /// Flow Control
    fn trigger_interupt(mut cpu: Cpu) -> UnknownCpu {
        cpu.program_counter += 1;
        UnknownCpu::Inter(InterruptedCpu{cpu})
    }

    fn jump_offset(mut cpu: Cpu) -> UnknownCpu {
        // TODO what happens when jump is too large UB?
        cpu.program_counter += cpu
            .current_instruction()
            .i() as u8;
        UnknownCpu::Ok(cpu)
    }

    fn jump_to_rd(mut cpu: Cpu) -> UnknownCpu {
        let jump_to = cpu.read(cpu.current_instruction().r_dest());
        if jump_to > 29 - 4 {
            UnknownCpu::Inter(InterruptedCpu{cpu})
        } else {
            cpu.program_counter = jump_to;
            UnknownCpu::Ok(cpu)
        }
    }

    fn jump_to_i(mut cpu: Cpu) -> UnknownCpu {
        let jump_to = cpu.current_instruction().i();
        if !(0..=29 - 4).contains(&jump_to) {
            UnknownCpu::Inter(InterruptedCpu{cpu})
        } else {
            cpu.program_counter = jump_to as u8;
            UnknownCpu::Ok(cpu)
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
        let pc = cpu.program_counter;
        cpu = match cpu.clock() {
            UnknownCpu::Inter(_) => panic!("Software interupt called"),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(pc + 4, cpu.program_counter);
        println!("SEOND, {:?}", cpu.current_instruction());

        cpu = match cpu.clock() {
            UnknownCpu::Inter(_) => panic!("Software interupt called"),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(pc + 8, cpu.program_counter);
        println!("Third, {:?}", cpu.current_instruction());

        cpu = match cpu.clock() {
            UnknownCpu::Inter(_) => panic!("Software interupt called"),
            UnknownCpu::Ok(cpu) => cpu,
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
            UnknownCpu::Inter(_) => (),
            UnknownCpu::Ok(cpu) => panic!("Cpu should be in interupted state {}", cpu.show())
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

    //TODO Add test for out of bounds jumps

    #[test]
    fn test_jump_i_instruction() {
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
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(9, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(21, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn test_jump_rd_instruction() {
        let mut cpu = Cpu::new_blank();
        // Fill with Intrupts
        for i in 0..15 {
            let intrupt = Instruction::from_opcode(32, 21);
            cpu.load_instruction(i * 4 + 1, &intrupt);
        }

        let mut jump_to_1 = Instruction::from_opcode(30,  0);
        let mut jump_to_21 = Instruction::from_opcode(30, 0);
        let mut jump_to_9 = Instruction::from_opcode(30, 0);
        println!("Set 1");
        // We will jump to the value in register 10
        jump_to_1.r_dest_set(6);
        // We set the value of register 10 to 1
        cpu.write(6, 1);
        // So this instruction will just to instruction 1
        println!("Set 21");
        jump_to_21.r_dest_set(7);
        cpu.write(7, 21);
        println!("Set 9");
        jump_to_9.r_dest_set(8);
        cpu.write(8, 9);

        cpu.load_instruction(1, &jump_to_9);
        cpu.load_instruction(9, &jump_to_21);
        cpu.load_instruction(21, &jump_to_1);

        println!("{cpu}");

        cpu.program_counter = 1;
        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(9, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(21, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu.release()),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn test_memory_setting() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            let mut values = [0;MEMORY_SIZE as usize];
            values.try_fill(&mut rng).unwrap();
            
            for (value, address) in values.iter().zip(0..) {
                println!("{:?}", cpu.memory.write(address, *value));
                println!("Wrote {} to  {}", value, address);
                println!("data {}", cpu.memory);
            }
            println!("MEME\n{}", cpu.memory);
            for (value, address) in values.iter().zip(0..) {
                assert_eq!(Some(*value), cpu.memory.read(address), 
                        "Write/Read num {} from ADDR: {} failed\n\n{}",
                        i,
                        address,
                        cpu.memory
                        );
            }
        }
    }

    #[test]
    fn test_ld8_bo_base() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..10 { 
            let mut values = [43;(MEMORY_SIZE - 1) as usize];
            for i in 0..values.len() {
                values[i] = i as u8
            }
            values.try_fill(&mut rng).unwrap();
            for (value, address) in values.iter().zip(0..) {
                cpu.memory.write(address, *value).ok();
                //println!("Wrote {} to  {}", value, address);
                //println!("data {}", cpu.memory);
            }
            for address in 3..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(17, 0);
                instruction.r_target_set(1);
                instruction.r_base_set(address);
                instruction.i_offset_set(0);
                cpu.load_instruction(1, &instruction);
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu.release()),
                };
                cpu.program_counter = 1;

                assert_eq!(values[address as usize], cpu.read(1), "{}", instruction);
            }
        }
    }

    #[test]
    fn test_ld8_bo_offset() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..10 { 
            let mut values = [43;(MEMORY_SIZE - 1) as usize];
            for i in 0..values.len() {
                values[i] = i as u8
            }
            values.try_fill(&mut rng).unwrap();
            for (value, address) in values.iter().zip(0..) {
                cpu.memory.write(address, *value).ok();
                //println!("Wrote {} to  {}", value, address);
                //println!("data {}", cpu.memory);
            }
            for address in 3..MEMORY_SIZE - 1 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(17, 0);
                instruction.r_target_set(1);
                instruction.r_base_set(0);
                instruction.i_offset_set(address as u32);
                cpu.load_instruction(1, &instruction);
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu.release()),
                };
                cpu.program_counter = 1;

                assert_eq!(values[address as usize], cpu.read(1), "{}", instruction);
            }
        }
    }


    #[test]
    fn test_ld16_bo_base() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..10 { 
            let mut values = [43;(MEMORY_SIZE - 1) as usize];
            for i in 0..values.len() {
                values[i] = i as u8
            }
            values.try_fill(&mut rng).unwrap();
            for (value, address) in values.iter().zip(0..) {
                cpu.memory.write(address, *value).ok();
                //println!("Wrote {} to  {}", value, address);
                //println!("data {}", cpu.memory);
            }
            for address in 4..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(18, 0);
                instruction.r_target_set(1);
                instruction.r_base_set(address);
                instruction.i_offset_set(0);
                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu.release()),
                };

                let value = cpu.read(1) << 4 | cpu.read(2);
                let expected = values[address as usize] << 4 | values[address as usize + 1];
                assert_eq!(expected, value, "{}", instruction);
            }
        }
    }
}   
