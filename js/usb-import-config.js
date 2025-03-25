import * as abw from './abw.js';
import { log } from './main.js';
import { makeSureLoggedIn, executeCmdGetResponse } from './usb-cli.js';




export async function onUsbImportConfigButtonClick() {

    try {

        loader_div.style.display = 'block';
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = true;
        });


        await makeSureLoggedIn();

        const options = {
            types: [
                {
                    description: 'Text Files',
                    accept: {
                        'text/plain': ['.txt'],
                    },
                },
            ],
        };
        const [fileHandle] = await window.showOpenFilePicker(options);
        const file = await fileHandle.getFile();
        const contents = await file.text();
        // const lines = contents.split( /[\r\n]+/ );
        const lines = contents.split(/\r?\n/);
        // const lines = contents.split(/\r\n/);
        // const lines = contents.split(/\n/);

        let params = gblIsAT2 ? abw.PARAMS : abw.PARAMS_AT3;

        // const encoder = new TextEncoder('ascii');


        let cmd = `sys log off`;
        log(cmd);
        let { response } = await executeCmdGetResponse(cmd);
        log(response);


        for (const line of lines) {


            let line_without_comments = line.split('#')[0].trim();
            if (line_without_comments.length == 0) { continue; }
            
            let segments = line_without_comments.split('=');
            if (segments.length != 2) {
                log(`ERROR IN CONFIGURATION FILE (invalid format): ${line}\n> Line ignored`);
                continue;
            } 
            let [paramName, paramValueString] = segments;
            paramName = paramName.trim();
            paramValueString = paramValueString.trim();
            let paramID;
            let foundParam = params.find(param => param[0] == paramName);
            if (foundParam) { 
                paramID = foundParam[1];
            } else {
                log(`ERROR IN CONFIGURATION FILE (unknown parameter): ${line}\n> Line ignored`);
                continue;
            }


            if (gblIsAT2) {

                const paramValue = parseInt(paramValueString);
                if (isNaN(paramValue)) {
                    log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
                    continue;
                }
                paramValueString = paramValue.toString();

            } else {
                
                switch (foundParam[2]) {
                    case 'i32':
                        const paramValue = parseInt(paramValueString);
                        if (isNaN(paramValue)) {
                            log(`ERROR IN CONFIGURATION FILE (invalid number format): ${line}\n> Line ignored`);
                            continue;
                        }
                        paramValueString = paramValue.toString();
                        break;
                    case 'string':
                        if ((paramValueString.slice(0,1) != '"') && (paramValueString.slice(-1) != '"')) {
                            log(`ERROR IN CONFIGURATION FILE (Expected format is "abdc"): ${line}\n> Line ignored`);
                            continue;
                        }
                        break;

                    case 'array':

                        if ((paramValueString.slice(0,1) != '"') && (paramValueString.slice(-1) != '"')) {
                            log(`ERROR IN CONFIGURATION FILE (Expected format is "{aa,bb,cc,...}"): ${line}\n> Line ignored`);
                            continue;
                        }
                        paramValueString = paramValueString.slice(1, -1).trim();
                        if ((paramValueString.slice(0,1) != '{') && (paramValueString.slice(-1) != '}')) {
                            log(`ERROR IN CONFIGURATION FILE (Expected format is "{aa,bb,cc,...}"): ${line}\n> Line ignored`);
                            continue;
                        }
                        const paramValueArray = paramValueString.slice(1, -1).trim().split(',');
                        for (let i=0; i < paramValueArray.length; i++) {
                            const v = parseInt(paramValueArray[i], 16);
                            if ( isNaN(v) || (v > 255) || (v < 0) ) {
                                log(`ERROR IN CONFIGURATION FILE (Invalid array element. Expected format is "{aa,bb,cc,...}"): ${line}\n> Line ignored`);
                                continue;
                            }
                        }

                        break;

                    default:
                        continue;
                }

            }
            log('----------------------------------------------------------');           
            log(`Config File Line: ${line}`);
            log('----------------------------------------------------------');           
            const cmd = `config set 0x${paramID.toString(16).padStart(4, '0')} ${paramValueString}`;
            log(cmd);

            ( { response } = await executeCmdGetResponse(cmd) );
            log(response);

        }


        cmd = `config save`;
        log(cmd);
        ( { response } = await executeCmdGetResponse(cmd) );
        log(response);

        log(`> The selected Configuration has been imported`);


        cmd = `sys log on`;
        log(cmd);
        ({ response } = await executeCmdGetResponse(cmd));
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