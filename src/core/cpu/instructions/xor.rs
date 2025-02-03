use crate::core::cpu::instructions::common::{AddressMode, ExecutableInstruction};
use crate::core::cpu::Cpu;
use crate::cpu::instructions::common::FetchedData;

#[derive(Debug, Clone, Copy)]
pub struct XorInstruction {
    pub address_mode: AddressMode,
}

impl ExecutableInstruction for XorInstruction {
    fn execute(&self, cpu: &mut Cpu, fetched_data: FetchedData) {
        cpu.registers.a ^= (fetched_data.value & 0xFF) as u8;
        cpu.registers.set_flags(
            ((cpu.registers.a == 0) as i8).into(),
            0.into(),
            0.into(),
            0.into(),
        );
    }

    fn get_address_mode(&self) -> AddressMode {
        self.address_mode
    }
}
