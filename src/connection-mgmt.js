import * as abw from './abw.js';
import { onStopCliButtonClick } from './cli.js';


export function log(line) {
    log_div.append(line + '\n');
    log_div.scrollTop = log_div.scrollHeight;
}

export function streamLog(text) {
    log_div.append(text);
    log_div.scrollTop = log_div.scrollHeight;
}

// function log(message) {
//     const messageSpan = document.createElement('span');
//     messageSpan.textContent = message;
//     log_div.appendChild(messageSpan);
// }


export function clearLog() {
    log_div.textContent = '';
}

export async function sleep(ms) { return new Promise(resolve => setTimeout(resolve, ms)) };


export async function onRequestBluetoothDeviceButtonClick() {
    try {

        loader_div.style.display = 'block';

        log('Requesting permission for the browser to access a Bluetooth device...');

        const options = {
            filters: [
                { namePrefix: 'AB' },
            ],
            optionalServices: [
                abw.services.abeeway_primary.uuid,
                abw.services.device_information.uuid,
                abw.services.tx_power.uuid,
                abw.services.battery.uuid,
                abw.services.environmental_sensing.uuid,
                abw.services.environmental_sensing.uuid,
                abw.services.immediate_alert.uuid,
            ],
        };

        gblDevice = await navigator.bluetooth.requestDevice(options);

        gblDevice.addEventListener('gattserverdisconnected', onDisconnected);

        device_name_span.textContent = gblDevice.name;
        connection_status_span.textContent = '- Connecting...';

        forget_bluetooth_device_button.style.display = 'inline-block';
        connect_bluetooth_device_button.style.display = 'none';
        disconnect_bluetooth_device_button.style.display = 'none';
        refresh_connection_status_button.style.display = 'none';
    
        bluetooth_services_div.style.display = 'none';
        command_input.style.display = "none";
        stop_cli_button.style.display = "none";

        request_bluetooth_device_button.style.display = "none";
        usb_connect_device_button.style.display = "none";
        usb_disconnect_device_button.style.display = "none";
        // usb_import_config_button.style.display = "none";
        // usb_export_config_button.style.display = "none";
        
       

        log(`> New device has been added to the permitted device list of your browser:  ${gblDevice.name}`);

        await onConnectBluetoothDeviceButtonClick();

        loader_div.style.display = 'none';

    }
    catch(error) {
        log('Argh! ' + error);
        loader_div.style.display = 'none';
    }
}

export async function onForgetBluetoothDeviceButtonClick() {
    try {

        loader_div.style.display = 'block';
        log(`Forgetting ${gblDevice.name} Bluetooth device...`);

        await onDisconnectBluetoothDeviceButtonClick()
        
        await gblDevice.forget();

        gblDevice = null;
        gblIsAT2 = false;
        gblDevEUIHex = '';
        device_name_span.textContent = '';

        forget_bluetooth_device_button.style.display = 'none';
        connect_bluetooth_device_button.style.display = 'none';
        disconnect_bluetooth_device_button.style.display = 'none';
        refresh_connection_status_button.style.display = 'none';

        request_bluetooth_device_button.style.display = "inline-block";
        usb_connect_device_button.style.display = "inline-block";

        log('> Bluetooth device has been forgotten.');
        loader_div.style.display = 'none';

    }
    catch(error) {
        log('Argh! ' + error);
        loader_div.style.display = 'none';
    }
}

export async function onConnectBluetoothDeviceButtonClick() {

    try {

        loader_div.style.display = 'block';
        connection_status_span.textContent = '- Connecting...';
        log(`Connecting device ${gblDevice.name}...`);
        log('...it may take a while');
        log('...you can speed-up the process by restarting the Smart Badge or applying the magnet on the Compact Tracker');
        let server = await gblDevice.gatt.connect();

        if (server.connected) {

            connection_status_span.textContent = '- Connected';
            log(`> Device ${gblDevice.name} has been connected`);

            forget_bluetooth_device_button.style.display = 'inline-block';
            connect_bluetooth_device_button.style.display = 'none';
            disconnect_bluetooth_device_button.style.display = 'inline-block';
            refresh_connection_status_button.style.display = 'inline-block';


            // Find services

            log(`Looking for services and characteristics on device ${gblDevice.name}...`);
            await abw.findServices(server);
            log('> All services and characteristics have been found');


            await setBLESpeed(abw.WR_VERY_FAST_CONN);


            // Detect HW Family

            if (gblDevice.name.substring(0,3) == 'ABW') {

                const decoder = new TextDecoder('utf-8');
                const chr = abw.services.device_information.chars;
                const software_revision = await chr.software_revision.obj.readValue();
                gblIsAT2 = decoder.decode(software_revision).substring(0, 1) == '2';
                log('> Detected Hardware Family: ' + (gblIsAT2 ? 'AT2' : 'AT3') );
                connection_status_span.textContent = `- Connected (${gblIsAT2 ? 'AT2' : 'AT3'})`;

                // gblIsAT2 = true;
                // log('> Detected Hardware Family: AT2');
                // connection_status_span.textContent = `- Connected (AT2)`;

            } else {
                gblIsAT2 = false;
                log('> Detected Hardware Family: AT3');
                connection_status_span.textContent = `- Connected (AT3)`;
            }

            // Detect DevEUI

            if (gblIsAT2) {
                gblDevEUIHex = abw.DEV_EUI_PREFIX_AT2 + gblDevice.name.substring(3);
            } else {
                gblDevEUIHex = abw.DEV_EUI_PREFIX_AT3 + gblDevice.name.substring(2);
            }
            log('> Detected DevEUI: ' + gblDevEUIHex);
            device_name_span.textContent = gblDevEUIHex;


            // Show BLE Detvices Buttons
        
            bluetooth_services_div.style.display = 'block';

            if (gblIsAT2) {

                // Chhange the BLE Advertisement duration to 1800s!
                
                const buffer = new ArrayBuffer(6);
                const view = new DataView(buffer);
                view.setUint8(0, abw.WR_WRITE_CONF);
                view.setUint8(1, 111);   // ble_cnx_adv_duration             
                view.setInt32(2, 1800);  // 30 min

                const chr_configuration = abw.services.abeeway_primary.chars.configuration.obj;

                await chr_configuration.writeValueWithoutResponse(buffer);
                log('> The BLE Advertisement duration has been changed to 1800s!')

            } 

            log(abw.CONNECTION_ALERT);

            if (!gblIsAT2)  {
                log(abw.AT3_LIMITATIONS);
            }
        
        } else {

            gblIsAT2 = false;
            gblDevEUIHex = '';

            device_name_span.textContent = gblDevice ? gblDevice.name : '';
            connection_status_span.textContent = '- Not Connected';

            forget_bluetooth_device_button.style.display = 'inline-block';
            connect_bluetooth_device_button.style.display = 'inline-block';
            disconnect_bluetooth_device_button.style.display = 'none';
            refresh_connection_status_button.style.display = 'inline-block';
        
            bluetooth_services_div.style.display = 'none';
            command_input.style.display = "none";
            stop_cli_button.style.display = "none";

            log(`> Device ${gblDevice.name} couldn't be connected`);

        }

        loader_div.style.display = 'none';

    }
    catch(error) {
        log('Argh! ' + error);
        log('Plese try to connect again!');
        loader_div.style.display = 'none';
    }

}

function onDisconnected(event) {

    if (gblGetTiltParamsInterval > 0) {
        clearInterval(gblGetTiltParamsInterval);
    }

    gblIsAT2 = false;
    gblDevEUIHex = '';

    device_name_span.textContent = gblDevice ? gblDevice.name : '';
    connection_status_span.textContent = '- Not Connected';

    log(`EVENT: Device ${gblDevice.name} has been disconnected`);

    forget_bluetooth_device_button.style.display = 'inline-block';
    connect_bluetooth_device_button.style.display = 'inline-block';
    disconnect_bluetooth_device_button.style.display = 'none';
    refresh_connection_status_button.style.display = 'inline-block';

    bluetooth_services_div.style.display = 'none';

    command_input.style.display = "none";
    stop_cli_button.style.display = "none";
    loader_div.style.display = 'none';

    tilt_monitoring_modal.style.display = 'none';

}

export async function onDisconnectBluetoothDeviceButtonClick() {

    try {



        await onStopCliButtonClick();



        loader_div.style.display = 'block';

        connection_status_span.textContent = '- Disconnecting...';

        log(`Disconnecting device ${gblDevice.name}...`);
        await gblDevice.gatt.disconnect();

    } catch(error) {
        log('Argh! ' + error);
        loader_div.style.display = 'none';
    }

}

export async function onRefreshConnectionStatusButtonClick() {

    try {

        loader_div.style.display = 'block';

        log(`Updating connection status of device ${gblDevice.name}...`);


        if (gblDevice.gatt.connected) {


            await setBLESpeed(abw.WR_VERY_FAST_CONN);
            

            connection_status_span.textContent = '- Connected';

            forget_bluetooth_device_button.style.display = 'inline-block';
            connect_bluetooth_device_button.style.display = 'none';
            disconnect_bluetooth_device_button.style.display = 'inline-block';
            refresh_connection_status_button.style.display = 'inline-block';


            // Detect HW Family

            const decoder = new TextDecoder('utf-8');
            const chr = abw.services.device_information.chars;
            const software_revision = await chr.software_revision.obj.readValue();
            gblIsAT2 = decoder.decode(software_revision).substring(0, 1) == '2';
            log('> Detected Hardware Family: ' + (gblIsAT2 ? 'AT2' : 'AT3') );
            connection_status_span.textContent = `- Connected (${gblIsAT2 ? 'AT2' : 'AT3'})`;

            if (gblDevice.name.substring(0,3) == 'ABW') {

                const decoder = new TextDecoder('utf-8');
                const chr = abw.services.device_information.chars;
                const software_revision = await chr.software_revision.obj.readValue();
                gblIsAT2 = decoder.decode(software_revision).substring(0, 1) == '2';
                log('> Detected Hardware Family: ' + (gblIsAT2 ? 'AT2' : 'AT3') );
                connection_status_span.textContent = `- Connected (${gblIsAT2 ? 'AT2' : 'AT3'})`;
                
                // gblIsAT2 = true;
                // log('> Detected Hardware Family: AT2');
                // connection_status_span.textContent = `- Connected (AT2)`;

            } else {
                gblIsAT2 = false;
                log('> Detected Hardware Family: AT3');
                connection_status_span.textContent = `- Connected (AT3)`;
            }


            // Detect DevEUI

            if (gblIsAT2) {
                gblDevEUIHex = abw.DEV_EUI_PREFIX_AT2 + gblDevice.name.substring(3);
            } else {
                gblDevEUIHex = abw.DEV_EUI_PREFIX_AT3 + gblDevice.name.substring(2);
            }
            log('> Detected DevEUI: ' + gblDevEUIHex);
            device_name_span.textContent = gblDevEUIHex;


            // Show BLE Devices Buttons
        
            bluetooth_services_div.style.display = 'block';

            log(`> Device ${gblDevice.name} is connected`);
        
        } else {

            gblIsAT2 = false;
            gblDevEUIHex = '';

            device_name_span.textContent = gblDevice ? gblDevice.name : '';
            connection_status_span.textContent = '- Not Connected';

            forget_bluetooth_device_button.style.display = 'inline-block';
            connect_bluetooth_device_button.style.display = 'inline-block';
            disconnect_bluetooth_device_button.style.display = 'none';
            refresh_connection_status_button.style.display = 'inline-block';
        
            bluetooth_services_div.style.display = 'none';
            command_input.style.display = "none";
            stop_cli_button.style.display = "none";

            log(`> Device ${gblDevice.name} is not connected`);

        }

        loader_div.style.display = 'none';

    } catch(error) {
        log('Argh! ' + error);
        loader_div.style.display = 'none';
    }

}

export async function setBLESpeed(speed) {

    let speedText;
    switch (speed) {
        case abw.WR_VERY_FAST_CONN:
            speedText = 'Very Fast';
            break;
        case abw.WR_FAST_CONN:
            speedText = 'Fast';
            break;
        case abw.WR_SLOW_CONN:
            speedText = 'Slow';
            break;
    }

    log(`Setting BLE Speed to ${speedText} for device ${gblDevEUIHex}...`);

    const chr_custom_simple_cmd = abw.services.abeeway_primary.chars.custom_simple_cmd.obj;
    await chr_custom_simple_cmd.writeValueWithoutResponse(Uint8Array.of(speed));
    const res = await chr_custom_simple_cmd.readValue();
    if (res.getUint8(0)== speed) {
        log(`> The BLE Speed has been set to ${speedText} for device ${gblDevEUIHex}`);
    } else {
        log(`> Failed to set BLE Speed to ${speedText} for device ${gblDevEUIHex}`);
    }

}

export function createStreamFromEvents(target, eventName) {
    return new ReadableStream({
        start(controller) {
            target.addEventListener(eventName, (event) => {
                controller.enqueue(event.target.value);
            });
        },
        cancel() {
            console.log(`The Event Stream of '${eventName}' events has been canceled`);
        },
    });
}
