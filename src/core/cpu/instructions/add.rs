use crate::core::cpu::instructions::common::{AddressMode, ExecutableInstruction};
use crate::core::cpu::Cpu;
use crate::cpu::instructions::common::{FetchedData, RegisterType};

#[derive(Debug, Clone, Copy)]
pub struct AddInstruction {
    pub address_mode: AddressMode,
}

impl ExecutableInstruction for AddInstruction {
    fn execute(&self, cpu: &mut Cpu, fetched_data: FetchedData) {
        match self.address_mode {
            AddressMode::IMP
            | AddressMode::D16
            | AddressMode::D8
            | AddressMode::R_HLI(_)
            | AddressMode::R_HLD(_)
            | AddressMode::HLI_R(_)
            | AddressMode::HLD_R(_) => unreachable!("not used"),
            AddressMode::HL_SPe8 => execute_add(cpu, fetched_data, RegisterType::SP),
            AddressMode::R_R(r1, _r2) | AddressMode::MR_R(r1, _r2) | AddressMode::R_MR(r1, _r2) => {
                execute_add(cpu, fetched_data, r1)
            }
            AddressMode::R_D8(r1)
            | AddressMode::R(r1)
            | AddressMode::R_D16(r1)
            | AddressMode::R_A8(r1)
            | AddressMode::A8_R(r1)
            | AddressMode::D16_R(r1)
            | AddressMode::MR_D8(r1)
            | AddressMode::MR(r1)
            | AddressMode::A16_R(r1)
            | AddressMode::R_A16(r1) => execute_add(cpu, fetched_data, r1),
        }
    }

    fn get_address_mode(&self) -> AddressMode {
        self.address_mode
    }
}

// todo: test or rewrite casting, do they are correct?
fn execute_add(cpu: &mut Cpu, fetched_data: FetchedData, r1: RegisterType) {
    let mut val_u32: u32 = cpu.registers.read_register(r1) as u32 + fetched_data.value as u32;
    let is_16bit = r1.is_16bit();

    if is_16bit {
        cpu.update_cycles(1);
    }

    if r1 == RegisterType::SP {
        let fetched_val_i8 = fetched_data.value as i8;
        let val = cpu.registers.read_register(r1).wrapping_add(fetched_val_i8 as u16);
        val_u32 = val as u32;
    }

    let mut z = if (val_u32 & 0xFF) == 0 {
        Some(true)
    } else {
        Some(false)
    };
    let mut h = (cpu.registers.read_register(r1) & 0xF) + (fetched_data.value & 0xF) >= 0x10;
    let mut c = (cpu.registers.read_register(r1) & 0xFF) + (fetched_data.value & 0xFF) >= 0x100;

    if is_16bit {
        z = None;
        h = (cpu.registers.read_register(r1) & 0xFFF) + (fetched_data.value & 0xFFF) >= 0x1000;
        let n = (cpu.registers.read_register(r1) as u32) + (fetched_data.value as u32);
        c = n >= 0x10000;
    }

    if r1 == RegisterType::SP {
        z = Some(false);
        h = (cpu.registers.read_register(r1) & 0xF) + (fetched_data.value & 0xF) >= 0x10;
        c = (cpu.registers.read_register(r1) & 0xFF) + (fetched_data.value & 0xFF) >= 0x100;
    }

    let val = val_u32 & 0xFFFF;
    cpu.registers.set_register(r1, val as u16);
    cpu.registers
        .flags
        .set(z.into(), false.into(), h.into(), c.into());
}
