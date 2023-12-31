#![allow(dead_code)]
pub mod instruction;
mod memory;
mod flags;

use std::fmt;
use rand;
pub use instruction::Instruction;
use memory::Memory;
use memory::SimpleMemory;
use flags::Flags;

#[must_use]
pub enum UnknownCpu {
    Inter(Cpu),
    Ok(Cpu)
}


pub struct Cpu {
    general_purpose: [u8;29],
    stack_pointer: u8,
    pub program_counter: u8,
    flags: Flags,
    pub memory: Box<dyn Memory>,
}

impl fmt::Display for Cpu {

    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "StackPointer {:#?}", self.stack_pointer)?;
        writeln!(fmt, "Program Counter {:#?}", self.program_counter)?;
        writeln!(fmt, "Flag Register {}", self.flags)?;


        writeln!(fmt, "Current Instructions")?;
        // Current place in instructions
        let first = i8::max(0, (self.program_counter as i8)/4 - 4) as u8;
        for i in first..first + 9{
            let pc = i * 4;
                write!(fmt, "{pc:3}||")?;
            if pc == self.program_counter {
                write!(fmt, "-->")?;
            } else {
                write!(fmt, "   ")?;
            }
            writeln!(fmt,"{}", self.instruction_at(pc))?
        }
        
        // Print General Registers
        writeln!(fmt, "General Registers")?;
        for i in 0..32 {
            if i % 4 == 1 {
                write!(fmt, "\n|{:3}|", i)?;
            };
            write!(fmt, "{:X},", self.read(i))?;
        }
        Ok(())
    }
}

impl Cpu {
    const fn is_valid_register(&self, value: u8) -> bool {
        return value < 31
    }
    
    fn copy_from_memory(&mut self, from: u8, to:u8) -> Result<(),&'static str> {
        if !self.is_valid_register(to) {
            return Err("Invalid register to write too");
        }
        let value = self.memory.read(from)
            .ok_or("Invalid read from memory")?;
        self.write(to, value);
        Ok(())
    }

    fn copy_to_memory(&mut self, from: u8, to:u8) -> Result<(),&'static str> {
        if !self.is_valid_register(from) {
            return Err("Invalid register to write to");
        }
        let value = self.read(from);
        self.memory.write(to, value)?;
        Ok(())
    }

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
            program_counter: 1,
            flags: Flags::new(),
            memory: Box::new(SimpleMemory::new())
        };
        cpu.general_purpose.try_fill(&mut rng)
            .expect("Failed to create random values on Cpu creation");
        cpu
    }
    /// Creates a new zero'd cpu
    pub fn new_blank() -> Cpu {
        Cpu {
            general_purpose: [0;29],
            stack_pointer: 0,
            program_counter: 1,
            flags: Flags::new(),
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
        Instruction::decode(self.memory.read_u32(i).unwrap())
    }

    pub fn load_instruction(&mut self, location: u8, instruction: &Instruction) {
            println!("Loading into {}: {}", location, instruction);
        //TODO Add check for write...
        match self.memory.write_u32(location, instruction.encode()) {
            Ok(_) => (),
            Err(_) => panic!("Instruction failed to be loaded into memory"),

        }
    }

    /// Simulates a rising edge on the clock 
    pub fn clock(mut self) -> UnknownCpu {
        let instruction = self.current_instruction();
        // Check flags
        //println!("Cpu flags {}, instruction flags {}", self.flags, instruction.flags);
        if !Flags::instruction_can_run(&self.flags, &instruction.flags) {
            println!("Skipping instruction");
            self.program_counter += 4;
            return UnknownCpu::Ok(self);
        } else {
            println!("Not skipping instruction");
        }

        println!("Running instruction at {}: {instruction}", self.program_counter);
        match instruction.opcode {
            0 => InstSet::logical_left_shift_rd(self),
            1 => InstSet::logical_left_shift_ri(self),
            2 => InstSet::logical_right_shift_rd(self),
            3 => InstSet::logical_right_shift_ri(self),
            4 => InstSet::logical_and_rd(self),
            5 => InstSet::logical_and_ri(self),
            6 => InstSet::logical_or_rd(self),
            7 => InstSet::logical_or_ri(self),
            8 => InstSet::logical_xor_rd(self),
            9 => InstSet::logical_xor_ri(self),
            10 => InstSet::logical_not_rd(self),
            11 => InstSet::logical_add_rd(self),
            12 => InstSet::logical_add_ri(self),
            13 => InstSet::sub_rd(self),
            14 => InstSet::sub_ri(self),
            15 => InstSet::multiply_rd(self),
            16 => InstSet::multiply_ri(self),
            17 => InstSet::load_8_bo(self),
            18 => InstSet::load_8_bi(self),
            19 => InstSet::load_16_bo(self),
            20 => InstSet::load_16_bi(self),
            21 => InstSet::load_32_bo(self),
            22 => InstSet::load_32_bi(self),
            23 => InstSet::store_8_bo(self),
            24 => InstSet::store_8_bi(self),
            25 => InstSet::store_16_bo(self),
            26 => InstSet::store_16_bi(self),
            27 => InstSet::store_32_bo(self),
            28 => InstSet::store_32_bi(self),
            29 => InstSet::jump_offset(self),
            30 => InstSet::jump_to_rd(self),
            31 => InstSet::jump_to_i(self),
            32 => InstSet::trigger_interupt(self),
            opcode => {
                println!("Instruction {opcode} not implemented yet\nInstruction {}\n cpu: {}",instruction, self.show());
                UnknownCpu::Inter(self)
            }

        }
        //return UnknownCpu::Ok(self)
    }

}

struct InstSet {}
impl InstSet {
    fn apply_rd_function<F>(mut cpu:Cpu, op:F) -> UnknownCpu 
        where F: Fn(u8, u8) -> (u8, bool) {
            let instruction = cpu.current_instruction();
            let x = cpu.read(instruction.r_x());
            let y = cpu.read(instruction.r_y());
            println!("{x}, {y}");
            let (result, carry) = op(x,y);
            cpu.flags.carry = carry;
            cpu.flags.greater = result > 0;
            // cpu.flags.less = result < 0;
            cpu.flags.zero = result == 0;
            println!("{result}, {result}");
            cpu.write(
                instruction.r_dest(),
                result
                );
            cpu.program_counter += 4;
            UnknownCpu::Ok(cpu)
        }

    fn apply_ri_function<F>(mut cpu:Cpu, op:F) -> UnknownCpu 
        where F: Fn(u8, i16) -> (u8, bool) {
            let instruction = cpu.current_instruction();
            let x = cpu.read(instruction.r_x());
            let y = instruction.i_y();
            println!("RI: {x}, {y}");
            println!("{instruction}");
            let (result, carry) = op(x,y);
            println!("{result}, {carry}");
            cpu.flags.carry = carry;
            cpu.flags.greater = result > 0;
            //TODO Review the meaning of less than zero, Does the ALU assume signned ints?
            // cpu.flags.less = result < 0;
            cpu.flags.zero = result == 0;
            if result > u8::MAX.into() { 
                panic!("What should I do with a result value too large for target?");
            }
            if result < u8::MIN.into() { 
                panic!("What should I do with a result value too small for target?");
            }
            cpu.write(
                instruction.r_dest(),
                result
                );
            cpu.program_counter += 4;
            UnknownCpu::Ok(cpu)
        }

    /// Operations
    fn logical_right_shift_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |value, shift|((value >> shift) & 0xFF, false))
    }

    fn logical_right_shift_ri(cpu:Cpu) -> UnknownCpu {
         InstSet::apply_ri_function(cpu, |value, shift|((value >> shift) & 0xFF, false))
    }

    fn logical_left_shift_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |value, shift|((value << shift) & 0xFF, false))
    }

    fn logical_left_shift_ri(cpu:Cpu) -> UnknownCpu {
         InstSet::apply_ri_function(cpu, |value, shift|((value << shift) & 0xFF, false))
    }

    fn logical_and_ri(cpu:Cpu) -> UnknownCpu {
         InstSet::apply_ri_function(cpu, |x, y|(x & y as u8, false))
    }

    fn logical_and_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, y|(x & y, false))
    }

    fn logical_or_ri(cpu:Cpu) -> UnknownCpu {
         InstSet::apply_ri_function(cpu, |x, y| (x | y as u8, false))
    }

    fn logical_or_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, y| (x | y, false))
    }

    fn logical_xor_ri(cpu:Cpu) -> UnknownCpu {
         InstSet::apply_ri_function(cpu, |x, y|(x ^ y as u8, false))
    }

    fn logical_xor_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, y|(x ^ y, false))
    }

    fn logical_not_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, _|(!x, false))
    }
        
    fn logical_add_ri(cpu:Cpu) -> UnknownCpu {
         InstSet::apply_ri_function(cpu, |x, y|x.overflowing_add(y as u8))
    }

    fn logical_add_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, y|x.overflowing_add(y))
    }

    fn sub_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, y|x.overflowing_sub(y))
    }

    fn sub_ri(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_ri_function(cpu, 
                                   |x, y| match y {
                                       y if y < 0 => x.overflowing_sub(y as u8),
                                       _ => x.overflowing_sub(y as u8)
                                   })
    }

    fn multiply_rd(cpu:Cpu) -> UnknownCpu {
        InstSet::apply_rd_function(cpu, |x, y|x.overflowing_mul(y))
    }

    fn multiply_ri(cpu:Cpu) -> UnknownCpu {
        if cpu.current_instruction().i_y() < 0 {
            panic!("Don't know how to handle negative multiply right now")
        }
        InstSet::apply_ri_function(cpu, |x, y| x.overflowing_mul(y as u8))
    }
    ///Memory
    fn load_8_bi(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let index = cpu.read(instruction.r_index());
        let memory_address = base + index;
        match cpu.copy_from_memory(memory_address, instruction.r_dest()) {
            Ok(()) => UnknownCpu::Ok(cpu),
            Err(msg) => {
                println!("{msg}"); 
                cpu.program_counter += 4;
                return UnknownCpu::Inter(cpu);
            }
        }
    }

    fn load_8_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let memory_address = base + instruction.i_offset() as u8;
        match cpu.copy_from_memory(memory_address, instruction.r_dest()) {
            Ok(()) => UnknownCpu::Ok(cpu),
            Err(msg) => {
                println!("{msg}"); 
                cpu.program_counter += 4;
                return UnknownCpu::Inter(cpu);
            }
        }
    }

    fn load_16_bi(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let index = cpu.read(instruction.r_index());
        // TODO Add check for overflow
        let memory_address = base + index;
        //TODO Add a check that this can all be done before hand. ie make atomic
        for i in 0..2 {
            match cpu.copy_from_memory(memory_address + i, instruction.r_dest() + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        return UnknownCpu::Ok(cpu)
    }

    fn load_16_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        // TODO Add check for overflow
        let memory_address = base + instruction.i_offset() as u8;
        //TODO Add a check that this can all be done before hand. ie make atomic
        for i in 0..2 {
            match cpu.copy_from_memory(memory_address + i, instruction.r_dest() + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        return UnknownCpu::Ok(cpu)
    }

    fn load_32_bi(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let index = cpu.read(instruction.r_index());
        let memory_address = base + index;
        //TODO Add a check that this can all be done before hand. ie make atomic
        for i in 0..4 {
            match cpu.copy_from_memory(memory_address + i, instruction.r_dest() + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn load_32_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let memory_address = base + instruction.i_offset() as u8;
        //TODO Add a check that this can all be done before hand. ie make atomic
        for i in 0..4 {
            match cpu.copy_from_memory(memory_address + i, instruction.r_dest() + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn store_8_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let memory_address = base + instruction.i_offset() as u8;
        println!("STORE|FROM:{}, TO:{}", instruction.r_target(), memory_address);
        match cpu.copy_to_memory(instruction.r_target(), memory_address) {
            Ok(()) => (),
            Err(_msg) => return UnknownCpu::Inter(cpu),
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn store_8_bi(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let offset = cpu.read(instruction.r_index());
        let memory_address = base + offset;
        println!("STORE|FROM:{}, TO:{}", instruction.r_target(), memory_address);
        match cpu.copy_to_memory(instruction.r_target(), memory_address) {
            Ok(()) => (),
            Err(_msg) => return UnknownCpu::Inter(cpu),
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn store_16_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let memory_address = base + instruction.i_offset() as u8;
        println!("STORE|FROM:{}, TO:{}", instruction.r_target(), memory_address);
        //TODO Make atomic
        for i in 0..2 {
            match cpu.copy_to_memory(instruction.r_target() + i, memory_address + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn store_16_bi(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let offset = cpu.read(instruction.r_index());
        let memory_address = base + offset;
        println!("STORE|FROM:{}, TO:{}", instruction.r_target(), memory_address);
        for i in 0..2 {
            match cpu.copy_to_memory(instruction.r_target() + i, memory_address + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn store_32_bi(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let offset = cpu.read(instruction.r_index());
        let memory_address = base + offset;
        println!("STORE|FROM:{}, TO:{}", instruction.r_target(), memory_address);
        for i in 0..4 {
            match cpu.copy_to_memory(instruction.r_target() + i, memory_address + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    fn store_32_bo(mut cpu: Cpu) -> UnknownCpu {
        let instruction = cpu.current_instruction();
        let base = cpu.read(instruction.r_base());
        let memory_address = base + instruction.i_offset() as u8;
        println!("STORE|FROM:{}, TO:{}", instruction.r_target(), memory_address);
        //TODO Make atomic
        for i in 0..4 {
            match cpu.copy_to_memory(instruction.r_target() + i, memory_address + i) {
                Ok(()) => (),
                Err(_msg) => return UnknownCpu::Inter(cpu),
            }
        }
        cpu.program_counter += 4;
        UnknownCpu::Ok(cpu)
    }

    /// Flow Control
    fn trigger_interupt(cpu: Cpu) -> UnknownCpu {
        UnknownCpu::Inter(cpu)
    }

    fn jump_offset(mut cpu: Cpu) -> UnknownCpu {
        // TODO what happens when jump is too larg?
        cpu.program_counter += cpu
            .current_instruction()
            .i() as u8;
        UnknownCpu::Ok(cpu)
    }

    fn jump_to_rd(mut cpu: Cpu) -> UnknownCpu {
        let jump_to = cpu.read(cpu.current_instruction().r_dest());
        cpu.program_counter = jump_to;
        UnknownCpu::Ok(cpu)
    }

    fn jump_to_i(mut cpu: Cpu) -> UnknownCpu {
        let jump_to = cpu.current_instruction().i().try_into();
        match jump_to {
            Err(_) =>{
                println!("Can't handle program counter of {}", cpu.current_instruction().i());
                UnknownCpu::Inter(cpu)
            },
            Ok(to) => {
                cpu.program_counter = to;
                UnknownCpu::Ok(cpu)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::Fill;
    use memory::MEMORY_SIZE;

    #[test]
    fn test_program_counter_inc() {
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 1;
        // Load in jump + 1 commands
        let mut jump_1 = Instruction::from_opcode(29);

        jump_1.i_set(4);
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
        let throw_interupt = Instruction::from_opcode(32);
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
        let mut jump_1 = Instruction::from_opcode(29);
        jump_1.i_set(4);
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
            let mut intrupt = Instruction::from_opcode(32);
            intrupt.opcode = 21;
            cpu.load_instruction(i * 4 + 1, &intrupt);
        }
        let mut jump_to_1 = Instruction::from_opcode(31);
        jump_to_1.i_set(1);
        let mut jump_to_21 = Instruction::from_opcode(31);
        jump_to_21.i_set(21);
        let mut jump_to_9 = Instruction::from_opcode(31);
        jump_to_9.i_set(9);

        cpu.load_instruction( 1, &jump_to_9);
        cpu.load_instruction(9, &jump_to_21);
        cpu.load_instruction(21, &jump_to_1);

        cpu.program_counter = 1;
        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(9, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(21, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn test_jump_rd_instruction() {
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 0;
        // Fill with Intrupts
        for i in 0..15 {
            let mut intrupt = Instruction::from_opcode(32);
            intrupt.i_set(47);
            cpu.load_instruction(i * 4, &intrupt);
        }

        let mut jump_to_0 = Instruction::from_opcode(30);
        let mut jump_to_6 = Instruction::from_opcode(30);
        let mut jump_to_2 = Instruction::from_opcode(30);
        println!("Set 1");
        // We will jump to the value in register 10
        jump_to_0.r_dest_set(1);
        // We set the value of register 10 to 1
        cpu.write(1, 0);
        // So this instruction will just to instruction 1
        println!("Set 21");
        jump_to_6.r_dest_set(2);
        cpu.write(2, 24);
        println!("Set 9");
        jump_to_2.r_dest_set(3);
        cpu.write(3, 8);

        cpu.load_instruction(0, &jump_to_2);
        cpu.load_instruction(8, &jump_to_6);
        cpu.load_instruction(24, &jump_to_0);

        println!("Before\n{cpu}");

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(8, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(24, cpu.program_counter);

        cpu = match cpu.clock() {
            UnknownCpu::Inter(cpu) => panic!("Unepected interupt {}", cpu),
            UnknownCpu::Ok(cpu) => cpu,
        };
        assert_eq!(0, cpu.program_counter);
    }

    #[test]
    fn test_memory_setting() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            let mut values = [0;memory::MEMORY_SIZE as usize];
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
            for address in 10..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(17);
                instruction.r_target_set(1);
                instruction.r_base_set(5);
                instruction.i_offset_set(0);
                cpu.write(5, address);
                cpu.load_instruction(1, &instruction);
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
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
            let mut values = [43; 32 as usize];
            for i in 0..values.len() {
                values[i] = i as u8
            }
            values.try_fill(&mut rng).unwrap();
            for (value, address) in values.iter().zip(0..) {
                cpu.memory.write(address, *value).ok();
                //println!("Wrote {} to  {}", value, address);
                //println!("data {}", cpu.memory);
            }
            for address in 6..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(17);
                instruction.r_target_set(1);
                instruction.r_base_set(5);
                instruction.i_offset_set(address as u32);
                cpu.write(5, 0);
                cpu.load_instruction(1, &instruction);
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
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
            let mut values = [43; MEMORY_SIZE as usize];
            for i in 0..values.len() {
                values[i] = i as u8
            }
            values.try_fill(&mut rng).unwrap();
            for (value, address) in values.iter().zip(0..) {
                cpu.memory.write(address, *value).ok();
                //println!("Wrote {} to  {}", value, address);
                //println!("data {}", cpu.memory);
            }
            for address in 6..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(19);
                instruction.r_target_set(1);
                instruction.r_base_set(5);
                cpu.write(5, address);
                instruction.i_offset_set(0);
                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = cpu.read(1) << 4 | cpu.read(2);
                let expected = values[address as usize] << 4 | values[address as usize + 1];
                assert_eq!(expected, value, "{}", instruction);
            }
        }
    }

    #[test]
    fn test_ld32_bo_base() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for _ in 0..10 { 
            let mut values = [43;MEMORY_SIZE as usize];
            for i in 0..values.len() {
                values[i] = i as u8
            }
            values.try_fill(&mut rng).unwrap();
            for (value, address) in values.iter().zip(0..) {
                cpu.memory.write(address, *value).ok();
                //println!("Wrote {} to  {}", value, address);
                //println!("data {}", cpu.memory);
            }
            for address in 5..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(21);
                instruction.r_target_set(1);
                instruction.r_base_set(5);
                cpu.write(5, address);
                instruction.i_offset_set(0);
                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                println!("|>|>{}\n", instruction);
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = 
                    (cpu.read(1) as u32) << 12 | 
                    (cpu.read(2) as u32) << 8 |
                    (cpu.read(3) as u32) << 4 |
                    (cpu.read(4) as u32);
                let expected = 
                    (values[address as usize] as u32) << 12 | 
                    (values[address as usize + 1] as u32) << 8 |
                    (values[address as usize + 2] as u32) << 4 |
                    (values[address as usize + 3] as u32);
                assert_eq!(expected, value, "Inst:{}\nexp:{:X}\nval:{:X}", instruction, expected, value);
            }
        }
    }

    #[test]
    fn test_st_bo_base() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 8..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(23);
                instruction.r_target_set(5);
                instruction.r_base_set(6);
                instruction.i_offset_set(0);
                let rand_value = rng.gen();
                cpu.write(5, rand_value);
                cpu.write(6, address);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = cpu.memory.read(address).unwrap();
                assert_eq!(rand_value, value, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }

    #[test]
    fn test_st_bo_offset() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 8..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(23);
                instruction.r_target_set(5);
                instruction.r_base_set(6);
                instruction.i_offset_set(address as u32);
                let rand_value = rng.gen();
                cpu.write(5, rand_value);
                cpu.write(6, 0);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = cpu.memory.read(address).unwrap();
                assert_eq!(rand_value, value, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }

    #[test]
    fn test_st_bi() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 8..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(24);
                instruction.r_target_set(5);
                instruction.r_base_set(6);
                instruction.r_index_set(7);
                let rand_value = rng.gen();
                cpu.write(5, rand_value);
                cpu.write(6, address);
                cpu.write(7, 0);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = cpu.memory.read(address).unwrap();
                assert_eq!(rand_value, value, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }

    #[test]
    fn test_st_16_bo_offset() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 8..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(25);
                instruction.r_target_set(5);
                instruction.r_base_set(7);
                instruction.i_offset_set(address as u32);
                let rand_value: u16 = rng.gen();
                cpu.write(5, (rand_value & 0xFF) as u8);
                cpu.write(6, (rand_value >> 8) as u8 & 0xFF );
                cpu.write(7, 0);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = (cpu.memory.read(address).unwrap() as u16) |
                    ((cpu.memory.read(address + 1).unwrap()) as u16) << 8;
                assert_eq!(rand_value, value as u16, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }

    #[test]
    fn test_st_32_bo_offset() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 10..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(27);
                instruction.r_target_set(5);
                instruction.r_base_set(9);
                instruction.i_offset_set(address as u32);
                let rand_value: u32 = rng.gen();
                cpu.write(5, (rand_value       & 0xFF) as u8);
                cpu.write(6, (rand_value >>  8 & 0xFF) as u8);
                cpu.write(7, (rand_value >> 16 & 0xFF) as u8);
                cpu.write(8, (rand_value >> 24 & 0xFF) as u8);
                cpu.write(9, 0);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = (cpu.memory.read(address).unwrap() as u32)   |
                    (cpu.memory.read(address + 1).unwrap() as u32) <<  8 | 
                    (cpu.memory.read(address + 2).unwrap() as u32) << 16 |
                    (cpu.memory.read(address + 3).unwrap() as u32) << 24;
                assert_eq!(rand_value, value as u32, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }

    #[test]
    fn test_st_16_bi() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 8..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(26);
                instruction.r_target_set(5);
                instruction.r_base_set(7);
                instruction.r_index_set(8);
                let rand_value: u16 = rng.gen();
                cpu.write(5, (rand_value & 0xFF) as u8);
                cpu.write(6, ((rand_value >> 8) & 0xFF) as u8);
                cpu.write(7, address);
                cpu.write(8, 0);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = cpu.memory.read(address).unwrap() as u16 | 
                    ((cpu.memory.read(address + 1).unwrap() as u16) << 8);
                assert_eq!(rand_value, value, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }

    #[test]
    fn test_st_32_bi() {
        let mut cpu = Cpu::new_blank();
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            for address in 11..31 {
                println!("{}", address);
                let mut instruction = Instruction::from_opcode(28);
                instruction.r_target_set(5);
                instruction.r_base_set(9);
                instruction.r_index_set(10);
                let rand_value: u32 = rng.gen();
                cpu.write(5, (rand_value         & 0xFF) as u8);
                cpu.write(6, ((rand_value >>  8) & 0xFF) as u8);
                cpu.write(7, ((rand_value >> 16) & 0xFF) as u8);
                cpu.write(8, ((rand_value >> 24) & 0xFF) as u8);
                cpu.write(9, address);
                cpu.write(10, 0);

                cpu.load_instruction(1, &instruction);
                cpu.program_counter = 1;
                cpu = match cpu.clock() {
                    UnknownCpu::Ok(cpu) => cpu,
                    UnknownCpu::Inter(cpu) => panic!("Unexpected intrrupt {}", cpu),
                };

                let value = cpu.memory.read(address).unwrap() as u32 | 
                    ((cpu.memory.read(address + 1).unwrap() as u32) << 8) |
                    ((cpu.memory.read(address + 2).unwrap() as u32) << 16)|
                    ((cpu.memory.read(address + 3).unwrap() as u32) << 24);
                assert_eq!(rand_value, value, "{}:{}|Inst:{}\nexp:{:X}\nval:{:X}", i, address, instruction, rand_value, value);
            }
        }
    }
    
    #[test]
    fn test_logic_left_shift_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(0);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x01);
        cpu.write(7, 1);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x02, cpu.read(5));
    }

    #[test]
    fn test_logic_left_shift_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(1);
        instruction.r_dest_set(5);
        // Value
        instruction.r_x_set(6);
        cpu.write(6, 0x01);
        // Shift
        instruction.i_y_set(2);
        cpu.write(7, 1);
        cpu.load_instruction(1, &instruction);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x04, cpu.read(5));
    }

    #[test]
    fn test_logic_left_shift_ri_negitive() {
        // If shift value is negative then nothing should happen
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(0);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(0x801);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x01);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x01, cpu.read(5));
    }

    #[test]
    fn test_logic_right_shift_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(2);  
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        println!("{instruction}");
        cpu.write(6, 0x02);
        cpu.write(7, 1);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x01, cpu.read(5));
    }

    #[test]
    fn test_logic_right_shift_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(3);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(1);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x02);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x01, cpu.read(5));
    }

    #[test]
    fn test_logic_left_shift_overflow() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(0);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x80);
        cpu.write(7, 1);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x00, cpu.read(5));
    }

    #[test]
    fn test_add() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x0F);
        cpu.write(7, 1);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x10, cpu.read(5));
    }

    #[test]
    fn test_add_multiple() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        instruction.r_dest_set(1);
        instruction.r_x_set(2);
        instruction.r_y_set(3);
        cpu.load_instruction(0, &instruction);
        for i in 0..100 {
            cpu.program_counter = 0;
            cpu.write(2, i);
            cpu.write(3, i+13);
            cpu = match cpu.clock() {
                UnknownCpu::Ok(ok) => ok,
                UnknownCpu::Inter(_) => panic!()
            };
            assert_eq!((2*i+13) & 0xFF, cpu.read(1));
        }
    }

    #[test]
    fn test_rd_function_incs_program_counter() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        instruction.r_dest_set(1);
        instruction.r_x_set(2);
        instruction.r_y_set(3);
        cpu.load_instruction(0, &instruction);
        cpu.program_counter = 0;
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(4, cpu.program_counter);
    }

    #[test]
    fn test_add_with_overflow() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0xFF);
        cpu.write(7, 2);
        //TODO ADD check for overflow flag
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x01, cpu.read(5));
    }

    #[test]
    fn test_add_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(1);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x0F);
        cpu.write(7, 1);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x10, cpu.read(5));
    }

    #[test]
    fn test_add_ri_negative_i() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(1 | (1 << 11));
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0xF1);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0xF0, cpu.read(5));
    }

    #[test]
    fn test_add_multiple_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);
        for i in 0..100 {
            cpu.program_counter = 0;
            instruction.r_dest_set(5);
            instruction.r_x_set(6);
            cpu.write(6, i);
            instruction.i_y_set((i+13).into());
            cpu.load_instruction(0, &instruction);
            cpu = match cpu.clock() {
                UnknownCpu::Ok(ok) => ok,
                UnknownCpu::Inter(_) => panic!()
            };
            assert_eq!((2*i+13) & 0xFF, cpu.read(5));
        }
    }

    #[test]
    fn test_add_with_overflow_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(2);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0xFF);
        cpu.write(7, 2);
        //TODO ADD check for overflow flag
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x01, cpu.read(5));
    }

    #[test]
    fn test_sub_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(13);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0xF4);
        cpu.write(7, 4);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0xF0, cpu.read(5));
    }

    #[test]
    fn test_sub_multiple_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(13);
        for i in 0..100 {
            instruction.r_dest_set(5);
            instruction.r_x_set(6);
            cpu.write(6, i*2+26);
            instruction.r_y_set(7);
            cpu.write(7, i+13);
            cpu.load_instruction(1, &instruction);
            cpu.program_counter = 1;
            cpu = match cpu.clock() {
                UnknownCpu::Ok(ok) => ok,
                UnknownCpu::Inter(_) => panic!()
            };
            assert_eq!((i+13) & 0xFF, cpu.read(5));
        }
    }

    #[test]
    fn test_sub_with_underflow_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(13);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        cpu.write(6, 0x00);
        instruction.r_y_set(7);
        cpu.write(7, 2);
        cpu.load_instruction(1, &instruction);
        //TODO ADD check for overflow flag
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0xFE, cpu.read(5));
    }

    #[test]
    fn test_sub_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(14);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        cpu.write(6, 0xF4);
        instruction.i_y_set(4);
        cpu.load_instruction(1, &instruction);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0xF0, cpu.read(5));
    }

    #[test]
    fn test_sub_multiple_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(14);
        for i in 0..100 {
            instruction.r_dest_set(5);
            instruction.r_x_set(6);
            cpu.write(6, i*2+26);
            instruction.i_y_set((i+13).into());
            cpu.load_instruction(1, &instruction);
            cpu.program_counter = 1;
            cpu = match cpu.clock() {
                UnknownCpu::Ok(ok) => ok,
                UnknownCpu::Inter(_) => panic!()
            };
            assert_eq!((i+13) & 0xFF, cpu.read(5));
        }
    }

    #[test]
    fn test_sub_with_underflow_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(14);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        cpu.write(6, 0x00);
        instruction.i_y_set(2);
        cpu.load_instruction(1, &instruction);
        //TODO ADD check for overflow flag
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0xFE, cpu.read(5));
    }
    //TODO Add multiply tests
    #[test]
    fn test_multipy() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(15);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 6);
        cpu.write(7, 7);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(6*7, cpu.read(5));
    }

    #[test]
    fn test_multiply_multiple() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(15);
        for i in 0..100 {
            cpu.program_counter = 0;
            instruction.r_dest_set(5);
            instruction.r_x_set(6);
            instruction.r_y_set(7);
            cpu.load_instruction(0, &instruction);
            cpu.write(6, i);
            cpu.write(7, i+13);
            cpu = match cpu.clock() {
                UnknownCpu::Ok(ok) => ok,
                UnknownCpu::Inter(_) => panic!()
            };
            assert_eq!(i.wrapping_mul(i+13) & 0xFF, cpu.read(5));
        }
    }

    #[test]
    fn test_multiply_with_overflow() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(15);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.r_y_set(7);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0xF0);
        cpu.write(7, 0x0F);
        //TODO ADD check for overflow flag
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x10, cpu.read(5));
    }

    #[test]
    fn test_multply_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(16);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(6);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x7);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(6*7, cpu.read(5));
    }

    //TODO What happens when we multply with a negative number?

    #[test]
    fn test_mutiply_multiple_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(16);
        for i in 0..100 {
            instruction.r_dest_set(5);
            instruction.r_x_set(6);
            cpu.write(6, i);
            instruction.i_y_set((i+13).into());
            cpu.load_instruction(0, &instruction);
            cpu.program_counter = 0;

            cpu = match cpu.clock() {
                UnknownCpu::Ok(ok) => ok,
                UnknownCpu::Inter(_) => panic!()
            };
            let y = i + 13;
            assert_eq!(i.wrapping_mul(i+13) & 0xFF, cpu.read(5), "{i} * {y} isn't correct");
        }
    }

#[test]
    fn test_multiply_with_overflow_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(16);
        instruction.r_dest_set(5);
        instruction.r_x_set(6);
        instruction.i_y_set(0xF0);
        cpu.load_instruction(1, &instruction);
        cpu.write(6, 0x0F);
        //TODO ADD check for overflow flag
        cpu = match cpu.clock() {
            UnknownCpu::Ok(ok) => ok,
            UnknownCpu::Inter(_) => panic!()
        };
        assert_eq!(0x10, cpu.read(5));
    }

    #[test]
    fn test_carry_flag_set_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0xFF);
       instruction.i_y_set(42);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(cpu.flags.carry);
    }

    #[test]
    fn test_carry_flag_not_set_ri() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x4);
       instruction.i_set(2);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(!cpu.flags.carry);
    }

    #[test]
    fn test_carry_flag_set_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(12);

       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x4);
       instruction.r_y_set(3);
       cpu.write(2, 0xFF);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(cpu.flags.carry);
    }

    #[test]
    fn test_carry_flag_not_set_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x2);
       instruction.r_y_set(3);
       cpu.write(2, 0x32);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(!cpu.flags.carry);
    }

    #[test]
    fn test_greater_flag_set_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x2);
       instruction.r_y_set(3);
       cpu.write(3, 0x32);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(cpu.flags.greater);
    }
    #[test]
    fn test_greater_flag_not_set_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x0);
       instruction.r_y_set(3);
       cpu.write(3, 0x0);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(!cpu.flags.greater);
    }
    /*
    #[test]
    fn test_lesser_flag_set_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(13);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x2);
       instruction.r_y_set(3);
       cpu.write(3, 0x4);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(cpu.flags.greater);
    }

    #[test]
    fn test_lesser_flag_not_set_rd() {
        let mut cpu = Cpu::new_blank();
        let mut instruction = Instruction::from_opcode(11);
        
       instruction.r_dest_set(1);
       instruction.r_x_set(2);
       cpu.write(2, 0x3);
       instruction.r_y_set(3);
       cpu.write(3, 0x1);
       cpu.load_instruction(1, &instruction);
       cpu = match cpu.clock() {
           UnknownCpu::Ok(ok) => ok,
           UnknownCpu::Inter(_) => panic!()
       };
       assert!(!cpu.flags.greater);
    }
    */
    #[test]
    fn test_flag_skips_instuction() {
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 0;

        // 10. Add 1 + 1 
        let mut i10 = Instruction::from_opcode(11);
        i10.r_x_set(1);
        cpu.write(1, 1);
        i10.r_y_set(2);
        cpu.write(2, 1);
        i10.r_dest_set(3);
        
        // 20. Jump if Zero to 5 
        let mut i20 = Instruction::from_opcode(31);
        i20.i_set(20);
        i20.flags.set_all_flags(false);
        i20.flags.zero = true;
        // 30. Interrupt
        let i30 = Instruction::from_opcode(32);
        // 40. Nothing
        // 50. Interrupt
        let i50 = Instruction::from_opcode(32);
        // Should be at 30
        
       cpu.load_instruction(0, &i10);
       cpu.load_instruction(4, &i20);
       cpu.load_instruction(8, &i30);
       cpu.load_instruction(20, &i50);
       let mut count = 0;
       cpu = loop {
           println!("PC:{}|| {}", cpu.program_counter, cpu.current_instruction());
           count += 1;
           if count > 10 {
               println!("{}", cpu);

               panic!("Program is suck in a loop")
           };
           cpu = match cpu.clock() {
               UnknownCpu::Ok(ok) => ok,
               UnknownCpu::Inter(ok) => break ok
           };
       };
       assert_ne!(cpu.program_counter, 20, "Program jumped even though last result wasn't Zero");
       assert_eq!(cpu.program_counter, 8, "Program didn't jump expected location");
        
    }

    #[test]
    fn test_flag_zero_gets_set() {
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 0;

        // 10. Add 0 + 0
        let mut i10 = Instruction::from_opcode(11);
        i10.r_x_set(1);
        i10.r_y_set(2);
        i10.r_dest_set(3);
        cpu.write(1, 0);
        cpu.write(2, 0);
        cpu.load_instruction(0, &i10);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(cpu) => cpu,
            UnknownCpu::Inter(_) => panic!(),
        };
        assert!(cpu.flags.zero);
    }

    #[test]
    fn test_flag_zero_does_not_gets_set() {
        let mut cpu = Cpu::new_blank();
        cpu.program_counter = 0;

        // 10. Add 1 + 1
        let mut i10 = Instruction::from_opcode(11);
        i10.r_x_set(1);
        i10.r_y_set(2);
        i10.r_dest_set(3);
        cpu.load_instruction(0, &i10);
        cpu.write(1, 1);
        cpu.write(2, 1);

        cpu = match cpu.clock() {
            UnknownCpu::Ok(cpu) => cpu,
            UnknownCpu::Inter(_) => panic!(),
        };
        assert!(!cpu.flags.zero);
    }
}   
