
#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub carry: bool,
    pub less_than_zero: bool,
    pub equal_to_zero: bool,
    pub greater_than_zero: bool,
    pub opcode: u8,
    pub operands: u32
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Flags

        write!(fmt, "|{}", if self.carry {"C"} else {"-"}).ok();
        write!(fmt, "{}", if self.less_than_zero {"N"} else {"-"}).ok();
        write!(fmt, "{}", if self.equal_to_zero {"Z"} else {"-"}).ok();
        write!(fmt, "{}", if self.greater_than_zero {"P"} else {"-"}).ok();

        // Opcode
        let opcode: String = format!("{}", self.opcode);
        write!(fmt, "|{:9}|", match self.opcode {
            
            29 => "Jump offset",
            30 => "Jump to",
            31 => "Jump to",
            32 => "Interupt",
            _ => opcode.as_str()
        }).ok();

        // Operands
        write!(fmt, "{:9}|", self.operands).ok();

        Ok(())
    }

}

impl Instruction {
    pub fn from_opcode(opcode: u8, operands: u32) -> Self {
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
    pub fn encode(&self) -> u32 {
        // TODO move this earlier
        if self.opcode > 0x3F {
            panic!("opcode too large");
        };
        if self.operands > 0x1FFFFF {
            panic!("opcode too large");
        };
        (if self.carry             {(1_u32) << 31} else{ 0 }) |
        (if self.less_than_zero    {1<< 30} else{ 0 }) as u32 |
        (if self.equal_to_zero     {1<< 29} else{ 0 }) as u32 |
        (if self.greater_than_zero {1<< 28} else{ 0 }) as u32 |
        (self.opcode as u32)            << 22 |
        self.operands
    }

    pub fn decode(value: u32) -> Self {
        println!(">> {}", value >> 30);
        Instruction {
            carry:             value >> 31 == 1,
            less_than_zero:   (value >> 30) & 1 == 1,
            equal_to_zero:    (value >> 29) & 1 == 1,
            greater_than_zero:(value >> 28) & 1 == 1,
            opcode:    (value >> 22 & 0x3F) as u8,
            operands:   value       & 0x3FFFFF,
        }
    }

    pub fn r_dest(&self) -> u8 {
        (self.operands >> 17) as u8        
    }

    pub fn r_target(&self) -> u8 {
        self.r_dest()
    }

    pub fn r_x(&self) -> u8 {
        ((self.operands >> 12) & 0x1F) as u8        
    }

    pub fn r_base(&self) -> u8 {
        self.r_x()
    }

    pub fn r_y(&self) -> u8 {
        ((self.operands >> 7) & 0x1F) as u8        
    }

    pub fn r_index(&self) -> u8 {
        self.r_y()
    }

    pub fn i_y(&self) -> i16 {
        if self.operands & 0x800 != 0 {
            0 - (self.operands & 0x7FF) as i16
        } else {
            (self.operands & 0x7FF) as i16        
        }
    }

    pub fn i_offset(&self) -> u32 {
        (self.operands & 0xFFF) as u32
    }

    pub fn i(&self) -> i32 {
        if self.operands & 0x200000 == 0 {
            (self.operands & 0x1FFFFF) as i32
        } else {
            println!("{}:{}", i32::MIN, (self.operands & 0x1FFFFF) as i32);
            0_i32 - ((self.operands & 0x1FFFFF) as i32)
        }
    

    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_interupt() {
        let instruction = Instruction::from_opcode(32, 0xABC);
        let target = 0x0800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");
    }
    
    #[test]
    fn encode_flags() {
        let mut instruction = Instruction::from_opcode(32, 0xABC);
        instruction.carry = true;
        let target = 0x8800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");

        instruction.less_than_zero = true;
        let target = 0xC800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");

        instruction.equal_to_zero = true;
        let target = 0xE800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");

        instruction.greater_than_zero = true;
        let target = 0xF800_0ABC;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");
    }

    #[test]
    fn decode_flags() {
        let mut instruction = Instruction::from_opcode(32, 0xABC);
        instruction.carry = true;
        let mut target = 0x8800_0ABC;
        let mut result = Instruction::decode(target);
        assert_eq!(instruction, result);

        instruction.less_than_zero = true;
        target = 0xC800_0ABC;
        result = Instruction::decode(target);
        assert_eq!(instruction, result);

        instruction.equal_to_zero = true;
        target = 0xE800_0ABC;
        result = Instruction::decode(target);
        assert_eq!(instruction, result);

        instruction.greater_than_zero = true;
        target = 0xF800_0ABC;
        result = Instruction::decode(target);
        assert_eq!(instruction, result);
    }

    #[test]
    fn encode_jump_offset() {
        let instruction = Instruction::from_opcode(31, 0xDCA);
        let target = 0x07C0_0DCA;
        let result = instruction.encode();
        assert_eq!(target, result, "\nT: {target:08x}\nR: {result:08x}");
    }

    #[test]
    fn operand_encoding_rd() {
        let i = Instruction::from_opcode(0, 0b11011_10101_00000_1111111);
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
        let i = Instruction::from_opcode(0, 0b00100_01010_11111_0000000);
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
}

