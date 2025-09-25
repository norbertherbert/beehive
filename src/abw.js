export async function findServices(server) {
    let srv;
    let chr;

    srv = services.abeeway_primary;
    srv.obj = await server.getPrimaryService(srv.uuid);
    chr = srv.chars.system_event;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.configuration;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.custom_simple_cmd;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.custom_mcu_fw_update;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.custom_send_cli_cmd;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.custom_rcv_serial_data;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);

    srv = services.device_information;
    srv.obj = await server.getPrimaryService(srv.uuid);
    chr = srv.chars.model_number;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    // chr = srv.chars.serial_number;
    // chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.firmware_revision;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.software_revision;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.manufacturer_name;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);

    srv = services.tx_power;
    srv.obj = await server.getPrimaryService(srv.uuid);
    chr = srv.chars.tx_power_level;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);

    srv = services.battery;
    srv.obj = await server.getPrimaryService(srv.uuid);
    chr = srv.chars.battery_level;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);
    chr = srv.chars.battery_power_state;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);

    srv = services.environmental_sensing;
    srv.obj = await server.getPrimaryService(srv.uuid);
    chr = srv.chars.temperature_celsius;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);

    srv = services.immediate_alert;
    srv.obj = await server.getPrimaryService(srv.uuid);
    chr = srv.chars.alert_level;
    chr.obj = await srv.obj.getCharacteristic(chr.uuid);

}


export let services = {
    abeeway_primary: {
        name: 'Abeeway Primary Service',
        uuid: '00008a45-1212-efde-1523-785feabcd123',
        obj: null,
        chars: {
            system_event: {
                name: 'System Event',
                uuid: '00002742-1212-efde-1523-785feabcd123',
                obj: null
            },
            configuration: {
                name: 'Confihuration',
                uuid: '00002740-1212-efde-1523-785feabcd123',
                obj: null
            },
            custom_simple_cmd: {
                name: 'Custom: Command',
                uuid: '0000273d-1212-efde-1523-785feabcd123',
                obj: null
            },
            custom_mcu_fw_update: {
                name: 'Custom: MCU Firmware Update',
                uuid: '0000273e-1212-efde-1523-785feabcd123',
                obj: null
            },
            custom_send_cli_cmd: {
                name: 'Custom: Send CLI Command',
                uuid: '00002748-1212-efde-1523-785feabcd123',
                obj: null
            },
            custom_rcv_serial_data: {
                name: 'Custom: Receive Serial Data',
                uuid: '00002749-1212-efde-1523-785feabcd123',
                obj: null
            },
        }
    },
    device_information: {
        name: "Device Information Service",
        uuid: '0000180a-0000-1000-8000-00805f9b34fb',
        chars: {
            model_number: {
                name: 'Model Number',
                uuid: '00002a24-0000-1000-8000-00805f9b34fb',
                obj: null
            },
            serial_number: {
                name: 'Serial Numbeer',
                uuid: '00002a25-0000-1000-8000-00805f9b34fb',
                obj: null
            },
            firmware_revision: {
                name: 'Firmware Revision',
                uuid: '00002a26-0000-1000-8000-00805f9b34fb',
                obj: null
            },
            software_revision: {
                name: 'Software Reevision',
                uuid: '00002a28-0000-1000-8000-00805f9b34fb',
                obj: null
            },
            manufacturer_name: {
                name: 'Manufacturer Name',
                uuid: '00002a29-0000-1000-8000-00805f9b34fb',
                obj: null
            },
        }
    },
    tx_power: {
        name: 'TX Power Service',
        uuid: '00001804-0000-1000-8000-00805f9b34fb',
        obj: null,
        chars: {
            tx_power_level: {
                name: 'TX Power Level',
                uuid: '00002a07-0000-1000-8000-00805f9b34fb',
                obj: null
            },
        }
    },
    battery: {
        name: 'Battery Service',
        uuid: '0000180f-0000-1000-8000-00805f9b34fb',
        obj: null,
        chars: {
            battery_level: {
                name: 'Battery Level [%]',
                uuid: '00002a19-0000-1000-8000-00805f9b34fb',
                obj: null
            },
            battery_power_state: {
                name: 'Battery State',
                uuid: '00002a1a-0000-1000-8000-00805f9b34fb',
                obj: null
            },
        }
    },
    environmental_sensing: {
        name: 'Environmental Sensing Service',
        uuid: '0000181a-0000-1000-8000-00805f9b34fb',
        obj: null,
        chars: {
            temperature_celsius: {
                name: 'Temperature [°C]',
                uuid: '00002a1f-0000-1000-8000-00805f9b34fb',
                obj: null
            },
        }
    },
    immediate_alert: {
        name: 'Immediate Alert Service',
        uuid: '00001802-0000-1000-8000-00805f9b34fb',
        obj: null,
        chars: {
            alert_level: {
                name: 'Alert Level',
                uuid: '00002a06-0000-1000-8000-00805f9b34fb',
                obj: null
            },
        }
    },
}



export const PERIPHERAL_NAME_MATCH_FILTER = "ABW";

// **********************************************
// *** ABEEWAY PRIMARY SERVICE
// **********************************************

export const SRV_ABEEWAY_PRIMARY = '00008a45-1212-efde-1523-785feabcd123';

// ABEEWAY PRIMARY SERVICE - SYSTEM EVENT CHARACTERISTICS

export const CHR_SYSTEM_EVENT    = '00002742-1212-efde-1523-785feabcd123';

// ABEEWAY PRIMARY SERVICE - CONFIGURATIION CHARACTERISTICS

export const CHR_CONFIGURATION   = '00002740-1212-efde-1523-785feabcd123';

// Protocol Payloads:
export const WR_WRITE_CONF      = 0x01;   // WR_WRITE_PARAM(1B)|ParamId(1B)|ParamValue(4B)
export const WR_READ_CONF       = 0x00;   // WR_READ_PARAM(1B)

export const NOTIF_CONF_SUCCESS = 0x00;
export const NOTIF_CONF_INVALID = 0x01;
export const NOTIF_CONF_FAIL = 0x01;
export const NOTIF_CONF_VALUE_NOT_SUPPORTED = 0x02;
export const NOTIF_CONF_DATA_LENGTH_ERROR = 0x03;

// NOTIFICATION Response: NOTIF_CONF_SUCCESS(1B)/NOTIF_CONF_INVALID(1B)|ParamId(1B)|ParamValue(4B)


// ABEEWAY PRIMARY SERVICE - CUSTOM - COMMAND CHARACTERISTICS

export const CHR_CUSTOM_SIMPLE_CMD = '0000273d-1212-efde-1523-785feabcd123';

// Protocol Payloads:
export const WR_CLEAR_BOND         = 0x99;
export const WR_RESET_TRACKER      = 0xfe;
export const WR_POWER_OFF_TRACKER  = 0xff;

export const WR_FAST_CONN          = 0x00;
export const WR_SLOW_CONN          = 0x01;
export const WR_VERY_FAST_CONN     = 0x02;

export const WR_ENABLE_BLE_CLI     = 0x03;
export const WR_DISABLE_BLE_CLI    = 0x04;
export const WR_SAVE_CONFIG        = 0x05;


export const RR_CUST_CMD_READ_ANS  = 0x00; // RR_READ_ANS

// Possible READ_RSP Payloads:
// [0x00, S]; S => Status (S=0x00 means success)


// ABEEWAY PRIMARY SERVICE - CUSTOM - MCU FIRMWARE UPDATE

export const CHR_CUSTOM_MCU_FW_UPDATE = '0000273e-1212-efde-1523-785feabcd123';

// Protocol Payloads:

export const WR_ENABLE_DFU        = 0x00; // WR_ENABLE_DFU(1B)|DevEUI(8B)
export const WR_START_DFU         = 0x01; // WR_START_DFU(1B)|BinarySize(4B)
export const WR_WRITE_BINARY_DATA = 0x02; // WR_WRITE_BINARY_DATA(1B)|Address(3B)|Data(16B)
export const WR_BINARY_DATA_CRC   = 0x03; // WR_BINARY_DATA_CRC(1B)|CRC(2B)
export const WR_ABORT_DFU         = 0x04; // WR_ABORT_DFU(1B)

export const RR_MCU_FW_UPDATE_READ_ANS   = 0x00; // RR_READ_ANS

export const NOTIF_WRITE_BINARY_DATA_ACK = 0x02; // NOTIF_WRITE_BINARY_DATA_ACK(1B)|Status(1B)|Address(3B)

// *************************************
// For AT2 FW Update
// *************************************

export const AT2_DFU_OPERATION_SUCCESS        = 0x00;
export const AT2_DFU_SRV_NOT_INITIALIZED      = 0x01;
export const AT2_DFU_STORAGE_NOT_INITIALIZED  = 0x02;
export const AT2_DFU_INVALID_DEV_EUI          = 0x03;
export const AT2_DFU_INTERNAL_ERROR_04        = 0x04;
export const AT2_DFU_INTERNAL_ERROR_05        = 0x05;
export const AT2_DFU_OPERATION_TIMEOUT        = 0x06;
export const AT2_DFU_INTERNAL_ERROR_07        = 0x07;
export const AT2_DFU_INTERNAL_ERROR_08        = 0x08;
export const AT2_DFU_CRC_ERROR                = 0x09;
export const AT2_DFU_STORAGE_OPERATION_ERROR  = 0x0A;
export const AT2_DFU_ADDRESS_RECEIVED_ERROR   = 0x0B;
export const AT2_DFU_BINARY_LENGTH_ERROR      = 0x0C;
export const AT2_DFU_BINARY_LENGTH_ERRROR     = 0x0D;
export const AT2_DFU_BATTERY_LEVEL_TOO_LOW    = 0x0E;
export const AT2_DFU_STORAGE_ERROR            = 0x0F;

export const AT2_DFU_STATUS_ARRAY = [
    'The operation completed successfully',
    'Service not initialized',
    'Storage not initialized',
    'Invalid Device EUI argument in the message',
    'Internal error',
    'Internal error',
    'Operation timeout',
    'Internal error',
    'Internal error',
    'Binary CRC error',
    'Storage operation error',
    'Address received error',
    'Binary length error',
    'Device EUI mismatch',
    'Battery level too low to start the DFU',
    'Storage error',
]



// *************************************
// For AT3 Fw upddate
// *************************************

export const DFU_OPERATION_SUCCESS             = 0x00;
export const DFU_SRV_NOT_INITIALIZED           = 0x01;
export const DFU_INVALID_STATE                 = 0x02;
export const DFU_WIFI_INIT_ERROR               = 0x03;
export const DFU_PAYLOAD_DATA_ERROR            = 0x04;
export const DFU_DATA_NULL                     = 0x05;
export const DFU_OPERATION_TIMEOUT             = 0x06;
export const DFU_INTERNAL_ERROR                = 0x07;
export const DFU_OPERATION_ABORTED             = 0x08;
export const DFU_CRC_ERROR                     = 0x09;
export const DFU_OSP_PARSING_ERROR             = 0x0A;
export const DFU_ADDRESS_OFFSET_ERROR          = 0x0B;
export const DFU_BINARY_LENGTH_ERROR           = 0x0C;
export const DFU_DEVEUI_MISMATCH               = 0x0D;
export const DFU_BATTERY_LEVEL_TOO_LOW         = 0x0E;
export const DFU_FLASH_STORAGE_ERROR           = 0x0F;

export const DFU_STATUS_ARRAY = [
    'The operation completed successfully', // 0x00
    'Service not initialized', // 0x01
    'Invalid state', // 0x02
    'WiFi init error', // 0x03
    'Payload data error', // 0x04
    'Data null error', // 0x05
    'Operation timeout', // 0x06
    'Internal error', // 0x07
    'Operation aaborted', // 0x08
    'Binary CRC error', // 0x09
    'OSP parsing error', // 0x0A
    'Address offset error', // 0x0B
    'Binary length error', // 0x0C
    'Device EUI mismatch', // 0x0D
    'Battery level too low', // 0x0E
    'Flash storage error', // 0x0F
]




// ABEEWAY PRIMARY SERVICE - CUSTOM - SEND CLI COMMAND

export const CHR_CUSTOM_SEND_CLI_CMD = '00002748-1212-efde-1523-785feabcd123';

// ABEEWAY PRIMARY SERVICE - CUSTOM - RECEIVE SERIAL DATA

export const CHR_CUSTOM_RCV_SERIAL_DATA = '00002749-1212-efde-1523-785feabcd123';



// **********************************************
// *** NORDIC DFU SERVICE
// **********************************************

// https://developer.nordicsemi.com/nRF51_SDK/nRF51_SDK_v8.x.x/doc/8.0.0/s110/html/a00103.html

// export const DFU_CONTROL_POINT 0x1531
// export const DFU_PACKET 0x1532
// export const DFU_VERSION 0x1534


// **********************************************
// *** DEVICE INFORMATION SERVICE
// **********************************************

export const SRV_DEVICE_INFORMATION = '0000180a-0000-1000-8000-00805f9b34fb';
export const CHR_MODEL_NUMBER       = '00002a24-0000-1000-8000-00805f9b34fb';
export const CHR_SERIAL_NUMBER      = '00002a25-0000-1000-8000-00805f9b34fb';
export const CHR_FIRMWARE_REVISION  = '00002a26-0000-1000-8000-00805f9b34fb';
export const CHR_SOFTWARE_REVISION  = '00002a28-0000-1000-8000-00805f9b34fb';
export const CHR_MANUFACTURER_NAME  = '00002a29-0000-1000-8000-00805f9b34fb';

// **********************************************
// *** TX POWER SERVICE
// **********************************************

export const SRV_TX_POWER = '00001804-0000-1000-8000-00805f9b34fb';
export const CHR_TX_POWER_LEVEL = '00002a07-0000-1000-8000-00805f9b34fb';

// **********************************************
// *** BATTERY SERVICE
// **********************************************

export const SRV_BATTERY = '0000180f-0000-1000-8000-00805f9b34fb';
export const CHR_BATTERY_LEVEL = '00002a19-0000-1000-8000-00805f9b34fb';
export const CHR_BATTERY_POWER_STATE = '00002a1a-0000-1000-8000-00805f9b34fb';

export const CHARGER_PRESENT_AND_CHARGING = 0x77; // => "Charger present and charging.",
export const CHARGER_PRESENT_BUT_NOT_CHARGING = 0x67; // => "Charger present but not charging.",
export const CHARGER_NOT_PRESENT_AND_DISCHARGING = 0x66; // => "Charger not present and discharging.",

// **********************************************
// *** ENVIRONMENTAL SENSING SERVICE
// **********************************************

export const SRV_ENVIRONMENTAL_SENSING = '0000181a-0000-1000-8000-00805f9b34fb';
export const CHR_TEMPERATURE_CELSIUS = '00002a1f-0000-1000-8000-00805f9b34fb';

// **********************************************
// *** IMMEDIATE ALERT SERVICE
// **********************************************

export const SRV_IMMEDIATE_ALERT = '00001802-0000-1000-8000-00805f9b34fb';
export const CHR_ALERT_LEVEL = '00002a06-0000-1000-8000-00805f9b34fb';

export const NO_ALERT = 0x00;
export const MILD_ALERT = 0x01;
export const HIGH_ALERT = 0x02;

export const DEV_EUI_PREFIX_AT2 = "20635F0";
export const DEV_EUI_PREFIX_AT3 = "20635F";



export const CONNECTION_ALERT = `
PLEASE NOTE THAT BEFORE YOU CONNECT ANY DEVICE, YOU MUST PAIR IT WITH YOUR OPERATING SYSTEM MANUALLY!
https://support.google.com/chrome/answer/6362090?visit_id=638683853091405069-3198009601&p=bluetooth&rd=1
IF THE DEVICE HAS BEEN PAIRED TO ANOTHER COMPUTER/PHONE BEFORE, THEN YOU MUST REMOVE THE BLE BOND FIRST.
`;


export const AT3_LIMITATIONS = `
This tool supports both Asset Tracker v2 (AT2) and Asset Tracker v3 (AT3).
However AT3 has limited support:
- Firmware Update is not supported!
- It is not possible to create BLE Bond to more than one device
  (You need to remove the current bond before creating a new one.)
`;


export const API_ALERT = `
ALERT!:
Web Bluetooth API and Filesystem API are not available
Please make sure the "Experimental Web Platform features" flag is enabled.
These APIs are available on the latest Chrome and Edge browsers.
`;


export const CONFIG_FLAGS = 13;
export const BLE_CLI_ACTIVE = 245;

export const BLE_CNX_ADV_DURATION_AT3 = 0x0b01;

export const PARAM_TYPE_DEPRECATED = 1;
export const PARAM_TYPE_INTEGER = 1;
export const PARAM_TYPE_FLOAT = 2;
export const PARAM_TYPE_STRING = 3;
export const PARAM_TYPE_BYTEARRAY = 4;

export const FW_TYPE_NONE = 0;
export const FW_TYPE_MCU = 1;
export const FW_TYPE_BLE_STACK = 2;

export const PARAMS = [
    ["ul_period", 0],
    ["lora_period", 1],
    ["pw_stat_period", 2],
    ["periodic_pos_period", 3],
    // ["unknown", 4],
    ["geoloc_sensor", 5],
    ["geoloc_method", 6],
    ["antenna", 7],
    ["motion_nb_pos", 8],
    ["gps_timeout", 9],
    ["agps_timeout", 10],
    ["gps_ehpe", 11],
    ["gps_convergence", 12],
    ["config_flags", 13],
    ["transmit_strat", 14],
    ["ble_beacon_cnt", 15],
    ["ble_beacon_timeout", 16],
    ["gps_standby_timeout", 17],
    ["confirmed_ul_bitmap", 18],
    ["confirmed_ul_retry", 19],
    ["motion_sensitivity", 20],
    ["shock_detection", 21],
    ["periodic_activity_period", 22],
    ["motion_duration", 23],
    ["geofencing_scan_period", 24],
    ["geofencing_collect_period", 25],
    ["ble_rssi_filter", 26],
    ["temperature_high", 27],
    ["temperature_low", 28],
    ["temperature_action", 29],
    ["transmit_strat_custom", 30],
    ["network_timeout_check", 31],
    ["network_timeout_reset", 32],
    ["collection_scan_type", 33],
    ["collection_nb_entry", 34],
    ["collection_ble_filter_type", 35],
    ["collection_ble_filter_main_1", 36],
    ["collection_ble_filter_main_2", 37],
    ["collection_ble_filter_sec_value", 38],
    ["collection_ble_filter_sec_mask", 39],
    ["battery_capacity", 40],
    ["reed_switch_configuration", 41],
    ["gnss_constellation", 42],
    ["prox_scan_pwr_min", 43],
    ["prox_distance_coef", 44],
    ["prox_scan_frequency", 45],
    ["prox_backtrace_max_age", 46],
    ["prox_distance_sliding_window", 47],
    ["prox_exposure_50", 48],
    ["prox_exposure_100", 49],
    ["prox_exposure_150", 50],
    ["prox_exposure_200", 51],
    ["prox_exposure_250", 52],
    ["prox_exposure_300", 53],
    ["prox_exposure_400", 54],
    ["prox_alarm_dist_immediate", 55],
    ["prox_alarm_exposure", 56],
    ["prox_warn_dist_immediate", 57],
    ["prox_warn_exposure", 58],
    ["prox_record_dist_immediate", 59],
    ["prox_record_exposure", 60],
    ["prox_alarm_buz_duration", 61],
    ["prox_warn_buz_duration", 62],
    ["prox_contact_policy", 63],
    ["prox_scan_duration", 64],
    ["prox_scan_window", 65],
    ["prox_scan_interval", 66],
    ["prox_alarm_remanence", 67],
    ["prox_warn_remanence", 68],
    ["prox_bcn_repeat", 69],
    ["prox_bcn_tx_power", 70],
    ["prox_reminder_period", 71],
    ["prox_reminder_distance", 72],
    ["prox_warn_disable_dist", 73],
    ["prox_alarm_disable_dist", 74],
    ["prox_max_speed_filter", 75],
    ["prox_max_update", 76],
    ["position_ble_filter_type", 77],
    ["position_ble_filter_main_1", 78],
    ["position_ble_filter_main_2", 79],
    ["position_ble_filter_sec_value", 80],
    ["position_ble_filter_sec_mask", 81],
    ["position_ble_report_type", 82],
    ["buzzer_volume", 83],
    ["angle_detect_mode", 84],
    ["angle_ref_acq", 85],
    ["angle_ref_acc_x", 86],
    ["angle_ref_acc_y", 87],
    ["angle_ref_acc_z", 88],
    ["angle_critical", 89],
    ["angle_critical_hyst", 90],
    ["angle_report_mode", 91],
    ["angle_report_period", 92],
    ["angle_report_repeat", 93],
    ["angle_rising_time", 94],
    ["angle_falling_time", 95],
    ["angle_learning_time", 96],
    ["angle_acc_accuracy", 97],
    ["angle_deviation_delta", 98],
    ["angle_deviation_min_interval", 99],
    ["angle_deviation_max_interval", 100],
    ["default_profile", 101],
    ["password", 102],
    ["gps_t0_timeout", 103],
    ["gps_fix_timeout", 104],
    ["geofencing_scan_duration", 105],
    ["beaconing_type", 106],
    ["beaconing_tx_power", 107],
    ["beaconing_static_interval", 108],
    ["beaconing_motion_interval", 109],
    ["beaconing_motion_duration", 110],
    ["ble_cnx_adv_duration", 111],
    ["beacon_id_0", 112],
    ["beacon_id_1", 113],
    ["beacon_id_2", 114],
    ["beacon_id_3", 115],
    ["beacon_id_4", 116],
    ["sos_period", 117],
    ["motion_debounce", 118],
    ["button_mapping", 119],
    ["default_datarate", 120],
    ["gps_ehpe_motion", 121],
    ["gps_convergence_motion", 122],
    ["gps_t0_timeout_motion", 123],
    // ["ble_cli_active", 245],
    // ["profile", 246],
    // ["consumption", 247],
    // ["ble_bond_info", 248],
    ["mode", 249],
    // ["acc_x_axis", 250],
    // ["acc_y_axis", 251],
    // ["acc_z_axis", 252],
    // ["ble_version", 253],
    // ["firmware_version", 254],
];


export const PARAMS_AT3 = [
    ["sys_highest_temperature", 0x0000,"i32"],
    ["sys_lowest_temperature", 0x0001,"i32"],
    ["sys_power_consumption", 0x0002,"i32"],

    ["core_monitoring_period", 0x0100,"i32"],
    ["core_status_period", 0x0101,"i32"],
    ["core_notif_enable", 0x0102,"array"],
    ["core_temp_high_threshold", 0x0103,"i32"],
    ["core_temp_low_threshold", 0x0104,"i32"],
    ["core_temp_hysteresis", 0x0105,"i32"],
    ["core_button1_map", 0x0106,"i32"],
    ["core_button2_map", 0x0107,"i32"],
    ["core_buttons_timing", 0x0108,"i32"],
    ["core_led0_map", 0x0109,"array"],
    ["core_led1_map", 0x010a,"array"],
    ["core_buzzer_map", 0x010b,"array"],
    ["core_almanac_validity", 0x010c,"i32"],
    ["core_almanac_outdated_ratio", 0x010d,"i32"],
    ["core_cli_password", 0x010e,"i32"],
    ["core_db_type_mask", 0x010f, "array"],

    ["geoloc_motion_period", 0x0200,"i32"],
    ["geoloc_static_period", 0x0201,"i32"],
    ["geoloc_sos_period", 0x0202,"i32"],
    ["geoloc_motion_nb_start", 0x0203,"i32"],
    ["geoloc_motion_nb_stop", 0x0204,"i32"],
    ["geoloc_start_stop_period", 0x0205,"i32"],
    ["geoloc_gnss_hold_on_mode", 0x0206,"i32"],
    ["geoloc_gnss_hold_on_timeout", 0x0207,"i32"],
    ["geoloc_profile0_triggers", 0x0208,"i32"],
    ["geoloc_profile1_triggers", 0x0209,"i32"],
    ["geoloc_profile2_triggers", 0x020a,"i32"],
    ["geoloc_gbe_profile0_techno", 0x020b,"array"],
    ["geoloc_gbe_profile1_techno", 0x020c,"array"],
    ["geoloc_gbe_profile2_techno", 0x020d,"array"],

    ["gnss_constellation", 0x0300,"i32"],
    ["gnss_max_time", 0x0301,"i32"],
    ["gnss_t0_timeout_static", 0x0302,"i32"],
    ["gnss_ehpe_static", 0x0303,"i32"],
    ["gnss_convergence_static", 0x0304,"i32"],
    ["gnss_t0_timeout_motion", 0x0305,"i32"],
    ["gnss_ehpe_motion", 0x0306,"i32"],
    ["gnss_convergence_motion", 0x0307,"i32"],
    ["gnss_standby_time", 0x0308,"i32"],
    ["gnss_agnss_max_time", 0x0309,"i32"],
    ["gnss_t1_timeout", 0x030a,"i32"],

    ["lr_constellation", 0x0400,"i32"],
    ["lr_scan_mode", 0x0401,"i32"],
    ["lr_nb_scan", 0x0402,"i32"],
    ["lr_inter_scan_time", 0x0403,"i32"],
    ["lr_wifi_report_nb_bssid", 0x0404,"i32"],
    ["lr_wifi_min_nb_bssid", 0x0405,"i32"],
    ["lr_wifi_min_rssi", 0x0406,"i32"],
    ["lr_wifi_bssid_mac_type", 0x0407,"i32"],

    ["ble_scan1_duration", 0x0500,"i32"],
    ["ble_scan1_window", 0x0501,"i32"],
    ["ble_scan1_interval", 0x0502,"i32"],
    ["ble_scan1_type", 0x0503,"i32"],
    ["ble_scan1_min_rssi", 0x0504,"i32"],
    ["ble_scan1_min_nb_beacons", 0x0505,"i32"],
    ["ble_scan1_filter1_mask", 0x0506,"array"],
    ["ble_scan1_filter1_value", 0x0507,"array"],
    ["ble_scan1_filter1_offset", 0x0508,"i32"],
    ["ble_scan1_filter2_mask", 0x0509,"array"],
    ["ble_scan1_filter2_value", 0x050a,"array"],
    ["ble_scan1_filter2_offset", 0x050b,"i32"],
    ["ble_scan1_nb_beacons", 0x050c,"i32"],
    ["ble_scan1_report_type", 0x050d,"i32"],
    ["ble_scan1_report_id_ofs", 0x050e,"i32"],

    ["ble_scan2_duration", 0x0600,"i32"],
    ["ble_scan2_window", 0x0601,"i32"],
    ["ble_scan2_interval", 0x0602,"i32"],
    ["ble_scan2_type", 0x0603,"i32"],
    ["ble_scan2_min_rssi", 0x0604,"i32"],
    ["ble_scan2_min_nb_beacons", 0x0605,"i32"],
    ["ble_scan2_filter1_mask", 0x0606,"array"],
    ["ble_scan2_filter1_value", 0x0607,"array"],
    ["ble_scan2_filter1_offset", 0x0608,"i32"],
    ["ble_scan2_filter2_mask", 0x0609,"array"],
    ["ble_scan2_filter2_value", 0x060a,"array"],
    ["ble_scan2_filter2_offset", 0x060b,"i32"],
    ["ble_scan2_nb_beacons", 0x060c,"i32"],
    ["ble_scan2_report_type", 0x060d,"i32"],
    ["ble_scan2_report_id_ofs", 0x060e,"i32"],

    ["accelero_motion_sensi", 0x0700,"i32"],
    ["accelero_motion_duration", 0x0701,"i32"],
    ["accelero_full_scale", 0x0702,"i32"],
    ["accelero_output_data_rate", 0x0703,"i32"],
    ["accelero_shock_threshold", 0x0704,"i32"],

    ["net_selection", 0x0800, "i32"],
    ["net_reconnection_spacing_static", 0x0801, "i32"],
    ["net_main_probe_timeout_static", 0x0802, "i32"],
    ["net_reconnection_spacing_motion", 0x0803, "i32"],
    ["net_main_probe_timeout_motion", 0x0804, "i32"],

    ["lorawan_cnx_timeout", 0x0900, "i32"],
    ["lorawan_heartbeat_period", 0x0901, "i32"],
    ["lorawan_probe_max_attempts_static", 0x0902, "i32"],
    ["lorawan_probe_period_static", 0x0903, "i32"],
    ["lorawan_confirm_notif_map", 0x0904, "array"],
    ["lorawan_confirm_notif_retry", 0x0905, "i32"],
    ["lorawan_s1_tx_strategy", 0x0906, "i32"],
    ["lorawan_s1_ul_port", 0x0907, "i32"],
    ["lorawan_s1_dl_port", 0x0908, "i32"],
    ["lorawan_probe_period_motion", 0x0909, "i32"],
    ["lorawan_probe_max_attempts_motion", 0x090a, "i32"],

    ["cell_sim_interface", 0x0a00,"i32"],
    ["cell_network_type", 0x0a01,"i32"],
    ["cell_search_bands", 0x0a02,"array"],
    ["cell_cnx_timeout_static", 0x0a03,"i32"],
    ["cell_cnx_timeout_motion", 0x0a04,"i32"],
    ["cell_cnx_nw_reconnect_timeout", 0x0a05,"i32"],
    ["cell_cnx_max_attempts", 0x0a06,"i32"],
    ["cell_access_point_name", 0x0a07,"string"],
    ["cell_operator_sim_slot_0", 0x0a08,"string"],
    ["cell_operator_sim_slot_1", 0x0a09,"string"],
    ["cell_low_power_mode", 0x0a0a,"i32"],
    ["cell_psm_tau_period", 0x0a0b,"i32"],
    ["cell_psm_active_time", 0x0a0c,"i32"],
    ["cell_edrx_pcl", 0x0a0d,"i32"],
    ["cell_edrx_ptw", 0x0a0e,"i32"],
    ["cell_rai_timeout", 0x0a0f,"i32"],
    ["cell_max_probe_attempts", 0x0a10,"i32"],
    ["cell_probe_period", 0x0a11,"i32"],
    ["cell_s1_transport_proto", 0x0a12,"i32"],
    ["cell_s1_ip_url_addr", 0x0a13,"string"],
    ["cell_s1_dst_ip_port", 0x0a14,"i32"],
    ["cell_s1_src_ip_port", 0x0a15,"i32"],
    ["cell_s1_tx_aggr_time", 0x0a16,"i32"],
    ["cell_apn_user_id", 0x0a17, "string"],     
    ["cell_apn_user_pwd", 0x0a18, "string"],
    ["cell_apn_auth_protocol", 0x0a19, "i32"],
    
    ["ble_cnx_tx_power", 0x0b00,"i32"],
    ["ble_cnx_adv_duration", 0x0b01,"i32"],
    ["ble_cnx_behavior", 0x0b02,"i32"],
    ["ble_beacon_tx_power", 0x0b03,"i32"],
    ["ble_beacon_type", 0x0b04,"i32"],
    ["ble_beacon_identifier", 0x0b05,"array"],
    ["ble_beacon_fast_adv_interval", 0x0b06,"i32"],
    ["ble_beacon_slow_adv_interval", 0x0b07,"i32"],

    
    ["nmeta_dataperiod"         , 0x0c00, "i32"],
    ["sensor0_meas_nsampling"   , 0x0c01, "i32"],
    ["sensor0_high_threshold"   , 0x0c02, "i32"],
    ["sensor0_low_threshold"    , 0x0c03, "i32"],
    ["sensor0_hysteresis"       , 0x0c04, "i32"],
    ["sensor0_meas_nmaxinterval", 0x0c05, "i32"],
    ["sensor0_telem_maxinterval", 0x0c06, "i32"],
    ["sensor0_cyclic_version"   , 0x0c07, "i32"],
    ["sensor1_meas_nsampling"   , 0x0c08, "i32"],
    ["sensor1_high_threshold"   , 0x0c09, "i32"],
    ["sensor1_low_threshold"    , 0x0c0a, "i32"],
    ["sensor1_hysteresis"       , 0x0c0b, "i32"],
    ["sensor1_meas_nmaxinterval", 0x0c0c, "i32"],
    ["sensor1_telem_maxinterval", 0x0c0d, "i32"],
    ["sensor1_cyclic_version"   , 0x0c0e, "i32"],
    ["sensor2_meas_nsampling"   , 0x0c0f, "i32"],
    ["sensor2_high_threshold"   , 0x0c10, "i32"],
    ["sensor2_low_threshold"    , 0x0c11, "i32"],
    ["sensor2_hysteresis"       , 0x0c12, "i32"],
    ["sensor2_meas_nmaxinterval", 0x0c13, "i32"],
    ["sensor2_telem_maxinterval", 0x0c14, "i32"],
    ["sensor2_cyclic_version"   , 0x0c15, "i32"],
    ["sensor3_meas_nsampling"   , 0x0c16, "i32"],
    ["sensor3_high_threshold"   , 0x0c17, "i32"],
    ["sensor3_low_threshold"    , 0x0c18, "i32"],
    ["sensor3_hysteresis"       , 0x0c19, "i32"],
    ["sensor3_meas_nmaxinterval", 0x0c1a, "i32"],
    ["sensor3_telem_maxinterval", 0x0c1b, "i32"],
    ["sensor3_cyclic_version"   , 0x0c1c, "i32"],
];
