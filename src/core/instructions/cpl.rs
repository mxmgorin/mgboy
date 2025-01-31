use crate::core::cpu::Cpu;
use crate::core::instructions::common::{AddressMode, ExecutableInstruction};

#[derive(Debug, Clone, Copy)]
pub struct CplInstruction;

impl ExecutableInstruction for CplInstruction {
    fn execute(&self, _cpu: &mut Cpu) {
        eprintln!("CplInstruction not impl")
    }

    fn get_address_mode(&self) -> AddressMode {
        AddressMode::IMP
    }
}
