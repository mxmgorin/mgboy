use crate::apu::channels::channel::ChannelType;
use crate::apu::channels::noise_channel::{NoiseChannel, CH4_END_ADDRESS, CH4_START_ADDRESS};
use crate::apu::channels::square_channel::{
    SquareChannel, CH1_END_ADDRESS, CH1_START_ADDRESS, CH2_END_ADDRESS, CH2_START_ADDRESS,
};
use crate::apu::channels::wave_channel::{
    WaveChannel, CH3_END_ADDRESS, CH3_START_ADDRESS, CH3_WAVE_RAM_END, CH3_WAVE_RAM_START,
};
use crate::apu::dac::apply_dac;
use crate::apu::mixer::Mixer;
use crate::{get_bit_flag, set_bit, CPU_CLOCK_SPEED};

pub const APU_CLOCK_SPEED: u16 = 512;
pub const SAMPLING_FREQUENCY: u16 = 44100;

pub const AUDIO_MASTER_CONTROL_ADDRESS: u16 = 0xFF26;
pub const SOUND_PLANNING_ADDRESS: u16 = 0xFF25;
pub const MASTER_VOLUME_ADDRESS: u16 = 0xFF24;
pub const AUDIO_BUFFER_SIZE: usize = 1024;

pub const FRAME_SEQUENCER_DIV: u16 = (CPU_CLOCK_SPEED / APU_CLOCK_SPEED as u32) as u16;

#[derive(Debug, Clone)]
pub struct Apu {
    // internal
    ch1: SquareChannel,
    ch2: SquareChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    nr52_master_ctrl: NR52,
    mixer: Mixer,

    // other data
    frame_sequencer_step: u8,
    ticks_count: u32,
    buffer: Box<[f32; AUDIO_BUFFER_SIZE]>,
    buffer_index: usize,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            ch1: SquareChannel::ch1(),
            ch2: SquareChannel::ch2(),
            ch3: WaveChannel::default(),
            ch4: NoiseChannel::default(),
            nr52_master_ctrl: NR52::default(),
            mixer: Default::default(),
            frame_sequencer_step: 0,
            ticks_count: 0,
            buffer: Box::new([0.0; AUDIO_BUFFER_SIZE]),
            buffer_index: 0,
        }
    }
}

impl Apu {
    pub fn tick(&mut self) {
        self.ticks_count = self.ticks_count.wrapping_add(1);
        self.sequence_frame();

        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();

        // down sample by nearest-neighbor
        let ticks_per_sample = CPU_CLOCK_SPEED / SAMPLING_FREQUENCY as u32;

        if self.ticks_count % ticks_per_sample == 0 {
            if self.is_buffer_full() {
                self.buffer_index = 0;
            }

            self.mixer.outputs[0] = apply_dac(self.nr52_master_ctrl, &self.ch1);
            self.mixer.outputs[1] = apply_dac(self.nr52_master_ctrl, &self.ch2);
            self.mixer.outputs[2] = apply_dac(self.nr52_master_ctrl, &self.ch3);
            let (output_left, output_right) = self.mixer.mix();

            self.buffer[self.buffer_index] = output_left;
            self.buffer[self.buffer_index + 1] = output_right;
            self.buffer_index += 2;
        }
    }

    pub fn take_buffer(&mut self) -> &[f32] {
        let buffer = &self.buffer[0..self.buffer_index];
        self.buffer_index = 0;

        buffer
    }

    pub fn is_buffer_empty(&self) -> bool {
        self.buffer_index == 0
    }

    pub fn is_buffer_full(&self) -> bool {
        self.buffer_index >= AUDIO_BUFFER_SIZE
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if (CH3_WAVE_RAM_START..=CH3_WAVE_RAM_END).contains(&address) {
            self.ch3.wave_ram.write(address, value);
            return;
        }

        if address == AUDIO_MASTER_CONTROL_ADDRESS {
            let prev_enable = self.nr52_master_ctrl.is_audio_on();
            self.nr52_master_ctrl.write(value);

            if !prev_enable && self.nr52_master_ctrl.is_audio_on() {
                // turning on
                self.ch3.wave_ram.clear_sample_buffer();
            }

            return;
        }

        if !self.nr52_master_ctrl.is_audio_on() {
            return;
        }

        // todo: the length timers (in NRx1) on monochrome models also writable event when turned off

        match address {
            CH1_START_ADDRESS..=CH1_END_ADDRESS => {
                self.ch1.write(address, value, &mut self.nr52_master_ctrl)
            }
            CH2_START_ADDRESS..=CH2_END_ADDRESS => {
                self.ch2.write(address, value, &mut self.nr52_master_ctrl)
            }
            CH3_START_ADDRESS..=CH3_END_ADDRESS => {
                self.ch3.write(address, value, &mut self.nr52_master_ctrl)
            }
            CH4_START_ADDRESS..=CH4_END_ADDRESS => {}
            AUDIO_MASTER_CONTROL_ADDRESS => self.nr52_master_ctrl.write(value),
            SOUND_PLANNING_ADDRESS => self.mixer.nr51_sound_panning.byte = value,
            MASTER_VOLUME_ADDRESS => self.mixer.nr50_master_volume.byte = value,
            CH3_WAVE_RAM_START..=CH3_WAVE_RAM_END => self.ch3.wave_ram.write(address, value),
            _ => panic!("Invalid APU address: {:x}", address),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            CH1_START_ADDRESS..=CH1_END_ADDRESS => self.ch1.read(address),
            CH2_START_ADDRESS..=CH2_END_ADDRESS => self.ch2.read(address),
            CH3_START_ADDRESS..=CH3_END_ADDRESS => self.ch3.read(address),
            CH4_START_ADDRESS..=CH4_END_ADDRESS => 0,
            AUDIO_MASTER_CONTROL_ADDRESS => self.nr52_master_ctrl.read(),
            SOUND_PLANNING_ADDRESS => self.mixer.nr51_sound_panning.byte,
            MASTER_VOLUME_ADDRESS => self.mixer.nr50_master_volume.byte,
            CH3_WAVE_RAM_START..=CH3_WAVE_RAM_END => self.ch3.wave_ram.read(address),
            _ => panic!("Invalid APU address: {:x}", address),
        }
    }

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
    /// The frame sequencer generates low frequency clocks for the modulation units. It is clocked by a 512 Hz timer.
    fn sequence_frame(&mut self) {
        if self.ticks_count % FRAME_SEQUENCER_DIV as u32 == 0 {
            match self.frame_sequencer_step {
                0 => {
                    // 256 Hz, tick_length
                    self.ch1.tick_length(&mut self.nr52_master_ctrl);
                    self.ch2.tick_length(&mut self.nr52_master_ctrl);
                    self.ch3.tick_length(&mut self.nr52_master_ctrl);
                }
                1 => {}
                2 => {
                    // tick length, sweep
                    self.ch1.tick_length(&mut self.nr52_master_ctrl);
                    self.ch2.tick_length(&mut self.nr52_master_ctrl);
                    self.ch3.tick_length(&mut self.nr52_master_ctrl);
                }
                3 => {}
                4 => {
                    // 256 Hz, tick_length
                    self.ch1.tick_length(&mut self.nr52_master_ctrl);
                    self.ch2.tick_length(&mut self.nr52_master_ctrl);
                    self.ch3.tick_length(&mut self.nr52_master_ctrl);
                }
                5 => {}
                6 => {
                    // 128 Hz, tick length, sweep
                    self.ch1.tick_length(&mut self.nr52_master_ctrl);
                    self.ch2.tick_length(&mut self.nr52_master_ctrl);
                    self.ch3.tick_length(&mut self.nr52_master_ctrl);
                }
                7 => {} // 64 Hz, tick envelope
                _ => unreachable!(),
            }

            self.frame_sequencer_step = (self.frame_sequencer_step + 1) & 7;
        }
    }
}

/// FF26 — NR52: Audio master control
#[derive(Debug, Clone, Default, Copy)]
pub struct NR52 {
    byte: u8,
}

impl NR52 {
    pub fn write(&mut self, value: u8) {
        let prev_enabled = self.is_audio_on();
        let new_enabled = get_bit_flag(value, 7);

        if !new_enabled && prev_enabled {
            // APU is turning off, clear all but Wave RAM
            self.byte = 0;
        } else if new_enabled {
            // Turn on APU
            self.byte |= 0b1000_0000;
        }
    }

    pub fn read(&self) -> u8 {
        self.byte | 0b0111_0000 // Bits 4-6 always read as 1
    }

    pub fn is_audio_on(&self) -> bool {
        get_bit_flag(self.byte, 7)
    }

    /// Only the status of the channels’ generation circuits is reported
    pub fn is_ch_active(&self, ch_type: &ChannelType) -> bool {
        get_bit_flag(self.byte, Self::get_enable_bit_pos(ch_type))
    }

    pub fn deactivate_ch(&mut self, ch_type: &ChannelType) {
        set_bit(&mut self.byte, Self::get_enable_bit_pos(ch_type), false);
    }

    pub fn activate_ch(&mut self, ch_type: &ChannelType) {
        set_bit(&mut self.byte, Self::get_enable_bit_pos(ch_type), true);
    }

    fn get_enable_bit_pos(ch_type: &ChannelType) -> u8 {
        match ch_type {
            ChannelType::CH1 => 0,
            ChannelType::CH2 => 1,
            ChannelType::CH3 => 2,
            ChannelType::CH4 => 3,
        }
    }
}

/// FF25 — NR51:
/// Each channel can be panned hard left, center, hard right, or ignored entirely.
/// Setting a bit to 1 enables the channel to go into the selected output.
#[derive(Debug, Clone, Default)]
pub struct NR51 {
    pub byte: u8,
}

impl NR51 {
    pub fn ch4_left(&self) -> bool {
        get_bit_flag(self.byte, 7)
    }
    pub fn ch3_left(&self) -> bool {
        get_bit_flag(self.byte, 6)
    }
    pub fn ch2_left(&self) -> bool {
        get_bit_flag(self.byte, 5)
    }
    pub fn ch1_left(&self) -> bool {
        get_bit_flag(self.byte, 4)
    }
    pub fn ch4_right(&self) -> bool {
        get_bit_flag(self.byte, 3)
    }
    pub fn ch3_right(&self) -> bool {
        get_bit_flag(self.byte, 2)
    }
    pub fn ch2_right(&self) -> bool {
        get_bit_flag(self.byte, 1)
    }
    pub fn ch1_right(&self) -> bool {
        get_bit_flag(self.byte, 0)
    }
}

/// FF24 — NR50: Master volume & VIN panning
/// A value of 0 is treated as a volume of 1 (very quiet), and a value of 7 is treated as a volume of 8 (no volume reduction). Importantly, the amplifier never mutes a non-silent input.
#[derive(Default, Debug, Clone)]
pub struct NR50 {
    pub byte: u8,
}

impl NR50 {
    pub fn left_volume(&self) -> u8 {
        let vol = (self.byte >> 4) & 0b111; // Extract bits 6-4

        vol + 1 // Convert 0-7 to 1-8
    }

    pub fn right_volume(&self) -> u8 {
        let vol = self.byte & 0b111; // Extract bits 2-0

        vol + 1 // Convert 0-7 to 1-8
    }

    pub fn vin_left_enabled(&self) -> bool {
        self.byte & 0b1000_0000 != 0
    }

    pub fn vin_right_enabled(&self) -> bool {
        self.byte & 0b0000_1000 != 0
    }
}
