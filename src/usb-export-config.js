import * as abw from './abw.js';
import { log } from './connection-mgmt.js';
import { makeSureLoggedIn, executeCmdGetResponse } from './usb-cli.js';

export async function onUsbExportConfigButtonClick() {

    try {

        loader_div.style.display = 'block';
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = true;
        });

        await makeSureLoggedIn();


        let cmd = `sys log off`;
        log(cmd);
        let { response } = await executeCmdGetResponse(cmd);
        log(response);

        
        let configFileString = '';

        if (gblIsAT2) {

            cmd = `config show`;
            log(cmd);
            ( { response } = await executeCmdGetResponse(cmd) );
            log(response);

            const lines = response.split(/\r\n/);

            for (const line of lines) {

                if (!line.startsWith(' ')) { continue }
                const lineSegments = line.trim().split(/\s+/);

                const paramID = parseInt(lineSegments[0], 10);
                if ( ( paramID >= 245 ) && ( paramID !== 249) ) {
                    continue;
                }
                
                configFileString += `${lineSegments[2]} = ${lineSegments[4]}\r\n`
            }

        } else {

            const cmd = `config show all`;
            log(cmd);

            const { response } = await executeCmdGetResponse(cmd);
            log(response);

            const lines = response.split(/\r\n/);

            for (const line of lines) {

                if (!line.startsWith(' 0x')) { continue }

                const lineSegments = line.trim().split(/\s+/);

                const paramID = parseInt(lineSegments[0], 16);
                const foundParam = abw.PARAMS_AT3.find(param => param[1] == paramID);
                
                if (foundParam) {

                    let paramValueString;

                    // params in the 'sys' group should not be exported
                    if (lineSegments[0].startsWith('0x00')) {
                        continue;
                    }

                    if (lineSegments[0].startsWith('0x05') || lineSegments[0].startsWith('0x06')) {
                        paramValueString = lineSegments[6];
                    } else {
                        paramValueString = lineSegments[5];
                    }

                    let paramValue;
                    switch (foundParam[2]) {
                        case 'i32':
                            paramValue = parseInt(paramValueString, 10);
                            break;
                        case 'string':
                            paramValue = paramValueString;
                            break;
                        case 'array':

                            paramValue = '"' + paramValueString.replace('\0', '') + '"';

                            // paramValue = `"${paramValueString.replace(/\s/g, '')}"`;
                            break;
                    }
                    configFileString += `${foundParam[0]} = ${paramValue}\r\n`

                } else {
                    log(`Unknown parameter id (0x${lineSegments[5].toString().padStart(4, '0')}) has been ignored from the export.`);
                }

            }           

        }

        // console.log(configFileString);
        gblConfig = configFileString;

        save_config_button.style.display = "block"
        document.querySelector("#save-confirm-modal").style.display = "block"

        log(`> The Configuration has been exported`);


        cmd = `sys log on`;
        log(cmd);
        ( { response } = await executeCmdGetResponse(cmd) );
        log(response);


        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = false;
        });

        loader_div.style.display = 'none';

    } catch(error) {
        log('Argh! ' + error);
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = false;
        });
        loader_div.style.display = 'none';
    }

}

export async function onUsbSaveConfigButtonClick() {

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