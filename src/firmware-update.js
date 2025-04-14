import * as abw from './abw.js';
import {log, setBLESpeed, createStreamFromEvents} from './connection-mgmt.js';


export async function onFirmwareUpdateButtonClick() {

    try {





        // if (!gblIsAT2) { log('ALERT: This function is not supported on AT3 yet!'); return; }





        loader_div.style.display = 'block';
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = true;
        });
        
        await setBLESpeed(abw.WR_VERY_FAST_CONN);

        const options = {
            types: [
              {
                description: 'Firmware Files',
                accept: {
                  'application/octet-stream': ['.bin'],
                },
              },
            ],
          };
        const [fileHandle] = await window.showOpenFilePicker(options);
        const file = await fileHandle.getFile();

        const byteStream = createByteStreamFromBlob(file);
        const byteStreamReader = byteStream.getReader(({ mode: "byob" }));
       
        const chr_custom_mcu_fw_update = abw.services.abeeway_primary.chars.custom_mcu_fw_update.obj;

        const eventStream = createStreamFromEvents(chr_custom_mcu_fw_update, 'characteristicvaluechanged');
        const eventReader = eventStream.getReader();
        
        chr_custom_mcu_fw_update.startNotifications();
        log(`> Firmware Update notifications have been started...`);


        let arrayBuffer;
        let dataView;
        let notif;

        // Get DevEUI

        const devEUI = BigInt("0x" + gblDevEUIHex);
        
        // Enable Firmware Update

        arrayBuffer = new ArrayBuffer(9);
        dataView = new DataView(arrayBuffer);
        dataView.setUint8(0, abw.WR_ENABLE_DFU);
        dataView.setBigUint64(1, devEUI);
        await chr_custom_mcu_fw_update.writeValueWithoutResponse(arrayBuffer);
        notif = await eventReader.read();
        if (notif.value.byteLength != 2 || notif.value.getUint8(0) != abw.WR_ENABLE_DFU || notif.value.getUint8(1) != 0) {
            throw Error(`Didn't receive proper value notification as response to Enable Firmware Update over BLE: ${notif.value.getUint8(1)}`);
        }

        log(`> Firmware Update over BLE is enabled...`);









        // Begin firmware update

        if (gblIsAT2) { 

            arrayBuffer = new ArrayBuffer(5);
            dataView = new DataView(arrayBuffer);
            dataView.setUint8(0, abw.WR_START_DFU);
            dataView.setUint32(1, file.size);
    
            await chr_custom_mcu_fw_update.writeValueWithoutResponse(arrayBuffer);
            notif = await eventReader.read();
            if (notif.value.byteLength != 2 || notif.value.getUint8(0) != abw.WR_START_DFU || notif.value.getUint8(1) != 0) {
                throw Error(`Didn't receive proper value notification as response to Start Firmware Update over BLE: ${notif.value.getUint8(1)}`);
            }
   
        } else {

            arrayBuffer = new ArrayBuffer(6);
            dataView = new DataView(arrayBuffer);
            dataView.setUint8(0, abw.WR_START_DFU);
            dataView.setUint32(1, file.size);
            dataView.setUint8(5, abw.FW_TYPE_MCU);
    
            await chr_custom_mcu_fw_update.writeValueWithoutResponse(arrayBuffer);
            notif = await eventReader.read();
            if (notif.value.byteLength != 2 || notif.value.getUint8(0) != abw.WR_START_DFU || notif.value.getUint8(1) != 0) {
                throw Error(`Didn't receive proper value notification as response to Start Firmware Update over BLE: ${notif.value.getUint8(1)}`);
            }

        }

        log(`> Firmware Update start message has been sent to the device...`);









        const chunkSize = 16;
        let offset = 0;
        let numOfChunks = Math.ceil(file.size / 16);
        let chunkIndex = 0;
        let bytesSent = 0;
        let crc = 0;
        

        command_input.style.display = "block";
        command_input.value = "";


        log(`> Firmware Update in progress...`);

        let byteStreamChunk;


        byteStreamChunk = await byteStreamReader.read(new Uint8Array(new ArrayBuffer(chunkSize)));
        crc = crc16(byteStreamChunk.value, crc);
        let buf = new ArrayBuffer(4 + byteStreamChunk.value.byteLength);
        let view = new Uint8Array(buf);
        view[0] = abw.WR_WRITE_BINARY_DATA;
        view[1] = (offset >> 16) & 0xff;
        view[2] = (offset >> 8) & 0xff;
        view[3] = offset & 0xff;
        view.set(byteStreamChunk.value, 4);
        await chr_custom_mcu_fw_update.writeValueWithoutResponse(buf);
        bytesSent += byteStreamChunk.value.length;
        offset += byteStreamChunk.value.length;
        chunkIndex += 1;
        command_input.value = `  FW Chunk: ${chunkIndex} / ${numOfChunks}`;


        byteStreamChunk = await byteStreamReader.read(new Uint8Array(new ArrayBuffer(chunkSize)));
        // while ((chunkIndex < numOfChunks) && !byteStreamChunk.done) {
        while (chunkIndex < numOfChunks) {

            if ((byteStreamChunk.value.length == 1) && (byteStreamChunk.value.at(0) == 0)) continue;
            

            crc = crc16(byteStreamChunk.value, crc);
            let buf = new ArrayBuffer(4 + byteStreamChunk.value.byteLength);
            let view = new Uint8Array(buf);
            view[0] = abw.WR_WRITE_BINARY_DATA;
            view[1] = (offset >> 16) & 0xff;
            view[2] = (offset >> 8) & 0xff;
            view[3] = offset & 0xff;
            view.set(byteStreamChunk.value, 4);
            await chr_custom_mcu_fw_update.writeValueWithoutResponse(buf);
            bytesSent += byteStreamChunk.value.length;
            offset += byteStreamChunk.value.length;
            chunkIndex += 1;
            command_input.value = `  FW Chunk: ${chunkIndex} / ${numOfChunks}`;


            notif = await eventReader.read();
            if (
                notif.value.byteLength != 5 || 
                notif.value.getUint8(0) != abw.WR_WRITE_BINARY_DATA
            ) {
                throw Error(`Error: Invalid response received to Write Binary Data chunk Request: ${notif.value.getUint8(1)}`);
            }
            if ( notif.value.getUint8(1) != abw.FW_UPDATE_COMPLETED_SUCCESSFULLY ) {
                throw Error(`Error recevied as response to Write Binary Data chunk Request: ${abw.FW_DFU_STATUS_ARRAY[notif.value.getUint8(1)]}`);
            }


            byteStreamChunk = await byteStreamReader.read(new Uint8Array(new ArrayBuffer(chunkSize)));
        }

        notif = await eventReader.read();
        if (
            notif.value.byteLength != 5 || 
            notif.value.getUint8(0) != abw.WR_WRITE_BINARY_DATA
        ) {
            throw Error(`Error: Invalid response received to Write Binary Data chunk Request: ${notif.value.getUint8(1)}`);
        }
        if ( notif.value.getUint8(1) != abw.FW_UPDATE_COMPLETED_SUCCESSFULLY ) {
            throw Error(`Error recevied as response to Write Binary Data chunk Request: ${abw.FW_DFU_STATUS_ARRAY[notif.value.getUint8(1)]}`);
        }

        
        log(command_input.value);
        command_input.value = "";
        command_input.style.display = "none";


        // Send CRC

        // console.log(`  Final CRC: ${crc}`);

        arrayBuffer = new ArrayBuffer(3);
        dataView = new DataView(arrayBuffer);
        dataView.setUint8(0, abw.WR_BINARY_DATA_CRC);
        dataView.setUint16(1, crc);

        await chr_custom_mcu_fw_update.writeValueWithoutResponse(arrayBuffer);
        notif = await eventReader.read();
        notif.value = notif.value;

        if (
            notif.value.byteLength != 2 || 
            notif.value.getUint8(0) != abw.WR_BINARY_DATA_CRC
        ) {
            throw Error(`Error: Invalid response received to Write CRC Request`);
        }
        if ( notif.value.getUint8(1) != abw.FW_UPDATE_COMPLETED_SUCCESSFULLY ) {
            throw Error(`Error received as a response to write CRC request: ${abw.FW_DFU_STATUS_ARRAY[notif.value.getUint8(1)]}`);
        }
        
        chr_custom_mcu_fw_update.stopNotifications();
        log(`> Firmware Update notifications have been stopped`);

        log(`> Firmware Update finished. Status: ${abw.FW_DFU_STATUS_ARRAY[notif.value.getUint8(1)]}`);
        log(`> Please wait until the device restarts. Don't start any BLE operation before that!`);
        log(`> The restart will take around 1 min.`);

        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = false;
        });
        // await setBLESpeed(abw.WR_FAST_CONN);
        loader_div.style.display = 'none';

    } catch(error) {
        log('Argh! ' + error);
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = false;
        });
        // await setBLESpeed(abw.WR_FAST_CONN);
        loader_div.style.display = 'none';
    }

}


// CRC16 XMODEM
function crc16(uint8Array, crc = 0, xorout = 0) {
    for(let i = 0, t; i < uint8Array.byteLength; i++, crc &= 0xFFFF) {
        t = (crc >> 8) ^ uint8Array[i];
        t ^= t >> 4;
        crc = (crc << 8) ^ (t << 12) ^ (t << 5) ^ t;
    }
    return crc ^ xorout;
}


function createByteStreamFromBlob(blob) {
    const blobStreamReader = blob.stream().getReader();
    return new ReadableStream({
        async pull(controller) {
            let { done, value: blobStreamChunk } = await blobStreamReader.read();
            if (done) {
                controller.enqueue(new Uint8Array(1));
                controller.close();
            } else {
                controller.enqueue(blobStreamChunk);
            }
        },
        type: 'bytes',
    });
}