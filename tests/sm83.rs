use crate::common::{run_sb_test_cases, run_test_case, Sm83TestCase};
use rusty_gb_emu::cpu::instructions::common::opcodes::INSTRUCTIONS_BY_OPCODES;
use rusty_gb_emu::cpu::instructions::common::Instruction;

mod common;

#[test]
fn test_sm83_custom() {
    let test_cases = Sm83TestCase::load_file("cb 28.json");

    for test_case in test_cases.iter() {
        run_test_case(test_case, true);
    }
}

#[test]
fn test_sm83_sb() {
    run_sb_test_cases(false)
}

#[test]
fn test_sm83() {
    let print_result = false;
    
    for (opcode, instruction) in INSTRUCTIONS_BY_OPCODES.iter().enumerate() {
        if let Instruction::Unknown(_) = instruction {
            continue;
        }
        
        if opcode == 0xCB {
            run_sb_test_cases(print_result);
        }

        let test_cases = Sm83TestCase::load_opcode(opcode as u16);

        for test_case in test_cases.iter() {
            run_test_case(test_case, print_result);
        }

        if print_result {
            println!("{:02X} passed {} test cases", opcode, test_cases.len());
        }
    }
}

#[test]
fn test_sm83_case_static() {
    let json_data = r#"
    {
        "name": "41 0000",
        "initial": {
            "pc": 9845,
            "sp": 50643,
            "a": 185,
            "b": 151,
            "c": 101,
            "d": 187,
            "e": 72,
            "f": 160,
            "h": 117,
            "l": 249,
            "ime": 1,
            "ie": 0,
            "ram": [[9845, 65]]
        },
        "final": {
            "a": 185,
            "b": 101,
            "c": 101,
            "d": 187,
            "e": 72,
            "f": 160,
            "h": 117,
            "l": 249,
            "pc": 9846,
            "sp": 50643,
            "ime": 1,
            "ram": [[9845, 65]]
        },
        "cycles": [[9845, 65, "r-m"]]
    }"#;

    let test_case = Sm83TestCase::from_json(json_data);

    run_test_case(&test_case, true);
}
