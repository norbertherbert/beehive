import { DEV_EUI_PREFIX_AT2 } from './abw.js';
import { log, streamLog, sleep } from './main.js';
import { addToCmdHistory, onBleCliCmdKeypressEvent, onStopCliButtonClick } from './cli.js';


let port, readLoopClosed, readableStreamClosed, keepReading;

export let writer, reader;


export async function onUsbConnectDeviceButtonClick() {

    if ('serial' in navigator) {
        try {
            
            // if (writer && writer) {

            // } else {           
                port = await navigator.serial.requestPort();
                await port.open({ baudRate: 57600 });
                port.addEventListener('disconnect', handleUnexpectedDisconnect);

                const textDecoder = new TextDecoderStream();
                readableStreamClosed = port.readable.pipeTo(textDecoder.writable);
                reader = textDecoder.readable.getReader();
                writer = port.writable.getWriter();
            // }

            await makeSureLoggedIn();
            gblDevEUIHex = await getDevEUI();
            gblIsAT2 = await isAT2();

            device_name_span.textContent = gblDevEUIHex;
            connection_status_span.textContent = `- Connected (${gblIsAT2 ? 'AT2' : 'AT3'})`;
            request_bluetooth_device_button.style.display = "none";
            usb_connect_device_button.style.display = "none";
            usb_disconnect_device_button.style.display = "inline-block";
            usb_services_div.style.display = "inline-block";

        } catch (e) {
            log(`Error: ${e.message}`);
            resetUI();
        }
    } else {
        log('Web Serial API not supported.');
    }

}

export async function onUsbDisconnectDeviceButtonClick() {
    handleDisconnect()
}

async function handleUnexpectedDisconnect() {
    
    resetUI();
    log('\nUnexpected disconnect of USB Device');
    keepReading = false;
    // reader.cancel();
    await readLoopClosed;
    await readableStreamClosed.catch(() => { /* Ignore the error */ });
    if (writer) { writer.releaseLock(); }
    reader = null;
    writer = null;    
    await port.close();
    log('USB Port Closed')
    // resetUI();

}

async function handleDisconnect() {

    keepReading = false;
    reader.cancel();
    await readLoopClosed;
    await readableStreamClosed.catch(() => { /* Ignore the error */ });
    writer.releaseLock();
    reader = null;
    writer = null;    
    await port.close();
    log('\nUSB Port Closed')
    resetUI();
}

function resetUI() {

    command_input.style.display = "none";
    command_input.value = "";
    command_input.removeEventListener('keypress', onUsbCliCmdKeypressEvent);
    stop_cli_button.style.display = "none";
    stop_cli_button.removeEventListener('click', onUsbStopCliButtonClick);

    device_name_span.textContent = "";
    connection_status_span.textContent = "";
    request_bluetooth_device_button.style.display = "inline-block";
    usb_connect_device_button.style.display = "inline-block";
    usb_disconnect_device_button.style.display = "none";
    usb_services_div.style.display = "none";

}


export async function readLoop() {
    keepReading = true;
    while ((port.readable && keepReading)) {
        try {
            const { value, done } = await reader.read();
            if (done) {
                console.log('Stream closed');
                break;
            }
            streamLog(value, false);
        } catch (error) {
            console.error('Read error:', error);
            break;
        } 
    }
    log('\nCLI reed loop ended');
}

export function startReadLoop() {
    readLoopClosed = readLoop();
}

export async function stopReadLoop() {
    keepReading = false;
    await writer.write(new TextEncoder().encode('\r\n'));
}

export async function onUsbStartCliButtonClick() {

    usb_services_div.style.display = "none";

    try {
        command_input.removeEventListener('keypress', onBleCliCmdKeypressEvent)
    } catch(e) {}
    command_input.addEventListener('keypress', onUsbCliCmdKeypressEvent);

    stop_cli_button.style.display = "block";
    try {
        stop_cli_button.removeEventListener('click', onStopCliButtonClick)
    } catch(e) {}
    stop_cli_button.addEventListener('click', onUsbStopCliButtonClick);

    log("\r\n".repeat(80));
    command_input.style.display = "block";
    command_input.focus();
    await writer.write(new TextEncoder().encode('\r\n'));

    startReadLoop();

    log('CLI reed loop started');

}


export async function onUsbStopCliButtonClick() {

    // alert('stop cli');
    
    command_input.style.display = "none";
    command_input.value = "";
    command_input.removeEventListener('keypress', onUsbCliCmdKeypressEvent);
    stop_cli_button.style.display = "none";
    stop_cli_button.removeEventListener('click', onUsbStopCliButtonClick);
    usb_services_div.style.display = "block";

    await stopReadLoop();
    
}

export async function onUsbCliCmdKeypressEvent(e) {
    if (e.key !== 'Enter') { return }
    try {
        const cmd = command_input.value;
        command_input.value = "";
        addToCmdHistory(cmd);
        log(cmd);
        await writer.write(new TextEncoder().encode(cmd + '\r\n'));
    } catch (e) {
        log(`USB Send Error: ${e}`);
    }
}


export const LOGIN_PROMPT = 'login:';
export const LOGIN_PROMPT_REGEX = /login.*:/;
export const STARTUP_PROMPT = 'startup>';
export const USER_PROMPT = 'user>';
export const SUPER_PROMPT = 'super>';

export async function executeCmdGetResponse(cmd) {

    await writer.write(new TextEncoder().encode(cmd + '\r\n'));

    let res, current_prompt;
    let response = '';
    
    while ( current_prompt == undefined ) {

        res = await reader.read();
        if (res.done) {
            throw Error('Read Stream is closed');
        }
        response += res.value;

        if (response.includes(USER_PROMPT)) {
            current_prompt = USER_PROMPT;
        // } else if (response.includes(LOGIN_PROMPT)) {
        //     current_prompt = LOGIN_PROMPT;
        } else if (response.includes(STARTUP_PROMPT)) {
            current_prompt = STARTUP_PROMPT;
        } else if (response.includes(SUPER_PROMPT)) {
            current_prompt = SUPER_PROMPT;

        } else if (response.match(LOGIN_PROMPT_REGEX)) {
            current_prompt = LOGIN_PROMPT;
        } else {
            current_prompt = undefined;
        }

    }
    
    return { response, current_prompt };

}

export async function makeSureLoggedIn() {
    let { response, current_prompt } = await executeCmdGetResponse('');
    while (current_prompt == LOGIN_PROMPT) {
        const passwd = prompt("Please enter the CLI PIN code", "123");
        ( { response, current_prompt } = await executeCmdGetResponse(passwd) );
        if (current_prompt == LOGIN_PROMPT) {
            log('Invalid password');
        }
    }
    if ( (current_prompt == USER_PROMPT) || (current_prompt == SUPER_PROMPT) ) {
        usb_import_config_button.style.display = 'inline-block';
        usb_export_config_button.style.display = 'inline-block';
    } else {
        usb_import_config_button.style.display = 'none';
        usb_export_config_button.style.display = 'none'; 
    }
}

async function getDevEUI() {
    const cmd = 'lora info';
    const { response } = await executeCmdGetResponse(cmd);
    const DevEUIIndex = response.indexOf('DevEUI:');
    const JoinEUIIndex = response.indexOf('JoinEUI:');
    const DevEUIString = response.slice(DevEUIIndex+7, JoinEUIIndex).replace(/[^0-9a-fA-F]/g, '').trim().toUpperCase();
    return DevEUIString;
}

async function isAT2() {
    const cmd = 'sys info';
    const { response } = await executeCmdGetResponse(cmd);

    return response.includes('AT2');
}

