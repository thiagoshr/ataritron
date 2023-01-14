use core::fmt;

#[derive(Debug)]
pub struct OutOfRangeError {
    value : u32,
    min : u32,
    max : u32,
}

impl fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value {} is outside of allowed range [{}, {}]", self.value, self.min, self.max)
    }
}

pub struct Memory {
    data : Vec<u8>,
    size : u32
}

impl Memory {
    fn address_in_bounds(addr : u16, size : u32) -> bool {
        (addr as u32) < size
    }

    pub fn new(size : u32) -> Result<Memory, OutOfRangeError> {
        // size for the atary 2600 is 13.5 kB.
        // we have a 16-bit bus so max size is 0xffff+1
        if size < 13824 || size > 0xffff + 1 {
            return Err(OutOfRangeError {
                value: size,
                min: 13824,
                max: 0xffff+1
            })
        }

        Ok(Memory {
            data: vec![0x00; size as usize],
            size
        })
    }

    pub fn load(&self, addr : u16) -> Result<u8, OutOfRangeError> {
        if Memory::address_in_bounds(addr, self.size) {
            Ok(self.data[addr as usize])
        } else {
            Err(OutOfRangeError {
                value: addr as u32,
                min: 0x0,
                max: (self.size - 1) as u32
            })
        }
    }

    pub fn store(&mut self, addr : u16, byte : u8) -> Result<(), OutOfRangeError> {
        if Memory::address_in_bounds(addr, self.size) {
            self.data[addr as usize] = byte;
            return Ok(())
        }

        Err(OutOfRangeError {
            value: addr as u32,
            min: 0x0,
            max: (self.size - 1) as u32
        })
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_inits_to_zero_array() {
        let size = 32 * 1024; // a middle-of-the-road test case

        let expected_mem = vec![0x00 as u8; size];
        assert_eq!(size, expected_mem.len());

        let new_mem = Memory::new(size as u32).unwrap();
        assert_eq!(expected_mem, new_mem.data);
    }

    #[test]
    fn memory_new_errors_on_size_too_big() {
        assert!(Memory::new(65537).is_err());
    }

    #[test]
    fn memory_new_errors_on_size_too_small() {
        assert!(Memory::new(13823).is_err());
    }

    #[test]
    fn memory_loads_a_byte() {
        let mut mem = Memory::new(16 * 1024).unwrap();
        let addr : u16 = 0x1200; // I picked the address
        let my_byte = 0x88; // why not 0x88? mem inits to 0x00 and that is tested

        // cannot rely on store function for unit test
        mem.data[addr as usize] = my_byte; 
        assert_eq!(my_byte, mem.load(addr).unwrap());
    }

    #[test]
    fn memory_stores_a_byte() {
        let mut mem = Memory::new(16 * 1024).unwrap();
        let addr = 0x3000;
        let my_value = 50;

        mem.store(addr, my_value).unwrap();
        assert_eq!(my_value, mem.data[addr as usize]);
    }
}

