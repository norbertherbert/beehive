import * as abw from './abw.js';
import {log, streamLog, setBLESpeed, createStreamFromEvents} from './connection-mgmt.js';
import { onUsbCliCmdKeypressEvent, onUsbStopCliButtonClick } from './usb-cli.js';


export const cmdHistory = [];
export let cmdIndex = 0;
export function onArrowUpDn(kbdEvent, inputElement) {
    switch (kbdEvent.code) {
        case 'ArrowUp':
            if (cmdIndex > 0) { 
                cmdIndex -= 1;
                setTimeout(() => {inputElement.value = cmdHistory[cmdIndex]}, 50);
            } 
            break;
        case 'ArrowDown':
            if (cmdIndex < cmdHistory.length) {
                cmdIndex += 1;
                if (cmdIndex == cmdHistory.length) {
                    inputElement.value = '';
                } else {
                    inputElement.value = cmdHistory[cmdIndex];
                }
            }
            break;
        default:
    }
}


export function addToCmdHistory(cmd) {
    if (
        (cmd != '') && 
        (cmd != cmdHistory[cmdHistory.length-1]) && 
        (cmd != '123')    // TODO: Find a better way to detect if it is a password!
    ) {
        cmdHistory.push(cmd);   
    }
    cmdIndex = cmdHistory.length;
}

export async function onBleCliCmdKeypressEvent(e) {
    if (e.key !== 'Enter') { return }
    try {

        let cmd = command_input.value;
        command_input.value = "";
        addToCmdHistory(cmd);
        log(cmd);

        const encoder = new TextEncoder("ascii");
        const chr_custom_send_cli_cmd = abw.services.abeeway_primary.chars.custom_send_cli_cmd.obj;
        while (cmd.length > 19) {
            const cmdChunk = cmd.substring(0,19);
            cmd = cmd.substring(19);
            await chr_custom_send_cli_cmd.writeValueWithoutResponse(encoder.encode(cmdChunk));
        }
        await chr_custom_send_cli_cmd.writeValueWithoutResponse(encoder.encode(cmd + "\r\n"));

    }
    catch(error) {
        log('Argh! ' + error);
    }
}


function onCliCommandResponse(event) {
    
    let val = new TextDecoder("ascii").decode(event.target.value);
    streamLog(val);

}


export async function onStartCliButtonClick() {

    try {

        loader_div.style.display = 'block';
        log(`Starting the CLI...`);

        await setBLESpeed(abw.WR_VERY_FAST_CONN);

        const chr_configuration = abw.services.abeeway_primary.chars.configuration.obj;
        const chr_custom_send_cli_cmd = abw.services.abeeway_primary.chars.custom_send_cli_cmd.obj;
        const chr_custom_rcv_serial_data = abw.services.abeeway_primary.chars.custom_rcv_serial_data.obj;

        const confEventStream = createStreamFromEvents(chr_configuration, 'characteristicvaluechanged');
        const confEventReader = confEventStream.getReader();
        await chr_configuration.startNotifications();
        log(`> Configuration notifications have been started`);

        let res;

        if (gblIsAT2) {

            // Enable BLE CLI in config_flags

            await chr_configuration.writeValueWithoutResponse(Uint8Array.of(
                abw.WR_READ_CONF, 
                abw.CONFIG_FLAGS,
            ));
            res = await confEventReader.read();
            if ((res.value.getUint8(3) & 1<<4)==0) {

                await chr_configuration.writeValueWithoutResponse(Uint8Array.of(
                    abw.WR_WRITE_CONF, 
                    abw.CONFIG_FLAGS,
                    res.value.getUint8(2), 
                    res.value.getUint8(3) | 1<<4, 
                    res.value.getUint8(4), 
                    res.value.getUint8(5)
                ));

                log("> BLE CLI (bit 20) has been enabled in config_flags.");

            } else {
                log("> BLE CLI (bit 20) is already enabled in config_flags.");
            }

            // Turn on BLE CLI

            await chr_configuration.writeValueWithoutResponse(Uint8Array.of(
                abw.WR_WRITE_CONF, 
                abw.BLE_CLI_ACTIVE,
                0, 0, 0, 1,
            ));

            log("> BLE CLI has been turned on.");

        } else {

            const chr_custom_cmd = abw.services.abeeway_primary.chars.custom_cmd.obj;
            await chr_custom_cmd.writeValue(Uint8Array.of(abw.WR_ENABLE_BLE_CLI));
            log("> BLE CLI has been turned on.");

        }
        
        await chr_custom_rcv_serial_data.startNotifications();
        log(`> Serial Data notifications have been started`);

        chr_custom_rcv_serial_data.addEventListener('characteristicvaluechanged', onCliCommandResponse);

        // These two lines are needed as a workaround to show the login prompt at start
        await new Promise(r => setTimeout(r, 300));
        await chr_custom_send_cli_cmd.writeValueWithoutResponse(Uint8Array.of(
            13, // '\r'.charCodeAt(0) & 0xff,
            10, // '\n'.charCodeAt(0) & 0xff,
        ));

        log("\r\n".repeat(80));
        bluetooth_services_div.style.display = "none";
        refresh_connection_status_button.style.display = "none";
        command_input.style.display = "block";
        command_input.value = "";
        command_input.focus();
        
   
        try {
            command_input.removeEventListener('keypress', onUsbCliCmdKeypressEvent);
        } catch(e) {}
        command_input.addEventListener('keypress', onBleCliCmdKeypressEvent);

        stop_cli_button.style.display = "block";
        try {
            stop_cli_button.removeEventListener('click', onUsbStopCliButtonClick);
        } catch(e) {}
        stop_cli_button.addEventListener('click', onStopCliButtonClick);

        loader_div.style.display = 'none';

    } catch(error) {
        log('Argh! ' + error);
        loader_div.style.display = 'none';
    }

}

export async function onStopCliButtonClick() {

    try {

        loader_div.style.display = 'block';
        log("\r\n".repeat(3) + "Stopping the CLI...");

        const chr_configuration = abw.services.abeeway_primary.chars.configuration.obj;
        const chr_custom_rcv_serial_data = abw.services.abeeway_primary.chars.custom_rcv_serial_data.obj;

        await chr_custom_rcv_serial_data.removeEventListener('characteristicvaluechanged', onCliCommandResponse);
        await chr_custom_rcv_serial_data.stopNotifications();
        log(`> Serial Data notifications have been stopped`);


        if (gblIsAT2) {

            await chr_configuration.writeValueWithoutResponse(Uint8Array.of(
                abw.WR_WRITE_CONF, 
                abw.BLE_CLI_ACTIVE,
                0, 0, 0, 0,
            ));

            log("> BLE CLI has been turned off.");

        } else {

            const chr_custom_cmd = abw.services.abeeway_primary.chars.custom_cmd.obj;
            await chr_custom_cmd.writeValue(Uint8Array.of(abw.WR_DISABLE_BLE_CLI));
            log("> BLE CLI has been turned off.");

        }

        
        await chr_configuration.stopNotifications()
        log(`> Configuration notifications have been stopped`);

        bluetooth_services_div.style.display = "block";
        refresh_connection_status_button.style.display = "inline-block";

        command_input.style.display = "none";
        command_input.value = "";
        command_input.removeEventListener('keypress', onBleCliCmdKeypressEvent);
        stop_cli_button.style.display = "none";
        stop_cli_button.removeEventListener('click', onStopCliButtonClick);

        log(`> CLI has been closed`);
        loader_div.style.display = 'none';

        await setBLESpeed(abw.WR_VERY_FAST_CONN);

    } catch(error) {
        log('Argh! ' + error);
        loader_div.style.display = 'none';
    }

}