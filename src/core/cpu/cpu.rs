use crate::core::bus::Bus;
use crate::core::cpu::instructions::common::{AddressMode, ExecutableInstruction, Instruction};
use crate::core::cpu::stack::Stack;
use crate::core::cpu::Registers;
use crate::core::debugger::Debugger;
use crate::core::InterruptType;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub bus: Bus,
    pub registers: Registers,
    pub enabling_ime: bool,
    pub ticks: i32,
    pub current_opcode: u8,
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        Self {
            bus,
            registers: Registers::new(),
            enabling_ime: false,
            ticks: 0,
            current_opcode: 0,
        }
    }

    pub fn step(&mut self, debugger: &mut Option<Debugger>) -> Result<(), String> {
        if self.bus.io.interrupts.cpu_halted {
            self.update_cycles(1);

            if self.bus.io.interrupts.int_flags != 0 {
                self.bus.io.interrupts.cpu_halted = false;
            }

            return Ok(());
        }

        let pc = self.registers.pc;
        self.current_opcode = self.fetch_opcode();

        let Some(instruction) = Instruction::get_by_opcode(self.current_opcode) else {
            return Err(format!(
                "Unknown instruction OPCODE: {:X}",
                self.current_opcode,
            ));
        };

        let fetched_data = AddressMode::fetch_data(self, instruction.get_address_mode());

        #[cfg(debug_assertions)]
        if let Some(debugger) = debugger.as_mut() {
            debugger.print_cpu_info(self, pc, instruction, self.current_opcode, &fetched_data);
            debugger.update(self);
            debugger.print();
        }

        instruction.execute(self, fetched_data);

        if self.bus.io.interrupts.int_master_enabled {
            if let Some(it) = self.bus.io.interrupts.get_interrupt() {
                self.handle_interrupt(it);
            }

            self.enabling_ime = false;
        }

        if self.enabling_ime {
            self.bus.io.interrupts.int_master_enabled = true;
        }

        Ok(())
    }

    pub fn update_cycles(&mut self, cpu_cycles: i32) {
        for _ in 0..cpu_cycles {
            for _ in 0..4 {
                self.ticks += 1;

                if self.bus.io.timer.tick() {
                    self.bus
                        .io
                        .interrupts
                        .request_interrupt(InterruptType::Timer);
                }

                //ppu_tick(); todo
            }

            //dma_tick(); todo
        }
    }

    fn handle_interrupt(&mut self, it: InterruptType) {
        self.bus.io.interrupts.handle_interrupt(it);

        let address = it.get_address();
        Stack::push16(&mut self.registers, &mut self.bus, address);
        self.registers.pc = address;
    }

    fn fetch_opcode(&mut self) -> u8 {
        let opcode = self.bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        opcode
    }
}
