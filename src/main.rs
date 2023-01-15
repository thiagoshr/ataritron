use memory::Memory;
use cpu::Cpu;

mod memory;
mod cpu;

fn main() {
    let mem = Memory::new(64*1024).unwrap();
    let _cpu = Cpu::new(mem);
}
