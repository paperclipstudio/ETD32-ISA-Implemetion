#[allow(unused_imports)]
use rand::Rng;


// How to check to see if that values are too large for real instruction????
// As in i_set we check the size of value but this would be harder to do at 
// initalision of an enum.
//
// Like this??
struct U5 {
    value:u8
}

impl U5 {
    fn new(value:u8) -> Self {
      U5 {value}
    }

    fn set(&mut self, value:u8) {
        self.value = value & 0x1F
    }

    fn get(self) -> u8 {
          self.value
    }
}

struct Rd {
    dest:u8,
    x:U5,
    y:u8
}

struct Bo {
    target:u8,
    base:u8,
    offset:u8,
}

enum Alt {
    AddRd(Rd),
    SubRd(Rd),
    Jump(u32),
    Move(Bo),
}

impl Alt {
    fn test() { 
        let mut t = Alt::AddRd(Rd{
            dest:2,
            x:U5::new(5),
            y:6
        });
        match t {
            Alt::AddRd(ref mut operands) => operands.x.set(3),
            _ => (),
        };

        let val = match t {
            Alt::AddRd(operands) => operands.x.get(),
            _ => 0,
        };
        println!("{val}");

    }
}

/// 
///

struct U52 {
    value:u8
}

impl U52 {
    fn new(value:u8) -> Self {
      U52 {value}
    }

    fn set(&mut self, value:u8) {
        self.value = value & 0x1F
    }

    fn get(self) -> u8 {
          self.value
    }
}

struct Rd2 {
    dest:U52,
    x:U52,
    y:U52
}

struct Bo2 {
    target:u8,
    base:u8,
    offset:u8,
}

enum Memory {
    LD8(Bo2)
}

enum Flow {
    Jump(u32),
}

enum Operation {
    AddRd(Rd),
    SubRd(Rd),
}

enum Test {A, B, C}

enum Alt2 {
    Test(Test),
    Memory(Memory),
    Operation(Operation),
    Flow,
}

impl Alt2 {
    fn test() { 
        let mut t = Alt2::Operation(Operation::AddRd(Rd{
            dest:2,
            x:U5::new(5),
            y:6
        }));
        match t {
            Alt2::Operation(ref mut oper) => match oper {
                Operation::AddRd(ref mut rd) => rd.x.set(5),
                _ => (),
            },
            _ => (),

        };

        let val = match t {
            Alt2::Operation(oper) => match oper {
                Operation::AddRd(rd) => rd.x.get(),
                _ => 0,
            },
            _ => 0,
        };

        let r = Alt2::Test(Test::A);
    }
}

pub const NEGITIVE_BIT: u32 = 1 << 21;
#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub carry: bool,
    pub less_than_zero: bool,
    pub equal_to_zero: bool,
    pub greater_than_zero: bool,
    pub opcode: u8,
    operands: u32
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
        write!(fmt, "|{:20}|", match self.opcode {
            17 => "Load from Mem BO",
            29 => "Jump offset",
            30 => "Jump to Rd",
            31 => "Jump to I",
            32 => "Interupt",
            _ => opcode.as_str()
        }).ok();

        // Operands
        match (self.opcode, self.opcode % 2 == 0) {
            // Logic Rd
            (0..=16, false)             => write!(fmt, "{:4}:{:4}:{:4}:{:4}|", self.r_dest(), self.r_x(), self.r_y(), 0).ok(),
            (0..=16, true)              => write!(fmt, "{:4}:{:4}:{:9}|", self.r_dest(), self.r_x(), self.i_y()).ok(),
            (17..=28, false)            => write!(fmt, "{:4}:{:4}:{:9}|", self.r_target(), self.r_base(), self.i_offset()).ok(),
            (17..=28, true)             => write!(fmt, "Memory             |").ok(),
            (29, _) | (31, _) | (32, _) => write!(fmt, "{:16}|", self.opcode).ok(),
            (30, _)                     => write!(fmt, "{:16}|", self.r_dest()).ok(),
            (33..=u8::MAX, _)           => write!(fmt, "{:16}|", self.opcode).ok(),
        };


  //      write!(fmt, "{:9}|", self.operands).ok();

        Ok(())
    }

}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Self {
       Instruction {
            carry: false,
            less_than_zero: false,
            equal_to_zero: false,
            greater_than_zero: false,
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
        (if self.carry             {(1_u32) << 31} else{ 0 }) |
        (if self.less_than_zero    {1<< 30} else{ 0 }) as u32 |
        (if self.equal_to_zero     {1<< 29} else{ 0 }) as u32 |
        (if self.greater_than_zero {1<< 28} else{ 0 }) as u32 |
        (self.opcode as u32)            << 22 |
        self.operands
    }

    pub fn decode(value: u32) -> Self {
        Instruction {
            carry:             value >> 31 == 1,
            less_than_zero:   (value >> 30) & 1 == 1,
            equal_to_zero:    (value >> 29) & 1 == 1,
            greater_than_zero:(value >> 28) & 1 == 1,
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
        let mut instruction = Instruction::from_opcode(32);
        instruction.operands = 0xABC;
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

