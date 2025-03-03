use crate::cpu::instructions::{AddressMode, ExecutableInstruction, RegisterType};
use crate::cpu::instructions::{DataDestination, DataSource, FetchedData};
use crate::cpu::{Cpu, CpuCallback};

#[derive(Debug, Clone, Copy)]
pub struct LdInstruction {
    pub address_mode: AddressMode,
}

impl ExecutableInstruction for LdInstruction {
    fn execute(&self, cpu: &mut Cpu, callback: &mut impl CpuCallback, fetched_data: FetchedData) {
        match fetched_data.dest {
            DataDestination::Register(r) => {
                if self.address_mode.is_hl_spi8() {
                    let h_flag = (cpu.registers.sp & 0xF) + (fetched_data.value & 0xF) >= 0x10;
                    let c_flag = (cpu.registers.sp & 0xFF) + (fetched_data.value & 0xFF) >= 0x100;

                    cpu.registers
                        .flags
                        .set(false.into(), false.into(), Some(h_flag), Some(c_flag));
                    let offset_e = fetched_data.value as i8; // truncate to 8 bits (+8e)

                    cpu.registers.set_register(
                        RegisterType::HL,
                        cpu.registers.sp.wrapping_add(offset_e as u16),
                    );

                    callback.m_cycles(1, &mut cpu.bus);
                } else {
                    if let DataSource::Register(src_r) = fetched_data.source {
                        if r.is_16bit() && src_r.is_16bit() {
                            callback.m_cycles(1, &mut cpu.bus);
                        }
                    }

                    cpu.registers.set_register(r, fetched_data.value);
                }
            }
            DataDestination::Memory(addr) => match fetched_data.source {
                DataSource::Memory(_) => unreachable!(),
                DataSource::Register(r) | DataSource::MemoryRegister(r, _) => {
                    if r.is_16bit() {
                        cpu.write_to_memory(
                            addr + 1,
                            ((fetched_data.value >> 8) & 0xFF) as u8,
                            callback,
                        );
                        cpu.write_to_memory(addr, (fetched_data.value & 0xFF) as u8, callback);
                    } else {
                        cpu.write_to_memory(addr, fetched_data.value as u8, callback);
                    }
                }
                DataSource::Immediate => {
                    cpu.write_to_memory(addr, fetched_data.value as u8, callback);
                }
            },
        }
    }

    fn get_address_mode(&self) -> AddressMode {
        self.address_mode
    }
}
