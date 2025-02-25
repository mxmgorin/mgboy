use crate::apu::wave_channel::WaveChannel;
use crate::apu::{APU_CLOCK_SPEED, NR52};
use crate::CPU_CLOCK_SPEED;

// The frame sequencer generates low frequency clocks for the modulation units. It is clocked by a 512 Hz timer.
// Step   Length Ctr  Vol Env     Sweep
// ---------------------------------------
// 0      Clock       -           -
// 1      -           -           -
// 2      Clock       -           Clock
// 3      -           -           -
// 4      Clock       -           -
// 5      -           -           -
// 6      Clock       -           Clock
// 7      -           Clock       -
// ---------------------------------------
// Rate   256 Hz      64 Hz       128 Hz
const CYCLES_DIV: u16 = (CPU_CLOCK_SPEED / APU_CLOCK_SPEED as u32) as u16;

#[derive(Debug, Clone, Default)]
pub struct FrameSequencer {
    step: u8,
}

impl FrameSequencer {
    // ticks every t-cycle
    pub fn tick(&mut self, sample_clock: u32, master_ctrl: &mut NR52, ch3: &mut WaveChannel) {
        if sample_clock % CYCLES_DIV as u32 == 0 {
            match self.step {
                0 => ch3.tick_length(master_ctrl), // tick_length
                1 => {}
                2 => ch3.tick_length(master_ctrl), // tick length, sweep
                3 => {}
                4 => ch3.tick_length(master_ctrl), // tick_length
                5 => {}
                6 => ch3.tick_length(master_ctrl), // tick length, sweep
                7 => {}                            // tick envelope
                _ => unreachable!(),
            }

            self.step = (self.step + 1) & 7;
        }
    }
}
