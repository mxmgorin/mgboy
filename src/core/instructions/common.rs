use crate::core::cpu::Cpu;
use crate::core::instructions::ccf::CcfInstruction;
use crate::core::instructions::table::INSTRUCTIONS_BY_OPCODES;
use crate::core::instructions::cpl::CplInstruction;
use crate::core::instructions::daa::DaaInstruction;
use crate::core::instructions::dec::DecInstruction;
use crate::core::instructions::di::DiInstruction;
use crate::core::instructions::halt::HaltInstruction;
use crate::core::instructions::inc::IncInstruction;
use crate::core::instructions::jp::JpInstruction;
use crate::core::instructions::jr::JrInstruction;
use crate::core::instructions::ld::LdInstruction;
use crate::core::instructions::nop::NopInstruction;
use crate::core::instructions::xor::XorInstruction;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Nop(NopInstruction),
    Inc(IncInstruction),
    Dec(DecInstruction),
    Ld(LdInstruction),
    Jr(JrInstruction),
    Daa(DaaInstruction),
    Cpl(CplInstruction),
    Ccf(CcfInstruction),
    Halt(HaltInstruction),
    Xor(XorInstruction),
    Di(DiInstruction),
    Jp(JpInstruction),
}

impl Instruction {
    pub fn get_by_opcode(opcode: u8) -> Option<&'static Instruction> {
        INSTRUCTIONS_BY_OPCODES.get(opcode as usize)
    }

    pub fn check_cond(cpu: &Cpu, cond: Option<ConditionType>) -> bool {
        let Some(cond) = cond else {
            return true;
        };

        match cond {
            ConditionType::C => cpu.get_flag_c(),
            ConditionType::NC => !cpu.get_flag_c(),
            ConditionType::Z => cpu.get_flag_z(),
            ConditionType::NZ => !cpu.get_flag_z(),
        }
    }

    pub fn goto_addr(cpu: &mut Cpu, cond: Option<ConditionType>, addr: u16, push_pc: bool) {
        if Instruction::check_cond(cpu, cond) {
            if push_pc {
                //emu_cycles(2);
                //stack_push16(cpu.registers.pc);  todo
            }

            cpu.registers.pc = addr;
            //emu_cycles(1);
        }
    }
}

impl ExecutableInstruction for Instruction {
    fn execute(&self, cpu: &mut Cpu) {
        match self {
            Instruction::Nop(inst) => inst.execute(cpu),
            Instruction::Inc(inst) => inst.execute(cpu),
            Instruction::Dec(inst) => inst.execute(cpu),
            Instruction::Ld(inst) => inst.execute(cpu),
            Instruction::Jr(inst) => inst.execute(cpu),
            Instruction::Daa(inst) => inst.execute(cpu),
            Instruction::Cpl(inst) => inst.execute(cpu),
            Instruction::Ccf(inst) => inst.execute(cpu),
            Instruction::Halt(inst) => inst.execute(cpu),
            Instruction::Xor(inst) => inst.execute(cpu),
            Instruction::Di(inst) => inst.execute(cpu),
            Instruction::Jp(inst) => inst.execute(cpu),
        }
    }

    fn get_address_mode(&self) -> AddressMode {
        match self {
            Instruction::Nop(inst) => inst.get_address_mode(),
            Instruction::Inc(inst) => inst.get_address_mode(),
            Instruction::Dec(inst) => inst.get_address_mode(),
            Instruction::Ld(inst) => inst.get_address_mode(),
            Instruction::Jr(inst) => inst.get_address_mode(),
            Instruction::Daa(inst) => inst.get_address_mode(),
            Instruction::Cpl(inst) => inst.get_address_mode(),
            Instruction::Ccf(inst) => inst.get_address_mode(),
            Instruction::Halt(inst) => inst.get_address_mode(),
            Instruction::Xor(inst) => inst.get_address_mode(),
            Instruction::Di(inst) => inst.get_address_mode(),
            Instruction::Jp(inst) => inst.get_address_mode(),
        }
    }
}

pub trait ExecutableInstruction {
    fn execute(&self, cpu: &mut Cpu);
    fn get_address_mode(&self) -> AddressMode;
}

/// Represents the various CPU registers in a Game Boy CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterType {
    /// Accumulator register, used for arithmetic and logic operations.
    A,
    /// Flags register, holds condition flags (Z, N, H, C).
    F,
    /// General-purpose register B.
    B,
    /// General-purpose register C.
    C,
    /// General-purpose register D.
    D,
    /// General-purpose register E.
    E,
    /// High byte of the HL register pair.
    H,
    /// Low byte of the HL register pair.
    L,
    /// Register pair combining A and F (used for specific operations).
    AF,
    /// Register pair combining B and C (used for addressing or data storage).
    BC,
    /// Register pair combining D and E (used for addressing or data storage).
    DE,
    /// Register pair combining H and L (often used as a memory address pointer).
    HL,
    /// Stack pointer, points to the top of the stack.
    SP,
    /// Program counter, points to the next instruction to be executed.
    PC,
}

impl RegisterType {
    pub fn is_16bit(&self) -> bool {
        match self {
            RegisterType::A
            | RegisterType::F
            | RegisterType::B
            | RegisterType::C
            | RegisterType::D
            | RegisterType::E
            | RegisterType::H
            | RegisterType::L => false,
            RegisterType::AF
            | RegisterType::BC
            | RegisterType::DE
            | RegisterType::HL
            | RegisterType::SP
            | RegisterType::PC => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionType {
    /// No Operation
    NOP,
    /// Load (LD) instruction
    LD,
    /// Increment (INC) instruction
    INC,
    /// Decrement (DEC) instruction
    DEC,
    /// Rotate Left Circular (RLCA) instruction
    RLCA,
    /// Add (ADD) instruction
    ADD,
    /// Rotate Right Circular (RRCA) instruction
    RRCA,
    /// Stop execution
    STOP,
    /// Rotate Left (RLA) instruction
    RLA,
    /// Jump Relative (JR) instruction
    JR,
    /// Rotate Right (RRA) instruction
    RRA,
    /// Decimal Adjust Accumulator (DAA) instruction
    DAA,
    /// Complement (CPL) instruction
    CPL,
    /// Set Carry Flag (SCF) instruction
    SCF,
    /// Complement Carry Flag (CCF) instruction
    CCF,
    /// Halt execution
    HALT,
    /// Add with Carry (ADC) instruction
    ADC,
    /// Subtract (SUB) instruction
    SUB,
    /// Subtract with Carry (SBC) instruction
    SBC,
    /// Logical AND (AND) instruction
    AND,
    /// Logical XOR (XOR) instruction
    XOR,
    /// Logical OR (OR) instruction
    OR,
    /// Compare (CP) instruction
    CP,
    /// Pop value from stack (POP) instruction
    POP,
    /// Jump (JP) instruction
    JP,
    /// Push value to stack (PUSH) instruction
    PUSH,
    /// Return from function (RET) instruction
    RET,
    /// CB prefix instruction (used for extended instructions)
    CB,
    /// Call function (CALL) instruction
    CALL,
    /// Return from interrupt (RETI) instruction
    RETI,
    /// Load high byte (LDH) instruction
    LDH,
    /// Jump to address in HL register (JPHL) instruction
    JPHL,
    /// Disable interrupts (DI) instruction
    DI,
    /// Enable interrupts (EI) instruction
    EI,
    /// Restart (RST) instruction
    RST,
    /// Error instruction
    ERR,
    /// Rotate Left Circular (RLC) instruction
    RLC,
    /// Rotate Right Circular (RRC) instruction
    RRC,
    /// Rotate Left (RL) instruction
    RL,
    /// Rotate Right (RR) instruction
    RR,
    /// Shift Left Arithmetic (SLA) instruction
    SLA,
    /// Shift Right Arithmetic (SRA) instruction
    SRA,
    /// Swap nibbles (SWAP) instruction
    SWAP,
    /// Shift Right Logical (SRL) instruction
    SRL,
    /// Test bit in register (BIT) instruction
    BIT,
    /// Reset bit in register (RES) instruction
    RES,
    /// Set bit in register (SET) instruction
    SET,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionType {
    /// Non-zero
    NZ,
    /// Zero
    Z,
    /// Non-carry
    NC,
    /// Carry
    C,
}

/// Represents the different address modes in the CPU's instruction set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum AddressMode {
    /// Immediate Addressing: The operand is directly specified in the instruction.
    IMP,
    /// Register: The operand is a register.
    R(RegisterType),
    /// Register with 16-bit immediate address: The operand is a 16-bit immediate value,
    /// and the instruction works with a register.
    R_D16(RegisterType),
    /// Register to Register: The operand is another register, and the instruction operates
    /// between two registers.
    R_R(RegisterType, RegisterType),
    /// Memory to Register: The operand is a memory location, and the instruction operates
    /// between memory and a register.
    MR_R(RegisterType, RegisterType),
    /// Register with 8-bit immediate value: The operand is an 8-bit immediate value,
    /// and the instruction operates with a register.
    R_D8(RegisterType),
    /// Register with Memory to Register: The instruction reads a value from memory and stores
    /// it into a register.
    R_MR(RegisterType),
    /// Register and HL increment: The instruction uses the `HL` register pair, increments it,
    /// and accesses memory using the updated value of `HL`.
    R_HLI(RegisterType, RegisterType),
    /// Register and HL decrement: The instruction uses the `HL` register pair, decrements it,
    /// and accesses memory using the updated value of `HL`.
    R_HLD(RegisterType, RegisterType),
    /// HL increment and Register: The instruction stores a value from a register to memory and
    /// increments the `HL` register pair.
    HLI_R(RegisterType, RegisterType),
    /// HL decrement and Register: The instruction stores a value from a register to memory and
    /// decrements the `HL` register pair.
    HLD_R(RegisterType, RegisterType),
    /// Register and 8-bit immediate address: The instruction uses a 8-bit immediate address and
    /// a register for memory access.
    R_A8(RegisterType),
    /// 8-bit address and Register: The instruction uses a memory address and a register to store
    /// a value from the register to memory.
    A8_R(RegisterType),
    /// HL and Special Register Pair: This mode uses the `HL` register and other special register pairs
    /// for specific operations.
    HL_SPR(RegisterType, RegisterType),
    /// 16-bit immediate data: The instruction involves a 16-bit immediate operand.
    D16,
    /// 8-bit immediate data: The instruction involves an 8-bit immediate operand.
    D8,
    /// 16-bit immediate data to Register: The instruction loads a 16-bit immediate operand to a register.
    D16_R(RegisterType),
    /// Memory Read and 8-bit immediate address: The instruction reads from memory using an 8-bit immediate address.
    MR_D8(RegisterType),
    /// Memory Read: The instruction performs a read operation from memory.
    MR(RegisterType),
    /// 16-bit Address and Register: The instruction works with a 16-bit memory address and a register.
    A16_R(RegisterType),
    /// Register and 16-bit Address: The instruction stores a value from a register to a 16-bit memory address.
    R_A16(RegisterType),
}
