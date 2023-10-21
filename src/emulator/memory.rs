use std::fmt;
use rand::Rng;
use rand::Fill;

pub const MEMORY_SIZE: u8 = 255;
// const MEMORY_SIZE: usize = 10;

#[derive(Debug)]
pub struct SimpleMemory {
    data: [u8; MEMORY_SIZE as usize]
}

pub trait Memory {
    fn read(&self, address: u8) -> Option<u8>;
    fn write(&mut self, address:u8, value: u8) -> Result<(), &'static str>;
}

impl SimpleMemory {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        SimpleMemory {
            data: [rng.gen(); MEMORY_SIZE as usize]
        }
    }

    pub fn new_blank() -> Self {
        SimpleMemory {
            data: [0;MEMORY_SIZE as usize]
        }
    }
}

impl Memory for SimpleMemory {
    fn read(&self, address: u8) -> Option<u8> {
        return if address < self.data.len() as u8 {
            Some(self.data[address as usize])
        } else {
           None 
        }
    }

    fn write(&mut self, address:u8, value: u8) -> Result<(), &'static str> {
        return if address >= self.data.len() as u8 {
            Err("Address out of memory")
        } else {
            self.data[address as usize] = value;
            Ok(())
        }
    }
}

impl fmt::Display for dyn Memory {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "|---|-----------------------|\n").ok();
        write!(fmt, "|   | 0| 1| 2| 3| 4| 5| 6| 7|\n").ok();
        write!(fmt, "|---|-----------------------|").ok();
        for i  in 0..MEMORY_SIZE {
            if i % 8 == 0 {
                write!(fmt, "\n|{:3}|", i).ok();
            }
            match self.read(i) {
                Some(v) => write!(fmt, "{:2X}|", v),
                None =>  write!(fmt, "XX|"),
            }.ok();
        }
        Ok(())
    }
}

impl fmt::Display for SimpleMemory {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        (self as &dyn Memory).fmt(fmt)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_setting() {
        let mut rng = rand::thread_rng();
        for i in 0..10 { 
            let mut values = [0;MEMORY_SIZE as usize];
            values.try_fill(&mut rng).unwrap();
            let mut memory = SimpleMemory::new();
            
            for (value, address) in values.iter().zip(0..) {
                println!("{:?}", memory.write(address, *value));
                println!("Wrote {} to  {}", value, address);
                println!("data {}",memory);
            }
            println!("MEME\n{}",memory);
            for (value, address) in values.iter().zip(0..) {
                assert_eq!(Some(*value),memory.read(address), 
                        "Write/Read num {} from ADDR: {} failed\n\n{}",
                        i,
                        address,
                        memory
                        );
            }
        }
    }

}
