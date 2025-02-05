use std::fs;
use rusty_gb_emu::bus::Bus;
use rusty_gb_emu::cpu::{Cpu, Flags, Registers};
use rusty_gb_emu::debugger::{CpuLogType, Debugger};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use rusty_gb_emu::cpu::instructions::common::ExecutableInstruction;
use rusty_gb_emu::cpu::instructions::common::opcodes::INSTRUCTIONS_BY_OPCODES;

pub fn run_test_case(test_case: &Sm83TestCase, print_result: bool) {
    let title = format!("Test case '{}'", test_case.name);

    let mut cpu = setup_cpu(test_case);
    let mut debugger = Some(Debugger::new(CpuLogType::None, false));
    cpu.step(&mut debugger).unwrap();

    let result = test_case.validate_final_state(&cpu);

    if let Err(err) = result {
        let inst = INSTRUCTIONS_BY_OPCODES[cpu.current_opcode as usize];
        print_with_dashes(&format!("{} ({:?} {:?})", title, inst.get_type(), inst.get_address_mode()));
        print_with_dashes("Result: FAILED");
        eprintln!("Error: {err}");
        eprintln!("{} ", serde_json::to_string_pretty(test_case).unwrap());
        panic!("Test case failed {}", test_case.name);        
    }

    if print_result {
        println!("{title}: OK");
    }
}

pub fn run_sb_test_cases(print_result: bool) {
    for i in 0..=256 {
        let test_cases = Sm83TestCase::load_file(&format!("cb {:02X}.json", i).to_lowercase());

        for test_case in test_cases.iter() {
            run_test_case(test_case, print_result);
        }

        println!("{:02X} passed {} test cases", 0xCB, test_cases.len());
    }
}

fn print_with_dashes(content: &str) {
    const TOTAL_LEN: usize = 100;
    let content_length = content.len();
    let dashes = "-".repeat(TOTAL_LEN.saturating_sub(content_length));
    println!("{} {}", content, dashes);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sm83TestCase {
    pub name: String,
    #[serde(rename = "initial")]
    pub initial_state: CpuState,
    #[serde(rename = "final")]
    pub final_state: CpuState,
    pub cycles: Vec<Cycle>,
}

impl Sm83TestCase {
    pub fn load_opcode(opcode: u16) -> Vec<Sm83TestCase> {
        Sm83TestCase::load_file(&format!("{:02X}.json", opcode))
    }

    pub fn load_file(file_name: &str) -> Vec<Sm83TestCase> {
        let json_path = PathBuf::from("tests")
            .join("sm83_data")
            .join("v1")
            .join(file_name);
        let json_data = fs::read_to_string(&json_path)
            .unwrap_or_else(|e| panic!("Failed to read file at {:?}: {}", json_path, e));

        serde_json::from_str(&json_data).unwrap()
    }

    pub fn from_json(json: &str) -> Sm83TestCase {
        serde_json::from_str(json).unwrap()
    }

    pub fn _assert_final_state(&self, cpu: &Cpu) {
        assert_eq!(cpu.registers.a, self.final_state.a);
        assert_eq!(cpu.registers.b, self.final_state.b);
        assert_eq!(cpu.registers.c, self.final_state.c);
        assert_eq!(cpu.registers.d, self.final_state.d);
        assert_eq!(cpu.registers.e, self.final_state.e);
        assert_eq!(cpu.registers.flags.byte, self.final_state.f);
        assert_eq!(cpu.registers.h, self.final_state.h);
        assert_eq!(cpu.registers.l, self.final_state.l);
        assert_eq!(cpu.registers.sp, self.final_state.sp);
        assert_eq!(cpu.registers.pc, self.final_state.pc);
        assert_eq!(cpu.enabling_ime, self.final_state.ime != 0);

        // todo: re-search
        //assert_eq!(cpu.bus.io.interrupts.ie_register, self.final_state.ie);
        //assert_eq!(cpu.cycles, self.final_state.cycles);

        // Assert RAM contents
        for ram in self.final_state.ram.iter() {
            assert_eq!(cpu.bus.read(ram.0), ram.1);
        }
    }

    pub fn validate_final_state(&self, cpu: &Cpu) -> Result<(), String> {
        if cpu.registers.a != self.final_state.a {
            return Err(format!("Invalid A: actual={}, expected={}", cpu.registers.a, self.final_state.a));
        }
        if cpu.registers.b != self.final_state.b {
            return Err(format!("Invalid B: actual={}, expected={}", cpu.registers.b, self.final_state.b));
        }
        if cpu.registers.c != self.final_state.c {
            return Err(format!("Invalid C: actual={}, expected={}", cpu.registers.c, self.final_state.c));
        }
        if cpu.registers.d != self.final_state.d {
            return Err(format!("Invalid D: actual={}, expected={}", cpu.registers.d, self.final_state.d));
        }
        if cpu.registers.e != self.final_state.e {
            return Err(format!("Invalid E: actual={}, expected={}", cpu.registers.e, self.final_state.e));
        }
        if cpu.registers.flags.byte != self.final_state.f {
            return Err(format!("Invalid F: actual={}, expected={}", cpu.registers.flags.byte, self.final_state.f));
        }
        if cpu.registers.h != self.final_state.h {
            return Err(format!("Invalid H: actual={}, expected={}", cpu.registers.h, self.final_state.h));
        }
        if cpu.registers.l != self.final_state.l {
            return Err(format!("Invalid L: actual={}, expected={}", cpu.registers.l, self.final_state.l));
        }
        if cpu.registers.sp != self.final_state.sp {
            return Err(format!("Invalid SP: actual={}, expected={}", cpu.registers.sp, self.final_state.sp));
        }
        if cpu.registers.pc != self.final_state.pc {
            return Err(format!("Invalid PC: actual={}, expected={}", cpu.registers.pc, self.final_state.pc));
        }
        if cpu.bus.io.interrupts.ime != (self.final_state.ime != 0) {
            return Err(format!("Invalid IME: actual={}, expected={}", cpu.bus.io.interrupts.ime, self.final_state.ime));
        }

        // todo: re-search
        //assert_eq!(cpu.bus.io.interrupts.ie_register, self.final_state.ie {}
        //assert_eq!(cpu.cycles, self.final_state.cycles {}

        // Assert RAM contents
        for ram in self.final_state.ram.iter() {
            if cpu.bus.read(ram.0) != ram.1 {
                return Err(format!("Invalid RAM: actual={}, expected={}", cpu.bus.read(ram.0), ram.1));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuState {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub ime: u8,
    #[serde(default)]
    pub ie: u8,
    pub ram: Vec<RamState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RamState(pub u16, pub u8);

#[derive(Debug, Serialize, Deserialize)]
pub struct Cycle(pub u16, pub u8, pub String);

pub fn setup_cpu(test_case: &Sm83TestCase) -> Cpu {
    let mut cpu = Cpu::new(setup_bus(test_case));
    set_cpu_state(&mut cpu, test_case);

    cpu
}

pub fn setup_bus(test_case: &Sm83TestCase) -> Bus {
    let mut bus = Bus::test();
    for ram in test_case.initial_state.ram.iter() {
        bus.write(ram.0, ram.1);
    }

    bus
}

pub fn set_cpu_state(cpu: &mut Cpu, test_case: &Sm83TestCase) {
    cpu.registers = Registers {
        a: test_case.initial_state.a,
        flags: Flags {
            byte: test_case.initial_state.f,
        },
        b: test_case.initial_state.b,
        c: test_case.initial_state.c,
        d: test_case.initial_state.d,
        e: test_case.initial_state.e,
        h: test_case.initial_state.h,
        l: test_case.initial_state.l,
        sp: test_case.initial_state.sp,
        pc: test_case.initial_state.pc,
    };
    cpu.bus.io.interrupts.ie_register = test_case.initial_state.ie;
    cpu.bus.io.interrupts.ime = test_case.initial_state.ime != 0
}
