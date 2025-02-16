use crate::mooneye::util::{assert_result, run_test_rom, MooneyeRomCategory};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(15);

#[test]
fn test_oam_dma_basic() {
    let name = "basic";
    let category = MooneyeRomCategory::OamDma.into();
    let result = run_test_rom(name, category, TIMEOUT);

    assert_result(name, category, result);
}

#[test]
fn test_oam_dma_reg_read() {
    let name = "reg_read";
    let category = MooneyeRomCategory::OamDma.into();
    let result = run_test_rom(name, category, TIMEOUT);

    assert_result(name, category, result);
}

#[test]
fn test_oam_dma_sources_gs() {
    let name = "sources-GS";
    let category = MooneyeRomCategory::OamDma.into();
    let result = run_test_rom(name, category, TIMEOUT);

    assert_result(name, category, result);
}
