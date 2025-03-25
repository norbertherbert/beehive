import * as abw from './abw.js';
import {log, setBLESpeed, createStreamFromEvents} from './main.js';
import {calculateTheta, calculateLocalPhi, calculateTiltX, calculateTiltZ} from './abeeway-decoder.js'; 


export async function onTiltMonitoringButtonClick() {

    try {

        if (!gblIsAT2) { log('ALERT: This function is not supported on AT3 yet!'); return; }

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

        let i = 1;
        const getCoords = async () => {
            if (i>10) { return }
            i++;
            const coordinates = [];
            for (let paramID of [250, 251, 252]) {
                const buffer = new ArrayBuffer(2);
                const view = new DataView(buffer);
                view.setUint8(0, abw.WR_READ_CONF);
                view.setUint8(1, paramID);
                chr_configuration.writeValueWithoutResponse( view.buffer );
                const confEvent = await confEventReader.read();
                const value = confEvent.value;
                const paramID1 = value.getUint16(0);
                if (paramID !== paramID1) {
                    coordinates.push(undefined);
                    continue;
                }
                let cord = value.getInt32(2);
                if (cord>0) {
                    cord = 0xffff & cord;
                }
                coordinates.push(cord);
            }
            const vec = {
                x: -coordinates[0],
                y: coordinates[1],
                z: -coordinates[2],
            }

            console.log(vec);
            log(`Theta: ${calculateTheta(vec).toFixed(2)}, Phi: ${calculateLocalPhi(vec).toFixed(2)}, tiltX: ${calculateTiltX(vec).toFixed(2)}, tiltZ: ${calculateTiltZ(vec).toFixed(2)}`);

            setTimeout(getCoords, 1000)
        }

        getCoords();


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
