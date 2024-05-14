use uuid::Uuid;
use btleplug::api::Characteristic;

#[derive(Debug, Default)]
pub struct AbwChars {
    pub system_event: Option<Characteristic>,
    pub configuration: Option<Characteristic>,
    pub custom_cmd: Option<Characteristic>,
    pub custom_mcu_fw_update: Option<Characteristic>,
    pub custom_send_cli_cmd: Option<Characteristic>,
    pub custom_rcv_serial_data: Option<Characteristic>,

    pub model_number: Option<Characteristic>,
    pub serial_number: Option<Characteristic>,
    pub firmware_revision: Option<Characteristic>,
    pub software_revision: Option<Characteristic>,
    pub manufacturer_name: Option<Characteristic>,
    pub tx_power: Option<Characteristic>,
    pub tx_power_level: Option<Characteristic>,
    pub battery_level: Option<Characteristic>,
    pub battery_state: Option<Characteristic>,
    pub temperature_celsius: Option<Characteristic>,
    pub alert_level: Option<Characteristic>,
} 

pub const PERIPHERAL_NAME_MATCH_FILTER: &str = "ABW";

pub const SRV_ABEEWAY_PRIMARY             :Uuid = Uuid::from_u128(0x_00008A45_1212_efde_1523_785feabcd123);
pub const CHR_SYSTEM_EVENT                :Uuid = Uuid::from_u128(0x_00002742_1212_efde_1523_785feabcd123);
pub const CHR_CONFIGURATION               :Uuid = Uuid::from_u128(0x_00002740_1212_efde_1523_785feabcd123);
pub const CHR_CUSTOM_CMD                  :Uuid = Uuid::from_u128(0x_0000273D_1212_efde_1523_785feabcd123);
pub const CHR_CUSTOM_MCU_FW_UPDATE        :Uuid = Uuid::from_u128(0x_0000273E_1212_efde_1523_785feabcd123);
pub const CHR_CUSTOM_SEND_CLI_CMD         :Uuid = Uuid::from_u128(0x_00002748_1212_efde_1523_785feabcd123);
pub const CHR_CUSTOM_RCV_SERIAL_DATA      :Uuid = Uuid::from_u128(0x_00002749_1212_efde_1523_785feabcd123);

pub const SRV_DEVICE_INFORMATION          :Uuid = Uuid::from_u128(0x_0000180a_0000_1000_8000_00805f9b34fb);
pub const CHR_MODEL_NUMBER                :Uuid = Uuid::from_u128(0x_00002a24_0000_1000_8000_00805f9b34fb);
pub const CHR_SERIAL_NUMBER               :Uuid = Uuid::from_u128(0x_00002a25_0000_1000_8000_00805f9b34fb);
pub const CHR_FIRMWARE_REVISION           :Uuid = Uuid::from_u128(0x_00002a26_0000_1000_8000_00805f9b34fb);
pub const CHR_SOFTWARE_REVISION           :Uuid = Uuid::from_u128(0x_00002a28_0000_1000_8000_00805f9b34fb);
pub const CHR_MANUFACTURER_NAME           :Uuid = Uuid::from_u128(0x_00002a29_0000_1000_8000_00805f9b34fb);

pub const SRV_TX_POWER                    :Uuid = Uuid::from_u128(0x_00001804_0000_1000_8000_00805f9b34fb);
pub const CHR_TX_POWER_LEVEL              :Uuid = Uuid::from_u128(0x_00002a07_0000_1000_8000_00805f9b34fb);

pub const SRV_BATTERY                     :Uuid = Uuid::from_u128(0x_0000180f_0000_1000_8000_00805f9b34fb);
pub const CHR_BATTERY_LEVEL               :Uuid = Uuid::from_u128(0x_00002a19_0000_1000_8000_00805f9b34fb);
pub const CHR_BATTERY_POWER_STATE         :Uuid = Uuid::from_u128(0x_00002a1a_0000_1000_8000_00805f9b34fb);

pub const SRV_ENVIRONMENTAL_SENSING       :Uuid = Uuid::from_u128(0x_0000181a_0000_1000_8000_00805f9b34fb);
pub const CHR_TEMPERATURE_CELSIUS         :Uuid = Uuid::from_u128(0x_00002a1f_0000_1000_8000_00805f9b34fb);

pub const SRV_IMMEDIATE_ALERT             :Uuid = Uuid::from_u128(0x_00001802_0000_1000_8000_00805f9b34fb);
pub const CHR_ALERT_LEVEL                 :Uuid = Uuid::from_u128(0x_00002a06_0000_1000_8000_00805f9b34fb);
