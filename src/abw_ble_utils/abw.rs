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

// **********************************************
// *** ABEEWAY PRIMARY SERVICE
// **********************************************

pub const SRV_ABEEWAY_PRIMARY             :Uuid = Uuid::from_u128(0x_00008A45_1212_efde_1523_785feabcd123);

// ABEEWAY PRIMARY SERVICE - SYSTEM EVENT CHARACTERISTICS

pub const CHR_SYSTEM_EVENT                :Uuid = Uuid::from_u128(0x_00002742_1212_efde_1523_785feabcd123);

// ABEEWAY PRIMARY SERVICE - CONFIGURATIION CHARACTERISTICS

pub const CHR_CONFIGURATION               :Uuid = Uuid::from_u128(0x_00002740_1212_efde_1523_785feabcd123);

// Protocol Payloads: 
pub const WR_WRITE_CONF            :u8 = 0x01;   // WR_WRITE_PARAM(1B)|ParamId(1B)|ParamValue(4B)
pub const WR_READ_CONF             :u8 = 0x00;   // WR_READ_PARAM(1B)
pub const NOTIF_CONF_SUCCESS       :u8 = 0x00;
pub const NOTIF_CONF_INVALID       :u8 = 0x01;
// NOTIFICATION Response: NOTIF_CONF_SUCCESS(1B)/NOTIF_CONF_INVALID(1B)|ParamId(1B)|ParamValue(4B) 


// ABEEWAY PRIMARY SERVICE - CUSTOM - COMMAND CHARACTERISTICS

pub const CHR_CUSTOM_CMD                  :Uuid = Uuid::from_u128(0x_0000273D_1212_efde_1523_785feabcd123);

// Protocol Payloads: 
pub const WR_CLEAR_BOND            :u8 = 0x99;
pub const WR_RESET_TRACKER         :u8 = 0xfe;
pub const WR_POWER_OFF_TRACKER     :u8 = 0xff;
pub const WR_FAST_CONN             :u8 = 0x00;
pub const WR_SLOW_CONN             :u8 = 0x01;
pub const WR_VERY_FAST_CONN        :u8 = 0x02;

pub const RR_CUST_CMD_READ_ANS     :u8 = 0x00; // RR_READ_ANS

// Possible READ_RSP Payloads:
// [0x00, S]; S => Status (S=0x00 means success)


// ABEEWAY PRIMARY SERVICE - CUSTOM - MCU FIRMWARE UPDATE

pub const CHR_CUSTOM_MCU_FW_UPDATE        :Uuid = Uuid::from_u128(0x_0000273E_1212_efde_1523_785feabcd123);

// Protocol Payloads: 

pub const WR_ENABLE_DFU                   :u8 = 0x00; // WR_ENABLE_DFU(1B)|DevEUI(8B)
pub const WR_START_DFU                    :u8 = 0x01; // WR_START_DFU(1B)|BinarySize(4B)
pub const WR_WRITE_BINARY_DATA            :u8 = 0x02; // WR_WRITE_BINARY_DATA(1B)|Address(3B)|Data(16B)
pub const WR_BINARY_DATA_CRC              :u8 = 0x03; // WR_BINARY_DATA_CRC(1B)|CRC(2B)
pub const WR_ABORT_DFU                    :u8 = 0x04; // WR_ABORT_DFU(1B)

pub const RR_MCU_FW_UPDATE_READ_ANS       :u8 = 0x00; // RR_READ_ANS

pub const NOTIF_WRITE_BINARY_DATA_ACK     :u8 = 0x02; // NOTIF_WRITE_BINARY_DATA_ACK(1B)|Status(1B)|Address(3B)




pub const FW_UPDATE_COMPLETED_SUCCESSFULLY :u8 = 0x00; //	Y	The operation completed successfully
pub const FW_SRV_NOT_IMITIALIZED :u8 = 0x01; //	Y	Service not  initialized
pub const FW_STORAGE_NOT_INITIALIZED :u8 = 0x02; //	N	Storage not initialized
pub const FW_INVALID_DEV_EUI :u8 = 0x03; //	N	Invalid Device EUI argument in the message
pub const FW_INTERNAL_ERROR_04 :u8 = 0x04; //	N	Internal error
pub const FW_INTERNAL_ERROR_05 :u8 = 0x05; //	N	Internal error
pub const FW_OPERATION_TIMEOUT :u8 = 0x06; //	N	Operation timeout
pub const FW_INTERNAL_ERROR_07 :u8 = 0x07; //	N	Internal error
pub const FW_INTERNAL_ERROR_08 :u8 = 0x08; //	N	Internal error
pub const FW_CRC_ERROR :u8 = 0x09; //	N	Binary CRC error
pub const FW_STORAGE_OPERATION_ERROR :u8 = 0x0A; //	N	Storage operation error
pub const FW_ADDRESS_RECEIVED_ERROR :u8 = 0x0B; //	Y	Address received error
pub const FW_BINARY_LENGTH_ERROR :u8 = 0x0c; //	N	Binary length error
pub const FW_BINARY_LENGTH_ERRROR :u8 = 0x0D; //	N	Device EUI mismatch
pub const FW_BATTERY_LEVEL_TOO_LOW :u8 = 0x0E; //	Y	Battery level too low to start the DFU
pub const FW_STORAGE_ERROR :u8 = 0x0F; //	N	Storage error

pub const FW_ERRORS: [&str; 16] = [
    "The operation completed successfully",
    "Service not  initialized",
    "Storage not initialized",
    "Invalid Device EUI argument in the message",
    "Internal error",
    "Internal error",
    "Operation timeout",
    "Internal error",
    "Internal error",
    "Binary CRC error",
    "Storage operation error",
    "Address received error",
    "Binary length error",
    "Device EUI mismatch",
    "Battery level too low to start the DFU",
    "Storage error",
];





// ABEEWAY PRIMARY SERVICE - CUSTOM - SEND CLI COMMAND

pub const CHR_CUSTOM_SEND_CLI_CMD         :Uuid = Uuid::from_u128(0x_00002748_1212_efde_1523_785feabcd123);

// ABEEWAY PRIMARY SERVICE - CUSTOM - RECEIVE SERIAL DATA

pub const CHR_CUSTOM_RCV_SERIAL_DATA      :Uuid = Uuid::from_u128(0x_00002749_1212_efde_1523_785feabcd123);



// **********************************************
// *** NORDIC DFU SERVICE
// **********************************************

// https://developer.nordicsemi.com/nRF51_SDK/nRF51_SDK_v8.x.x/doc/8.0.0/s110/html/a00103.html

// pub const DFU_CONTROL_POINT 0x1531
// pub const DFU_PACKET 0x1532
// pub const DFU_VERSION 0x1534


// **********************************************
// *** DEVICE INFORMATION SERVICE
// **********************************************

pub const SRV_DEVICE_INFORMATION          :Uuid = Uuid::from_u128(0x_0000180a_0000_1000_8000_00805f9b34fb);
pub const CHR_MODEL_NUMBER                :Uuid = Uuid::from_u128(0x_00002a24_0000_1000_8000_00805f9b34fb);
pub const CHR_SERIAL_NUMBER               :Uuid = Uuid::from_u128(0x_00002a25_0000_1000_8000_00805f9b34fb);
pub const CHR_FIRMWARE_REVISION           :Uuid = Uuid::from_u128(0x_00002a26_0000_1000_8000_00805f9b34fb);
pub const CHR_SOFTWARE_REVISION           :Uuid = Uuid::from_u128(0x_00002a28_0000_1000_8000_00805f9b34fb);
pub const CHR_MANUFACTURER_NAME           :Uuid = Uuid::from_u128(0x_00002a29_0000_1000_8000_00805f9b34fb);

// **********************************************
// *** TX POWER SERVICE
// **********************************************

pub const SRV_TX_POWER                    :Uuid = Uuid::from_u128(0x_00001804_0000_1000_8000_00805f9b34fb);
pub const CHR_TX_POWER_LEVEL              :Uuid = Uuid::from_u128(0x_00002a07_0000_1000_8000_00805f9b34fb);

// **********************************************
// *** BATTERY SERVICE
// **********************************************

pub const SRV_BATTERY                     :Uuid = Uuid::from_u128(0x_0000180f_0000_1000_8000_00805f9b34fb);
pub const CHR_BATTERY_LEVEL               :Uuid = Uuid::from_u128(0x_00002a19_0000_1000_8000_00805f9b34fb);
pub const CHR_BATTERY_POWER_STATE         :Uuid = Uuid::from_u128(0x_00002a1a_0000_1000_8000_00805f9b34fb);

pub const CHARGER_PRESENT_AND_CHARGING        :u8 = 0x77; // => "Charger present and charging.",
pub const CHARGER_PRESENT_BUT_NOT_CHARGING    :u8 = 0x67; // => "Charger present but not charging.",
pub const CHARGER_NOT_PRESENT_AND_DISCHARGING :u8 = 0x66; // => "Charger not present and discharging.",

// **********************************************
// *** ENVIRONMENTAL SENSING SERVICE
// **********************************************

pub const SRV_ENVIRONMENTAL_SENSING       :Uuid = Uuid::from_u128(0x_0000181a_0000_1000_8000_00805f9b34fb);
pub const CHR_TEMPERATURE_CELSIUS         :Uuid = Uuid::from_u128(0x_00002a1f_0000_1000_8000_00805f9b34fb);

// **********************************************
// *** IMMEDIATE ALERT SERVICE
// **********************************************

pub const SRV_IMMEDIATE_ALERT             :Uuid = Uuid::from_u128(0x_00001802_0000_1000_8000_00805f9b34fb);
pub const CHR_ALERT_LEVEL                 :Uuid = Uuid::from_u128(0x_00002a06_0000_1000_8000_00805f9b34fb);

pub const NO_ALERT     :u8 = 0x00;
pub const MILD_ALERT  :u8 = 0x01;
pub const HIGH_ALERT   :u8 = 0x02;


pub const FIX_FOR_NOT_ADVERTIZING: &'static str  = "
Make sure that the device you are looking for is advertizing and try again.
- Abeeway Smart Badges and Microtrackers will start advertizing for a few min right after they are turned on.
- Abeeway Compact trackers will start advertizing after placing and removing a magnet at their marked sides multiple times.
";

pub const FIX_FOR_CORRUPTED_PAIRING: &'static str  = "
Device cannot be connected. It is either not advertizing or its pairing is corrupted.
- Abeeway Smart Badges and Microtrackers will start advertizing for a few min right after they are turned on.
- Abeeway Compact trackers will start advertizing after placing and removing a magnet at their marked sides multiple times.
You can fix corrupted pairing in the following way:
1.   Turn OFF and then ON Bluetooth on your computer. 
1.1      Try to connect your device again. If this does not fix your issue, contunie with the next step. 
2.   Make a new pairing for your device: 
2.1.   Remove the BLE bond on your OS by using your OS's GUI
2.2    Make sure your device is advertizing
2.3.   Remove the BLE bond on your device by executing the following command:
           abeehive remove-bond <DEVICE>
2.4    Pair your device with your computer again.
";

pub const FIX_FOR_NOT_PAIRED: &'static str  = "
It seems that your device is not paired while the requested action requires authentication.
Please pair your device using your OS's GUI and try again.
The device may have an old bond to this or another computer. In such a case the OS will not find the device when you try to add.
You can fix this by executing the following command
    beehive remove-bond <DEVICE>
";