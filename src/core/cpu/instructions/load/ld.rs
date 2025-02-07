use crate::core::cpu::instructions::{AddressMode, ExecutableInstruction, RegisterType};
use crate::core::cpu::Cpu;
use crate::cpu::instructions::FetchedData;

#[derive(Debug, Clone, Copy)]
pub struct LdInstruction {
    pub address_mode: AddressMode,
}

impl ExecutableInstruction for LdInstruction {
    fn execute(&self, cpu: &mut Cpu, fetched_data: FetchedData) {
        match self.address_mode {
            AddressMode::D8
            | AddressMode::D16
            | AddressMode::IMP
            | AddressMode::MR(_)
            | AddressMode::R(_) => {
                unreachable!("not used for LD")
            }
            AddressMode::R_HLI(r1) | AddressMode::R_HLD(r1) => {
                cpu.registers.set_register(r1, fetched_data.value)
            }

            AddressMode::HLI_R(r2) | AddressMode::HLD_R(r2) => {
                write_to_addr(
                    cpu,
                    r2,
                    fetched_data.dest_addr.expect("must be set for HLI, HLD"),
                    fetched_data.value,
                );
            }
            AddressMode::R_D8(r1)
            | AddressMode::R_A8(r1)
            | AddressMode::R_A16(r1)
            | AddressMode::R_D16(r1) => {
                cpu.registers.set_register(r1, fetched_data.value);
            }
            AddressMode::A8_R(r1) | AddressMode::A16_R(r1) | AddressMode::MR_D8(r1) => {
                write_to_addr(
                    cpu,
                    r1,
                    fetched_data.dest_addr.expect("must be set"),
                    fetched_data.value,
                );
            }
            AddressMode::R_R(r1, r2) | AddressMode::R_MR(r1, r2) | AddressMode::MR_R(r1, r2) => {
                if let Some(dest_addr) = fetched_data.dest_addr {
                    write_to_addr(cpu, r2, dest_addr, fetched_data.value);
                } else {
                    cpu.registers.set_register(r1, fetched_data.value);
                }
            }
            // LD HL,SP+e8
            // Add the signed value e8 to SP and copy the result in HL.
            AddressMode::HL_SPe8 => {
                let h_flag = (cpu.registers.sp & 0xF) + (fetched_data.value & 0xF) >= 0x10;
                let c_flag = (cpu.registers.sp & 0xFF) + (fetched_data.value & 0xFF) >= 0x100;

                cpu.registers
                    .flags
                    .set(false.into(), false.into(), Some(h_flag), Some(c_flag));
                let offset_e = fetched_data.value as i8; // truncate to 8 bits (+8e)
                cpu.registers
                    .set_register(RegisterType::HL, cpu.registers.sp.wrapping_add(offset_e as u16));

                cpu.update_cycles(1);
            }
        }
    }

    fn get_address_mode(&self) -> AddressMode {
        self.address_mode
    }
}

fn write_to_addr(cpu: &mut Cpu, r2: RegisterType, addr: u16, value: u16) {
    if r2.is_16bit() {
        cpu.update_cycles(1);
        cpu.bus.write16(addr, value);
    } else {
        cpu.bus.write(addr, value as u8);
    }

    cpu.update_cycles(1);
}
