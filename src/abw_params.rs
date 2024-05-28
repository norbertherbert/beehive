use std::collections::BTreeMap;
use std::sync::OnceLock;
// use std::sync::LazyLock;

pub const PARAMS: [(&str, u8); 125] = [
    ("ul_period", 0),
    ("lora_period", 1),
    ("pw_stat_period", 2),
    ("periodic_pos_period", 3),
    ("unknown", 4),
    ("geoloc_sensor", 5),
    ("geoloc_method", 6),
    ("antenna", 7),
    ("motion_nb_pos", 8),
    ("gps_timeout", 9),
    ("agps_timeout", 10),
    ("gps_ehpe", 11),
    ("gps_convergence", 12),
    ("config_flags", 13),
    ("transmit_strat", 14),
    ("ble_beacon_cnt", 15),
    ("ble_beacon_timeout", 16),
    ("gps_standby_timeout", 17),
    ("confirmed_ul_bitmap", 18),
    ("confirmed_ul_retry", 19),
    ("motion_sensitivity", 20),
    ("shock_detection", 21),
    ("periodic_activity_period", 22),
    ("motion_duration", 23),
    ("geofencing_scan_period", 24),
    ("geofencing_collect_period", 25),
    ("ble_rssi_filter", 26),
    ("temperature_high", 27),
    ("temperature_low", 28),
    ("temperature_action", 29),
    ("transmit_strat_custom", 30),
    ("network_timeout_check", 31),
    ("network_timeout_reset", 32),
    ("collection_scan_type", 33),
    ("collection_nb_entry", 34),
    ("collection_ble_filter_type", 35),
    ("collection_ble_filter_main_1", 36),
    ("collection_ble_filter_main_2", 37),
    ("collection_ble_filter_sec_val", 38),
    ("collection_ble_filter_sec_mas", 39),
    ("battery_capacity", 40),
    ("reed_switch_configuration", 41),
    ("gnss_constellation", 42),
    ("prox_scan_pwr_min", 43),
    ("prox_distance_coef", 44),
    ("prox_scan_frequency", 45),
    ("prox_backtrace_max_age", 46),
    ("prox_distance_sliding_window", 47),
    ("prox_exposure_50", 48),
    ("prox_exposure_100", 49),
    ("prox_exposure_150", 50),
    ("prox_exposure_200", 51),
    ("prox_exposure_250", 52),
    ("prox_exposure_300", 53),
    ("prox_exposure_400", 54),
    ("prox_alarm_dist_immediate", 55),
    ("prox_alarm_exposure", 56),
    ("prox_warn_dist_immediate", 57),
    ("prox_warn_exposure", 58),
    ("prox_record_dist_immediate", 59),
    ("prox_record_exposure", 60),
    ("prox_alarm_buz_duration", 61),
    ("prox_warn_buz_duration", 62),
    ("prox_contact_policy", 63),
    ("prox_scan_duration", 64),
    ("prox_scan_window", 65),
    ("prox_scan_interval", 66),
    ("prox_alarm_remanence", 67),
    ("prox_warn_remanence", 68),
    ("prox_bcn_repeat", 69),
    ("prox_bcn_tx_power", 70),
    ("prox_reminder_period", 71),
    ("prox_reminder_distance", 72),
    ("prox_warn_disable_dist", 73),
    ("prox_alarm_disable_dist", 74),
    ("prox_max_speed_filter", 75),
    ("prox_max_update", 76),
    ("position_ble_filter_type", 77),
    ("position_ble_filter_main_1", 78),
    ("position_ble_filter_main_2", 79),
    ("position_ble_filter_sec_value", 80),
    ("position_ble_filter_sec_mask", 81),
    ("position_ble_report_type", 82),
    ("buzzer_volume", 83),
    ("angle_detect_mode", 84),
    ("angle_ref_acq", 85),
    ("angle_ref_acc_x", 86),
    ("angle_ref_acc_y", 87),
    ("angle_ref_acc_z", 88),
    ("angle_critical", 89),
    ("angle_critical_hyst", 90),
    ("angle_report_mode", 91),
    ("angle_report_period", 92),
    ("angle_report_repeat", 93),
    ("angle_rising_time", 94),
    ("angle_falling_time", 95),
    ("angle_learning_time", 96),
    ("angle_acc_accuracy", 97),
    ("angle_deviation_delta", 98),
    ("angle_deviation_min_interval", 99),
    ("angle_deviation_max_interval", 100),
    ("default_profile", 101),
    ("password", 102),
    ("gps_t0_timeout", 103),
    ("gps_fix_timeout", 104),
    ("geofencing_scan_duration", 105),
    ("beaconing_type", 106),
    ("beaconing_tx_power", 107),
    ("beaconing_static_interval", 108),
    ("beaconing_motion_interval", 109),
    ("beaconing_motion_duration", 110),
    ("ble_cnx_adv_duration", 111),
    ("beacon_id_0", 112),
    ("beacon_id_1", 113),
    ("beacon_id_2", 114),
    ("beacon_id_3", 115),
    ("beacon_id_4", 116),
    ("sos_period", 117),
    ("motion_debounce", 118),
    ("button_mapping", 119),
    ("default_datarate", 120),
    ("gps_ehpe_motion", 121),
    ("gps_convergence_motion", 122),
    ("gps_t0_timeout_motion", 123),
    // ("ble_cli_active", 245),
    // ("profile", 246),
    // ("consumption", 247),
    // ("ble_bond_info", 248),
    ("mode", 249),
    // ("acc_x_axis", 250),
    // ("acc_y_axis", 251),
    // ("acc_z_axis", 252),
    // ("ble_version", 253),
    // ("firmware_version", 254),
];

static PARAM_NAME_TO_ID: OnceLock<BTreeMap<&str, u8>> = OnceLock::new();
static PARAM_ID_TO_NAME: OnceLock<BTreeMap<u8, &str>> = OnceLock::new();

pub const UL_PERIOD :u8 = 0;
pub const CONFIG_FLAGS :u8 = 13;
pub const BLE_CLI_ACTIVE :u8 = 245;


pub fn get_param_name_to_id() -> &'static BTreeMap<&'static str, u8> {
    PARAM_NAME_TO_ID.get_or_init(|| BTreeMap::from(PARAMS))   
}

pub fn get_param_id_to_name() -> &'static BTreeMap<u8, &'static str> {
    PARAM_ID_TO_NAME.get_or_init(|| {
        let mut map: BTreeMap<u8, &str> = BTreeMap::new();
        for item in PARAMS {
            map.insert(item.1, item.0);
        };
        map
    })
}


// pub static PARAM_NAME_TO_ID_LAZY: LazyLock<BTreeMap<&str, u8>> = LazyLock::new(||{
//     BTreeMap::from(PARAMS_ARRAY)
// });
// pub static PARAM_ID_TO_NAME_LAZY: LazyLock<BTreeMap<u8, &str>> = LazyLock::new(||{
//     let mut map: BTreeMap<u8, &str> = BTreeMap::new();
//     for item in PARAMS_ARRAY {
//         map.insert(item.1, item.0);
//     };
//     map
// });
