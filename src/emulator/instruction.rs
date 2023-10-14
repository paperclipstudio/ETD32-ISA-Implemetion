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
        (if self.carry {1<< 31} else{ return 0 }) as u32 |
        (if self.less_than_zero {1<< 30} else{ return 0 }) as u32 |
        (if self.equal_to_zero {1<< 29} else{ return 0 }) as u32|
        (if self.greater_than_zero {1<< 28} else{ return 0 }) as u32 |
        // TODO have way to make sure this doesn't overflow
        (self.opcode            << 22) as u32 |
        self.operands          <<  0
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
    fn fail() {
        assert!(false)
    }
}

