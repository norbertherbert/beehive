
use anyhow::{Context, Result};
use btleplug::platform::Peripheral;
use btleplug::api::Peripheral as _ ;

use super::abw::{self, AbwChars};


pub async fn discover_chars(device: &Peripheral) -> Result<AbwChars> {

    let mut device_chars: abw::AbwChars = Default::default();

    device.discover_services().await.with_context(||"cannot discover BLE services")?;

    for char in device.characteristics() {
        match char.uuid {

            abw::CHR_SYSTEM_EVENT => {
                // println!("CHR_SYSTEM_EVENT");
                device_chars.system_event = Some(char);
            },
            abw::CHR_CONFIGURATION => {
                // println!("CHR_CONFIGURATION");
                device_chars.configuration = Some(char);
            },
            abw::CHR_CUSTOM_CMD => {
                // println!("CHR_CUSTOM_CMD");
                device_chars.custom_cmd = Some(char);
            },
            abw::CHR_CUSTOM_MCU_FW_UPDATE => {
                // println!("CHR_CUSTOM_MCU_FW_UPDATE");
                device_chars.custom_mcu_fw_update = Some(char);
            },
            abw::CHR_CUSTOM_SEND_CLI_CMD => {
                // println!("CHR_CUSTOM_SEND_CLI_CMD");
                device_chars.custom_send_cli_cmd = Some(char);
            },
            abw::CHR_CUSTOM_RCV_SERIAL_DATA => {
                // println!("CHR_CUSTOM_RCV_SERIAL_DATA");
                device_chars.custom_rcv_serial_data = Some(char);
            },


            abw::CHR_MODEL_NUMBER => {
                // println!("CHR_MODEL_NUMBER");
                device_chars.model_number = Some(char);
            },
            abw::CHR_SERIAL_NUMBER => {
                // println!("CHR_SERIAL_NUMBER");
                device_chars.serial_number = Some(char);
            },
            abw::CHR_FIRMWARE_REVISION => {
                // println!("CHR_FIRMWARE_REVISION");
                device_chars.firmware_revision = Some(char);
            },
            abw::CHR_SOFTWARE_REVISION => {
                // println!("CHR_SOFTWARE_REVISION");
                device_chars.software_revision = Some(char);
            },
            abw::CHR_MANUFACTURER_NAME => {
                // println!("CHR_MANUFACTURER_NAME");
                device_chars.manufacturer_name = Some(char);
            },
            abw::CHR_TX_POWER_LEVEL => {
                // println!("CHR_TX_POWER_LEVEL");
                device_chars.tx_power_level = Some(char);
            },
            abw::CHR_BATTERY_LEVEL => {
                // println!("CHR_BATTERY_LEVEL");
                device_chars.battery_level = Some(char);
            },
            abw::CHR_BATTERY_POWER_STATE => {
                // println!("CHR_BATTERY_POWER_STATE");
                device_chars.battery_state = Some(char);
            },
            abw::CHR_TEMPERATURE_CELSIUS => {
                // println!("CHR_TEMPERATURE_CELSIUS");
                device_chars.temperature_celsius = Some(char);
            },
            abw::CHR_ALERT_LEVEL => {
                // println!("CHR_ALERT_LEVEL");
                device_chars.alert_level = Some(char);
            },

            _ => {

            },

        }
    }

    Ok(device_chars)

}