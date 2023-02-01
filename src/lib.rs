mod feature_report;
extern crate hidapi;

use feature_report::{FeatureReport};
use hidapi::{HidDevice, HidError, DeviceInfo, HidApi};

const VENDOR_ID: u16 = 9610;
const PRODUCT_ID: u16 = 39;

pub struct GloriousDevice {
    data_device: HidDevice,
    control_device: HidDevice
}

impl GloriousDevice {
    pub fn new() -> Result<Self, HidError> {
        let api = HidApi::new()?;

        // Get all model o device infos
        let devices = Self::filter_model_o_devices(&api);
        let data_device = Self::find_data_device(&api, &devices);
        let control_device = Self::find_control_device(&api, &devices);

        if data_device.is_some() && control_device.is_some() {
            return Ok(GloriousDevice {
                data_device: data_device.unwrap(),
                control_device: control_device.unwrap()
            });
        }

        return Err(HidError::HidApiError { message: "Device not found".to_owned() });
    }

    // Filter device list for vendor / product id
    fn filter_model_o_devices(api: &HidApi) -> Vec<&DeviceInfo> {
        return api
            .device_list()
            .filter(|info|
                info.vendor_id() == VENDOR_ID &&
                info.product_id() == PRODUCT_ID
            )
            .collect();
    }

    fn find_data_device(api: &HidApi, devices: &Vec<&DeviceInfo>) -> Option<HidDevice> {
        return devices
            .iter()
            // Only check devices that can actually be opened
            .filter_map(|info| info.open_device(&api).ok())
            .filter(|device| {
                // We will create a ReportID 4 buffer to test if it gets sent
                // by the HID driver. If it fails, we got the wrong device.
                let mut buffer: [u8; 520] = [0x00; 520];
                buffer[0] = 0x04;
                return device
                    .send_feature_report(&mut buffer)
                    .is_ok();
            }).next();
    }

    fn find_control_device(api: &HidApi, devices: &Vec<&DeviceInfo>) -> Option<HidDevice> {
        return devices
            .iter()
            // Only check devices that can actually be opened
            .filter_map(|info| info.open_device(&api).ok())
            .filter(|device| {
                // We will create a ReportID 5 buffer to test if it gets sent
                // by the HID driver. If it fails, we got the wrong device.
                let mut buffer: [u8; 6] = [0x05, 0x00, 0x00, 0x00, 0x00, 0x00];
                return device
                    .send_feature_report(&mut buffer)
                    .is_ok();
            }).next();
    }

    fn prepare_settings_request(&self) -> Result<(), HidError> {
        let mut req = [0x05, 0x11, 0, 0, 0, 0];
        self.control_device.send_feature_report(&mut req)?;
        return Ok(());
    }

    pub fn get_settings(&self) -> Result<FeatureReport, HidError> {
        self.prepare_settings_request()?;
        let mut buffer: [u8; 520] = [0x00; 520];
        buffer[0] = 0x04;
        self.data_device.get_feature_report(&mut buffer)?;

        return Ok(
            FeatureReport::from_buffer(&buffer)
                .ok_or(HidError::HidApiError {
                    message: "Bad data".to_owned()
                })?
        );
    }

    pub fn commit_settings(
        &self, report: &FeatureReport
    ) -> Result<(), HidError> {
        let mut report_buffer = Vec::from(report.to_buffer());
        // When sending the settings, Byte[3] is always 0x7B!
        report_buffer[3] = 0x7B;
        self.data_device.send_feature_report(&report_buffer)?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::{GloriousDevice, feature_report::{LightingEffect, EffectBrightness, RGBColor}};

    #[test]
    fn connect() {
        GloriousDevice::new().unwrap();
    }

    #[test]
    fn lighting_effect() {
        let device = GloriousDevice::new().unwrap();

        // Test Get_Report ID 4 working
        let mut settings = device.get_settings().unwrap();

        let new_lighting_effect = LightingEffect::SingleColor {
            color: RGBColor::from_rbg_buffer(&[0x22, 0x24, 0x23]),
            brightness: EffectBrightness::High
        };

        settings.set_lighting_effect(new_lighting_effect);
        // Test Set_Report ID 4 working
        device.commit_settings(&settings).unwrap();

        match device.get_settings().unwrap().lighting_effect() {
            LightingEffect::SingleColor { color, brightness } => {
                assert!(color.red == 0x22);
                assert!(color.green == 0x23);
                assert!(color.blue == 0x24);
                assert!(matches!(brightness, EffectBrightness::High));
            },
            _ => assert!(false)
        };
    }
}
