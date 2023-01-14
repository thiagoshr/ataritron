use core::fmt::Display;
use std::fmt;

mod memory {
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
				size: size
			})
		}
	}
}

