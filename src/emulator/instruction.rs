#[allow(unused_imports)]
use rand::Rng;
use crate::emulator::flags::Flags;

pub const NEGITIVE_BIT: u32 = 1 << 21;
#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub flags: Flags,
    pub opcode: u8,
    operands: u32
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Flags
        write!(fmt, "|{}", if self.flags.carry {"C"} else {"-"}).ok();
        write!(fmt, "{}", if self.flags.less {"N"} else {"-"}).ok();
        write!(fmt, "{}", if self.flags.zero {"Z"} else {"-"}).ok();
        write!(fmt, "{}", if self.flags.greater {"P"} else {"-"}).ok();

        // Opcode
        let opcode: String = format!("{}", self.opcode);
        write!(fmt, "|{:20}|", match self.opcode {
            11 => "Addition RD",
            12 => "Addition RI",
            17 => "Load from Mem BO",
            23 => "Store 8bits",
            29 => "Jump offset",
            30 => "Jump to Rd",
            31 => "Jump to I",
            32 => "Interupt",
            _ => opcode.as_str()
        }).ok();

        // Operands
        match (self.opcode, self.opcode % 2 == 0) {
            // Logic Rd
            (0..=16, false)             => write!(fmt, "{:4}:{:4}:{:4}:    |", self.r_dest(), self.r_x(), self.r_y()).ok(),
            (0..=16, true)              => write!(fmt, "{:4}:{:4}:{:9}|", self.r_dest(), self.r_x(), self.i_y()).ok(),
            (17..=28, false)            => write!(fmt, "{:4}:{:4}:{:9}|", self.r_target(), self.r_base(), self.i_offset()).ok(),
            (17..=28, true)             => write!(fmt, "Memory             |").ok(),
            (29, _) | (31, _) | (32, _) => write!(fmt, "{:19}|", self.i()).ok(),
            (30, _)                     => write!(fmt, "{:19}|", self.r_dest()).ok(),
            (33..=u8::MAX, _)           => write!(fmt, "{:19}|", self.i()).ok(),
        };


  //      write!(fmt, "{:9}|", self.operands).ok();

        Ok(())
    }

}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Self {
       Instruction {
           flags: Flags::new(),
            opcode,
            operands: 0,
       }
    }


    #[allow(arithmetic_overflow)]
    pub fn encode(&self) -> u32 {
        // TODO move this earlier
        if self.opcode > 0x3F {
            panic!("opcode too large");
        };
        if self.operands > 0x3FFFFF {
            panic!("operands too large {:X}", self.operands);
        };
        (if self.flags.carry             {(1_u32) << 31} else{ 0 }) |
        (if self.flags.less    {1<< 30} else{ 0 }) as u32 |
        (if self.flags.zero     {1<< 29} else{ 0 }) as u32 |
        (if self.flags.greater {1<< 28} else{ 0 }) as u32 |
        (self.opcode as u32)            << 22 |
        self.operands
    }

    pub fn decode(value: u32) -> Self {
        let flags = Flags {
            carry:             value >> 31 == 1,
            less:   (value >> 30) & 1 == 1,
            zero:    (value >> 29) & 1 == 1,
            greater:(value >> 28) & 1 == 1,
        };
        Instruction {
            flags,
            opcode:    (value >> 22 & 0x3F) as u8,
            operands:   value       & 0x3FFFFF,
        }
    }

    // Returns 21-17
    fn first(&self) -> u8 {
        (self.operands >> 17) as u8        
    }

    // Returns 16-12
    fn second(&self) -> u8 {
        ((self.operands >> 12) & 0x1F) as u8        
    }

    // Returns 11-7
    fn third(&self) -> u8 {
        ((self.operands >> 7) & 0x1F) as u8        
    }

    // Returns 6-0
    fn fourth(&self) -> u8 {
        (self.operands & 0x1F) as u8        
    }


    // Should restructer this...... 
    // Yuck

    pub fn r_dest(&self) -> u8 {
        self.first()
    }

    pub fn r_dest_set(&mut self, value:u8) {
        // TODO add tests
        if value > 0x1F {
            panic!("Value too large")
        }
        self.operands &= !(0x1F << 16);
        self.operands |= (value as u32) << 17;
    }

    pub fn r_target(&self) -> u8 {
        self.r_dest()
    }

    pub fn r_target_set(&mut self, value: u8) {
        self.r_dest_set(value)
    }

    pub fn r_x(&self) -> u8 {
        self.second()
    }

    pub fn r_x_set(&mut self, value: u8) {
        //TODO add tests
        self.operands &= (!(0x1F << 12)) as u32;
        self.operands |= (0x1F & value as u32) << 12;
    }

    pub fn r_base(&self) -> u8 {
        self.r_x()
    }

    pub fn r_base_set(&mut self, value:u8) {
        self.r_x_set(value)
    }

    pub fn r_y(&self) -> u8 {
        ((self.operands >> 7) & 0x1F) as u8        
    }

    pub fn r_y_set(&mut self, value:u8) {
        self.operands &= (!(0x1F << 7)) as u32;
        self.operands |= (0x1F & value as u32) << 7;
    }

    pub fn r_index(&self) -> u8 {
        self.r_y()
    }

    pub fn r_index_set(&mut self, value:u8) {
        self.r_y_set(value)
    }
    
    //////
    pub fn i_y(&self) -> i16 {
        if self.operands & 0x800 == 0 {
            (self.operands & 0x7FF) as i16        
        } else {
            0 - (self.operands & 0x7FF) as i16
        }
    }

    pub fn i_y_set(&mut self, value:u16) {
        self.operands &= !0xFFF;
        self.operands |= (value & 0xFFF) as u32;
    }

    pub fn i_offset(&self) -> u32 {
        self.operands & 0xFFF
    }

    pub fn i_offset_set(&mut self, value: u32) {
        if value > 0xFFF{
            panic!("value is too large")
        }
        //TODO add tests
        self.operands &= !0xFFF;
        self.operands |= 0xFFF & value;
    }

    pub fn i(&self) -> i32 {
        if self.operands & 0x200000 == 0 {
            (self.operands & 0x1FFFFF) as i32
        } else {
            println!("{}:{}", i32::MIN, (self.operands & 0x1FFFFF) as i32);
            0_i32 - ((self.operands & 0x1FFFFF) as i32)
        }
    }

    pub fn i_set(&mut self, mut value: i32) {
        self.operands = 0;
        if value > 0 {
            self.operands |= NEGITIVE_BIT;
        } 
        // maybe throw error here if value is too large?
        value &= 0x1FFFFF;
        self.operands = value.abs() as u32;
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_interupt() {
        let mut instruction = Instruction::from_opcode(32);
        instruction.operands = 0xABC;

        let target = 0x0800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");
    }
    
    #[test]
    fn encode_flags() {
        let mut instruction = Instruction::from_opcode(32);
        instruction.operands = 0xABC;
        instruction.flags.carry = true;
        let target = 0x8800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");

        instruction.flags.less = true;
        let target = 0xC800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");

        instruction.flags.zero = true;
        let target = 0xE800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");

        instruction.flags.greater = true;
        let target = 0xF800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");
    }

    #[test]
    fn decode_flags() {
        let mut instruction = Instruction::from_opcode(32);
        instruction.operands = 0xABC;
        instruction.flags.carry = true;
        let mut target = 0x8800_0ABC;
        let mut result = Instruction::decode(target);
        assert_eq!(instruction, result);

        instruction.flags.less = true;
        target = 0xC800_0ABC;
        result = Instruction::decode(target);
        assert_eq!(instruction, result);

        instruction.flags.zero = true;
        target = 0xE800_0ABC;
        result = Instruction::decode(target);
        assert_eq!(instruction, result);

        instruction.flags.greater = true;
        target = 0xF800_0ABC;
        result = Instruction::decode(target);
        assert_eq!(instruction, result);
    }

    #[test]
    fn encode_jump_offset() {
        let mut instruction = Instruction::from_opcode(31);
        instruction.operands = 0xDCA;
        let target = 0x07C0_0DCA;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");
    }

    #[test]
    fn operand_encoding_rd() {
        let mut i = Instruction::from_opcode(0);
        i.operands = 0b11011_10101_00000_1111111;
        assert_eq!(i.r_dest(), 0b11011);
        assert_eq!(i.r_x(), 0b10101);
        assert_eq!(i.r_y(), 0b00000);
        assert_eq!(i.i_y(), 0b00000_1111111);
        assert_eq!(i.r_target(), 0b11011);
        assert_eq!(i.r_base(),   0b10101);
        assert_eq!(i.i_offset(), 127);
        assert_eq!(i.r_index(), 0b00000);
        assert_eq!(i.i(), -1527935);
    }


    #[test]
    fn operand_encoding_negitive() {
        let mut i = Instruction::from_opcode(0);
        i.operands = 0b00100_01010_11111_0000000;
        assert_eq!(i.r_dest(), 0b00100);
        assert_eq!(i.r_x(), 0b01010);
        assert_eq!(i.r_y(), 31);
        assert_eq!(i.i_y(), -1920);
        assert_eq!(i.r_target(), 0b00100);
        assert_eq!(i.r_base(),   0b01010);
        assert_eq!(i.i_offset(), 3968);
        assert_eq!(i.r_index(), 0b11111);
        assert_eq!(i.i(), 569216);
    }

    #[test] 
    fn i_offset_set() {
        let mut rng = rand::thread_rng();
        let mut instruction = Instruction::from_opcode(0);
        instruction.i_offset_set(0);
        assert_eq!(0, instruction.i_offset(), "Failed on value {}", 0);
        for i in 0..100 {
            let value = rng.gen::<u32>() & 0xFFF;
            instruction.i_offset_set(value);
            assert_eq!(value, instruction.i_offset(), "Failed on value {}, on the {} test", value, i);
        }
    }
}

