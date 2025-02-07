use crate::core::cpu::instructions::{AddressMode, ExecutableInstruction};
use crate::core::cpu::Cpu;
use crate::cpu::instructions::FetchedData;

#[derive(Debug, Clone, Copy)]
pub struct DecInstruction {
    pub address_mode: AddressMode,
}

impl ExecutableInstruction for DecInstruction {
    fn execute(&self, cpu: &mut Cpu, fetched_data: FetchedData) {
        let mut value = fetched_data.value.wrapping_sub(1);

        match self.address_mode {
            AddressMode::IMP
            | AddressMode::D16
            | AddressMode::HL_SPe8
            | AddressMode::D8
            | AddressMode::R_D16(_)
            | AddressMode::R_D8(_)
            | AddressMode::R_HLI(_)
            | AddressMode::R_HLD(_)
            | AddressMode::HLI_R(_)
            | AddressMode::HLD_R(_)
            | AddressMode::R_A8(_)
            | AddressMode::A8_R(_)
            | AddressMode::MR_D8(_)
            | AddressMode::A16_R(_)
            | AddressMode::R_A16(_)
            | AddressMode::R_R(_, _)
            | AddressMode::MR_R(_, _)
            | AddressMode::R_MR(_, _) => panic!("not used"),
            AddressMode::MR(_r1) => {
                cpu.write_to_memory(
                    fetched_data.dest_addr.expect("must exist for MR"),
                    value as u8,
                );
                set_flags(cpu, value);
            }
            AddressMode::R(r1) => {
                if r1.is_16bit() {
                    cpu.update_cycles(1);
                }

                cpu.registers.set_register(r1, value);
                value = cpu.registers.read_register(r1);
            }
        }

        set_flags(cpu, value);
    }

    fn get_address_mode(&self) -> AddressMode {
        self.address_mode
    }
}

pub fn set_flags(cpu: &mut Cpu, val: u16) {
    if (cpu.current_opcode & 0x0B) == 0x0B {
        return;
    }

    cpu.registers.flags.set(
        (val == 0).into(),
        true.into(),
        ((val & 0x0F) == 0x0F).into(),
        None,
    );
}
