import * as abw from './abw.js';
import {log, setBLESpeed, createStreamFromEvents} from './connection-mgmt.js';


export async function onImportConfigButtonClick() {

    try {

        loader_div.style.display = 'block';
        document.querySelectorAll('button').forEach(elem => {
            elem.disabled = true;
        });
        await setBLESpeed(abw.WR_VERY_FAST_CONN);

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


        log(`Starting Configuration notifications of device ${gblDevEUIHex}...`);

        const chr_configuration = abw.services.abeeway_primary.chars.configuration.obj;
        const confEventStream = createStreamFromEvents(chr_configuration, 'characteristicvaluechanged');
        const confEventReader = confEventStream.getReader();
        await chr_configuration.startNotifications();
        log(`> Configuration notifications have been started`);

        let params = gblIsAT2 ? abw.PARAMS : abw.PARAMS_AT3;

        const encoder = new TextEncoder('ascii');

        for (const line of lines) {

            let segments = line.split('#')[0].trim();
            if (segments.length == 0) { continue; }
            segments = segments.split('=');
            if (segments.length != 2) {
                log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
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
                log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
                continue;
            }

            let paramValue, buffer, view;

            if (gblIsAT2) {

                paramValue = parseInt(paramValueString);
                if (isNaN(paramValue)) {
                    log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
                    continue;
                }
                if (paramName == 'config_flags') {
                    // It seems that bit 20 of coonfig_flags need to be 0 otherwise the process will stick...
                    paramValue = paramValue & ~(1<<20);
                    log('Bit 20 of the config_flags parameter has been set to 0 (it is needed to keep the BLE connection open).')
                }

                buffer = new ArrayBuffer(6);
                view = new DataView(buffer);
                view.setUint8(0, abw.WR_WRITE_CONF);
                view.setUint8(1, paramID);
                view.setInt32(2, paramValue);

                // TODO: Do we have to save the config?

            } else {
                
                switch (foundParam[2]) {
                    case 'i32':
                        paramValue = parseInt(paramValueString);
                        if (isNaN(paramValue)) {
                            log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
                            continue;
                        }
                        buffer = new ArrayBuffer(7);
                        view = new DataView(buffer);
                        view.setUint8(0, abw.WR_WRITE_CONF);
                        view.setUint16(1, paramID);
                        view.setUint8(3, abw.PARAM_TYPE_INTEGER);
                        view.setInt32(4, paramValue);
                        break;

                    // TODO:
                    // case 'f32':
                    //     paramValue = parseFloat(paramValueString);
                    //     if (isNaN(paramValue)) {
                    //         log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
                    //         continue;
                    //     }
                    //     buffer = new ArrayBuffer(7);
                    //     view = new DataView(buffer);
                    //     view.setUint8(0, abw.WR_WRITE_CONF);
                    //     view.setUint16(1, paramID);
                    //     view.setUint8(3, abw.TYPE_INTEGER);
                    //     view.setInt32(4, paramValue);
                    //     break;

                    case 'string':

                        if ((paramValueString.slice(0,1) != '"') && (paramValueString.slice(-1) != '"')) {
                            log(`ERROR IN CONFIGURATION FILE (Expected format is "abdc"): ${line}\n> Line ignored`);
                            continue;
                        }

                        paramValueString = paramValueString.slice(1, -1).trim();
                        
                        // console.log('paramValueString: ' + paramValueString);
                        // console.log(paramValueString.length);

                        paramValue = encoder.encode(paramValueString);
                        // console.log(paramValue.length);

                        buffer = new ArrayBuffer(3 + paramValue.byteLength + 1);
                        view = new DataView(buffer);

                        view.setUint8(0, abw.WR_WRITE_CONF);
                        view.setUint16(1, paramID);
                        view.setUint8(3, abw.PARAM_TYPE_STRING);
                        view.setUint8(4 + paramValue.byteLength, 0); // null terminated string

                        view = new Uint8Array(buffer)
                        view.set(paramValue, 3);

                        // console.log(buffer)

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
                        paramValueString = paramValueString.slice(1, -1).trim();
                        const paramValueArray = paramValueString.split(',');

                        buffer = new ArrayBuffer(3 + paramValueArray.length);
                        view = new DataView(buffer);
                        view.setUint8(0, abw.WR_WRITE_CONF);
                        view.setUint16(1, paramID);
                        view.setUint8(3, abw.PARAM_TYPE_BYTEARRAY);
                        for (let i=0; i < paramValueArray.length; i++) {
                            const v = parseInt(paramValueArray[i], 16);
                            if ( isNaN(v) || (v > 255) || (v < 0) ) {
                                log(`ERROR IN CONFIGURATION FILE: ${line}\n> Line ignored`);
                                continue;
                            }
                            view.setUint8(4+i, v);
                        }

                        break;

                    default:
                        continue;
                }

                // TODO: new simple command 0x05 to send via char 0x273D to save the config
                // CHR_CUSTOM_CMD, WR_SAVE_CONFIG

                const chr_custom_cmd = abw.services.abeeway_primary.chars.custom_cmd.obj;
                await chr_custom_cmd.writeValue(Uint8Array.of(abw.WR_SAVE_CONFIG));
                log("> Parameter settings have been saved.");
                

            }

            await chr_configuration.writeValueWithoutResponse(buffer);

            const res = await confEventReader.read();
            let responseMsg = '';
            switch (res.value.getUint8(0)) {
                case abw.NOTIF_CONF_SUCCESS:
                    responseMsg = `> Parameter sent and accepted: ${line}`;
                    break;
                case abw.NOTIF_CONF_INVALID:               // Only relevant for AT2
                    responseMsg = `> The parameter value was not accepted by the device: '${line}'`;
                    break;
                case abw.NOTIF_CONF_VALUE_NOT_SUPPORTED:   // Only relevant for AT3
                    responseMsg = `> The parameter value is not supported: '${line}'`;
                    break;
                case abw.NOTIF_CONF_DATA_LENGTH_ERROR:    // Only relevant for AT3
                    responseMsg = `> The length of the parameter is invalid: '${line}'`;
                    break;
                default: 
                    responseMsg = `> The device sent an invalid response to the "parameter set" request: '${line}'`;
            }
            
            log(responseMsg);

        }

        log(`> Configuration notifications have been stopped`);
        log(`> The selected Configuration has been imported`);

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