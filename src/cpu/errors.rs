use std::{error::Error, fmt};

use crate::memory::OutOfRangeError;


#[derive(Debug)]
pub enum CpuError {
	InvalidAddressModeDerefenced,
	MemoryBoundsError(OutOfRangeError)
}

impl Error for CpuError {}

impl fmt::Display for CpuError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidAddressModeDerefenced => write!(f, "Invalid address mode dereferenced"),
			Self::MemoryBoundsError(e) => write!(f, "{}", e.to_string())
		}
	}
}
