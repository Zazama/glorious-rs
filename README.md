# glorious-rs

Control the Glorious Model O lighting effects.

## Usage

```rust
let device = GloriousDevice::new().unwrap();

// Get current device settings
let mut settings = device.get_settings().unwrap();

// Change the lighting effect
let new_lighting_effect = LightingEffect::SingleColor {
    color: RGBColor::from_rbg_buffer(&[0x22, 0x24, 0x23]),
    brightness: EffectBrightness::High
};

settings.set_lighting_effect(new_lighting_effect);
device.commit_settings(&settings).unwrap();
```