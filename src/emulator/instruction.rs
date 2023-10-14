#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub carry: bool,
    pub less_than_zero: bool,
    pub equal_to_zero: bool,
    pub greater_than_zero: bool,
    pub opcode: u8,
    pub operands: u32
}

// TODO Move into own class and test
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
    // TODO Add tests?
    // TODO Add checks for oversized values
    // TODO Must be a nicer way to do this....
        if self.opcode > 0x3F {
            panic!("opcode too large");
        };
        // TODO move this earlier
        if self.operands > 0x1FFFFF {
            panic!("opcode too large");
        };
        let num = (if self.carry             {(1 as u32) << 31} else{ 0 }) |
        (if self.less_than_zero    {1<< 30} else{ 0 }) as u32 |
        (if self.equal_to_zero     {1<< 29} else{ 0 }) as u32 |
        (if self.greater_than_zero {1<< 28} else{ 0 }) as u32 |
        // TODO have way to make sure this doesn't overflow
        (self.opcode as u32)            << 22 |
        self.operands;
        return num;
    }

    pub fn decode(value: u32) -> Self {
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
}

