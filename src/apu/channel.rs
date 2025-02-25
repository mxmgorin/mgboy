// Square 1: Sweep -> Timer -> Duty -> Length Counter -> Envelope -> Mixer
// Square 2:          Timer -> Duty -> Length Counter -> Envelope -> Mixer
// Wave:              Timer -> Wave -> Length Counter -> Volume   -> Mixer
// Noise:             Timer -> LFSR -> Length Counter -> Envelope -> Mixer
#[derive(Clone, Debug)]
pub enum ChannelType {
    CH1,
    CH2,
    CH3,
    CH4,
}

impl ChannelType {
    pub fn get_enable_bit_pos(&self) -> u8 {
        match self {
            ChannelType::CH1 => 0,
            ChannelType::CH2 => 1,
            ChannelType::CH3 => 2,
            ChannelType::CH4 => 3,
        }
    }

    pub fn get_initial_length_timer(&self) -> u16 {
        match self {
            ChannelType::CH1 | ChannelType::CH2 | ChannelType::CH4 => 64,
            ChannelType::CH3 => 256,
        }
    }

    pub fn get_length_timer_mask(&self) -> u8 {
        match self {
            ChannelType::CH1 | ChannelType::CH2 | ChannelType::CH4 => 0b0011_1111,
            ChannelType::CH3 => 0xFF,
        }
    }
}
