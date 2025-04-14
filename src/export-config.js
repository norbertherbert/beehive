import * as abw from './abw.js';
import {log, setBLESpeed, createStreamFromEvents} from './connection-mgmt.js';


export async function onExportConfigButtonClick() {

    try {

        loader_div.style.display = 'block';
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = true;
        });
        log(`Starting Configuration notifications of device ${gblDevEUIHex}...`);

        await setBLESpeed(abw.WR_VERY_FAST_CONN);


        const chr_configuration = abw.services.abeeway_primary.chars.configuration.obj;
        const confEventStream = createStreamFromEvents(chr_configuration, 'characteristicvaluechanged');
        const confEventReader = confEventStream.getReader();
        await chr_configuration.startNotifications();
        log(`> Configuration notifications have been started`);

        let lines = "";

        if (gblIsAT2) {
            for (const param of abw.PARAMS) {

                const buffer = new ArrayBuffer(2);
                const view = new DataView(buffer);
                view.setUint8(0, abw.WR_READ_CONF);
                view.setUint8(1, param[1]);
                chr_configuration.writeValueWithoutResponse( view.buffer );
                const confEvent = await confEventReader.read();
                const value = confEvent.value;
                const p = value.getUint16(0);
                const v = value.getInt32(2);
                const paramName = abw.PARAMS.find(param => param[1] == p )[0];
                const line = paramName + ' = ' + v;
                
                log(line);
                lines += line + '\n';

            }
        } else {

            const decoder = new TextDecoder('ascii');

            for (const param of abw.PARAMS_AT3) {
                
                if ( 
                    (param[1] < 0x10) || // params in the 'sys' group should not be exported
                    (param[1] == 0x010e) // 'core_cli_password' should not be exported
                ) {
                    continue;
                }

                const buffer = new ArrayBuffer(3);
                const view = new DataView(buffer);
                view.setUint8(0, abw.WR_READ_CONF);
                view.setUint16(1, param[1]);
                chr_configuration.writeValueWithoutResponse( view.buffer );
             
                const confEvent = await confEventReader.read();
                const value = confEvent.value;

                if (value.getUint8(0) != 0) { continue; }
                if (value.getUint16(1) != param[1]) { continue; }

                if (value.getUint8(0) != abw.NOTIF_CONF_SUCCESS) {
                    console.log(`ERROR while Reading Param: '${param[0]}'`);
                    console.log(value);
                    continue; 
                }

                let line, paramValue;
                switch (param[2]) {

                    case 'i32':
                        paramValue = value.getInt32(4);
                        line = `${param[0]} = ${paramValue}`;
                        break;
                    case 'string':
                        paramValue = decoder.decode(new DataView(value.buffer, 4, value.byteLength-5));
                        line = `${param[0]} = "${paramValue}"`;
                        break;
                    case 'array':
                        // console.log(param[0]);
                        // console.log(value);

                        paramValue = '"{' + value.getUint8(4).toString(16).padStart(2,0);
                        let i = 5;
                        while (i < value.byteLength) {
                            paramValue += ','
                            paramValue += value.getUint8(i).toString(16).padStart(2,0);
                            i++;
                        }
                        paramValue += '}"';
                        line = `${param[0]} = ${paramValue}`;
                        break;

                    default:
                        continue;

                }
              
                log(line);
                lines += line + '\n';

            }
        }

        await chr_configuration.stopNotifications()
        log(`> Configuration notifications have been stopped`);

        gblConfig = lines;

        save_config_button.style.display = "block"
        document.querySelector("#save-confirm-modal").style.display = "block"

        log(`> The Configuration has been exported`);

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

export async function onSaveConfigButtonClick() {

    document.querySelector('#save-confirm-modal').style.display = "none"

    if (gblConfig != '') {

        try {
            let taBlob = new Blob([gblConfig], {type: 'text/plain'});
            let dateString = new Date().toISOString().slice(0, 16);
            const pickerOptions = {
            suggestedName: `${gblIsAT2?'AT2':'AT3'}_${gblDevEUIHex}_${dateString}.txt`,
            types: [
                {
                description: 'Abeeway Configuration File',
                accept: {
                    'text/plain': ['.txt'],
                },
                },
            ],
            };
            const fileHandle = await window.showSaveFilePicker(pickerOptions);
            const writableFileStream = await fileHandle.createWritable();
            await writableFileStream.write(taBlob);
            await writableFileStream.close();
        } catch(error) {
            log('Argh! ' + error);
        }

        gblConfig = '';

        log(`> The Configuration has been saved`);

    };

}