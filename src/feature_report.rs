#[repr(u8)]
#[derive(Copy, Clone)]
pub enum LightingDirection {
    Down = 0x00,
    Up = 0x01
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum EffectSpeed {
    Low = 0x01,
    Medium = 0x02,
    High = 0x03
}

impl EffectSpeed {
    pub fn from_u8(value: u8) -> Self {
        return match value & 0x03 {
            0x01 => Self::Low,
            0x02 => Self::Medium,
            0x03 => Self::High,
            _ => Self::High
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum EffectBrightness {
    Low = 0x10,
    Medium = 0x20,
    High = 0x30,
    Highest = 0x40
}

impl EffectBrightness {
    pub fn from_u8(value: u8) -> Self {
        return match value & 0x70 {
            0x10 => Self::Low,
            0x20 => Self::Medium,
            0x30 => Self::High,
            0x40 => Self::Highest,
            _ => Self::Highest
        }
    }
}

#[derive(Copy, Clone)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl RGBColor {
    pub fn from_rbg_buffer(buffer: &[u8]) -> Self {
        return Self {
            red: *buffer.get(0).unwrap_or(&0),
            green: *buffer.get(2).unwrap_or(&0),
            blue: *buffer.get(1).unwrap_or(&0)
        };
    }

    pub fn to_rbg_buffer(&self) -> [u8; 3] {
        return [self.red, self.blue, self.green];
    }
}

#[derive(Copy, Clone)]
pub enum LightingEffect {
    Unknown,
    Off,
    GloriousMode { speed: EffectSpeed, direction: LightingDirection },
    Breathing { speed: EffectSpeed, colors: [RGBColor; 7] },
    SeamlessBreathingRGB { speed: EffectSpeed },
    SingleColor { color: RGBColor, brightness: EffectBrightness },
    BreathingSingleColor { color: RGBColor, speed: EffectSpeed },
    Tail { brightness: EffectBrightness, speed: EffectSpeed },
    Rave { colors: [RGBColor; 2], speed: EffectSpeed, brightness: EffectBrightness },
    Wave { speed: EffectSpeed, brightness: EffectBrightness }
}

impl LightingEffect {
    fn from_buffer(buffer: &[u8]) -> Self {
        // Lighting data starts at 53 and buffer len is 131
        // => 131 - 53 = 78
        if buffer.len() < 78 {
            return Self::Unknown;
        }

        return match buffer[0] {
            0x00 => LightingEffect::Off,
            0x01 => LightingEffect::GloriousMode {
                speed: EffectSpeed::from_u8(buffer[1]),
                direction: match buffer[2] {
                    0x00 => LightingDirection::Down,
                    _ => LightingDirection::Up
                }
            },
            0x02 => LightingEffect::SingleColor {
                color: RGBColor::from_rbg_buffer(&buffer[4..7]),
                brightness: EffectBrightness::from_u8(buffer[3])
            },
            0x03 => LightingEffect::Breathing {
                speed: EffectSpeed::from_u8(buffer[7]),
                colors: Self::rgbcolor_buffer_to_sized_array::<7>(&buffer[9..30])
            },
            0x04 => LightingEffect::Tail {
                brightness: EffectBrightness::from_u8(buffer[30]),
                speed: EffectSpeed::from_u8(buffer[30])
            },
            0x05 => LightingEffect::SeamlessBreathingRGB {
                speed: EffectSpeed::from_u8(buffer[31])
            },
            0x07 => LightingEffect::Rave {
                colors: Self::rgbcolor_buffer_to_sized_array::<2>(&buffer[64..70]),
                speed: EffectSpeed::from_u8(buffer[63]),
                brightness: EffectBrightness::from_u8(buffer[63])
            },
            0x09 => LightingEffect::Wave {
                speed: EffectSpeed::from_u8(buffer[71]),
                brightness: EffectBrightness::from_u8(buffer[71])
            },
            0x0A => LightingEffect::BreathingSingleColor {
                color: RGBColor::from_rbg_buffer(&buffer[73..76]),
                speed: EffectSpeed::from_u8(buffer[72])
            },
            _ => Self::Unknown
        };
    }

    fn set_in_buffer(&self, buffer: &mut [u8]) -> bool {
        if buffer.len() < 78 {
            return false;
        }

        match self {
            LightingEffect::Off | LightingEffect::Unknown => {
                buffer[0] = 0x00;
            }
            LightingEffect::GloriousMode { speed, direction } => {
                buffer[0] = 0x01;
                buffer[1] = EffectBrightness::Highest as u8 | *speed as u8;
                buffer[2] = *direction as u8;
            },
            LightingEffect::Breathing { speed, colors } => {
                buffer[0] = 0x03;
                buffer[7] = *speed as u8;
                buffer[9..30].copy_from_slice(&Self::rgbcolor_sized_array_to_rbg_buffer(colors));
            },
            LightingEffect::SeamlessBreathingRGB { speed } => {
                buffer[0] = 0x05;
                buffer[31] = *speed as u8;
            },
            LightingEffect::SingleColor { color, brightness } => {
                buffer[0] = 0x02;
                buffer[3] = *brightness as u8;
                buffer[4..7].copy_from_slice(&color.to_rbg_buffer());
            },
            LightingEffect::BreathingSingleColor { color, speed } => {
                buffer[0] = 0x0A;
                buffer[72] = *speed as u8;
                buffer[73..76].copy_from_slice(&color.to_rbg_buffer());
            },
            LightingEffect::Tail { brightness, speed } => {
                buffer[0] = 0x04;
                buffer[30] = *brightness as u8 | *speed as u8;
            },
            LightingEffect::Rave { colors, speed, brightness } => {
                buffer[0] = 0x07;
                buffer[63] = *brightness as u8 | *speed as u8;
                buffer[64..70].copy_from_slice(&Self::rgbcolor_sized_array_to_rbg_buffer(colors));
            },
            LightingEffect::Wave { speed, brightness } => {
                buffer[0] = 0x09;
                buffer[71] = *brightness as u8 | *speed as u8;
            }
        }

        if matches!(self, LightingEffect::Off) {
            buffer[77] = 0x03;
        } else {
            buffer[77] = 0x00;
        }

        return true;
    }

    fn rgbcolor_buffer_to_sized_array<const ARRAY_SIZE: usize>(buffer: &[u8]) -> [RGBColor; ARRAY_SIZE] {
        let mut color_array = [RGBColor { red: 0, green: 0, blue: 0}; ARRAY_SIZE];
        buffer
            .chunks_exact(3)
            .enumerate()
            .for_each(|(index, element)| {
                if color_array.len() > index && element.len() == 3 {
                    color_array[index] = RGBColor::from_rbg_buffer(&[element[0], element[1], element[2]]);
                }
            });

        return color_array;
    }

    fn rgbcolor_sized_array_to_rbg_buffer(colors: &[RGBColor]) -> Vec<u8> {
        return colors
            .iter()
            .map(|color| color.to_rbg_buffer())
            .flatten()
            .collect::<Vec<u8>>();
    }
}

pub struct FeatureReport {
    raw_data: Vec<u8>,
    lighting_effect: LightingEffect,
}

impl FeatureReport {
    pub fn from_buffer(buffer: &[u8]) -> Option<Self> {
        if buffer.len() < 131 {
            return None;
        }

        return Some(Self {
            raw_data: Vec::from(buffer),
            lighting_effect: LightingEffect::from_buffer(&buffer[53..])
        });
    }

    pub fn to_buffer(&self) -> [u8; 520] {
        let mut data = self.raw_data.clone();
        data.resize(520, 0x00);

        return data.try_into().unwrap();
    }

    pub fn lighting_effect(&self) -> LightingEffect {
        return self.lighting_effect.clone();
    }

    pub fn set_lighting_effect(&mut self, effect: LightingEffect) {
        effect.set_in_buffer(&mut self.raw_data[53..]);
        self.lighting_effect = effect;
    }
}