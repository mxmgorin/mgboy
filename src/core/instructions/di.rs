use crate::core::cpu::Cpu;
use crate::core::instructions::common::{AddressMode, ExecutableInstruction};

#[derive(Debug, Clone, Copy)]
pub struct DiInstruction;

impl ExecutableInstruction for DiInstruction {
    fn execute(&self, _cpu: &mut Cpu) {
        eprintln!("DiInstruction not impl")
    }

    fn get_address_mode(&self) -> AddressMode {
        AddressMode::IMP
    }
}
