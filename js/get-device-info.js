import * as abw from './abw.js';
import { log, setBLESpeed } from './main.js';


export async function onGetBluetoothDeviceInfoButtonClick() {

    try {

        loader_div.style.display = 'block';
        log(`Getting Bluetooth Device Information from device ${gblDevEUIHex}...`);

        await setBLESpeed(abw.WR_VERY_FAST_CONN);

        const decoder = new TextDecoder('utf-8');

        const chr = abw.services.device_information.chars;
        const model_number = await chr.model_number.obj.readValue();
        // const serial_number = await chr.serial_number.obj.readValue();
        const firmware_revision = await chr.firmware_revision.obj.readValue();
        const software_revision = await chr.software_revision.obj.readValue();
        const manufacturer_name = await chr.manufacturer_name.obj.readValue();
        log('> Model Number: ' + decoder.decode(model_number));
        // log('> Serial Number: ' + decoder.decode(serial_number));
        // console.log(serial_number);
        log('> Firmware Revision: ' + decoder.decode(firmware_revision));
        log('> Software Revision: ' + decoder.decode(software_revision));
        log('> Manufacturer Name: ' + decoder.decode(manufacturer_name));


        log(`Getting TX Power from device ${gblDevEUIHex}...`);
        const chr_tx_power_level = abw.services.tx_power.chars.tx_power_level.obj;
        const tx_power_level = await chr_tx_power_level.readValue();
        log('> TX Power Level: ' + tx_power_level.getUint8(0));

        
        log(`Getting Battery Information from device ${gblDevEUIHex}...`);
        const chr_battery_level = abw.services.battery.chars.battery_level.obj;
        const battery_level = await chr_battery_level.readValue();
        const chr_battery_power_state = abw.services.battery.chars.battery_power_state.obj;
        const battery_power_state = await chr_battery_power_state.readValue();
        log('> Battery Level: ' + battery_level.getUint8(0) + '%');
        const bps = battery_power_state.getUint8(0);
        switch (bps) {
            case abw.CHARGER_PRESENT_AND_CHARGING: 
                log('> Battery Power State: Charger present and charging.');
                break;
            case abw.CHARGER_PRESENT_BUT_NOT_CHARGING:
                log('> Battery Power State: Charger present but not charging.');
                break;
            case abw.CHARGER_NOT_PRESENT_AND_DISCHARGING: 
                log('> Battery Power State: Charger not present and discharging.');
                break;
            default:
                log('> Battery Power State: Unknown battery power state: ' + bps);
        };

        log(`Getting Environmental Sensing Data from device ${gblDevEUIHex}...`);
        const chr_temperature_celsius = abw.services.environmental_sensing.chars.temperature_celsius.obj;
        const temperature_celsius = await chr_temperature_celsius.readValue();
        const t = ((temperature_celsius.getUint8(0)<<0) + (temperature_celsius.getUint8(1)<<8)) / 10;
        log(`> Temperature: ${t} C`);
        // await setBLESpeed(abw.WR_FAST_CONN);
        loader_div.style.display = 'none';
    
    } catch(error) {
        // console.log(error);
        log('Argh! ' + error);
        if (error == 'NetworkError: Authentication failed.') {
            log(abw.CONNECTION_ALERT);
        }
        // await setBLESpeed(abw.WR_FAST_CONN);
        loader_div.style.display = 'none';
    }

}
