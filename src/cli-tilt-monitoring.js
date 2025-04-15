import * as abw from './abw.js';
import {log, setBLESpeed, createStreamFromEvents} from './connection-mgmt.js';


// import * as THREE from "three";
const { sin, cos, atan2, sqrt, PI, abs } = Math;

const CLI_PASSWD = '123';
const ANGLE_REQ_CMD_AT2 = 'sys acc show';
const ANGLE_REQ_CMD_AT3 = 'sys acc info';
const LOG_OFF_CMD = 'sys log off';


// ***********************************************
// *** Functions for angle calculations
// ***********************************************

// Angle variable names:
// -- psi: The angle between the Abeeway Device's -Z Axis and the Geographical North when the pole is in vertical orientation.
// -- phi: The angle between the direction of the pole's lane and the Geographical North.
// -- theta: The angle between the vertical direction and the pole.

const rad2Deg = 180/PI;
const deg2Rad = PI/180;

// --------------------------------------------------------

export const calculateYTheta = (gVec) => {
    const angle = atan2(
        sqrt(gVec.x**2 + gVec.z**2), gVec.y
    ) 
    return angle * 180/PI
}

export const calculateLocalYPhi = (gVec) => {
    const angle = atan2(gVec.x, gVec.z) 
    return angle * 180/PI
}


// ********************************************************
// Tilt Angle Threshold
const tiltAngleTresholdMultiplier = 3;



// --------------------------------------------------------
// +Z axes rotated in the ZX plane from Z axes towards X axes 

export const calculateTiltZX = (gVec) => {
    if (
        (gVec.x**2 + gVec.z**2) * tiltAngleTresholdMultiplier > gVec.y**2
    ) {
        return atan2(-gVec.x, gVec.z) * rad2Deg;
    } else {
        return undefined;
    }
}

// -Z axes rotated in the ZX plane from -Z axes towards X axes 
export const calculateTiltNZX = (gVec) => {
    if (
        (gVec.x**2 + gVec.z**2) * tiltAngleTresholdMultiplier > gVec.y**2
    ) {
        return atan2(-gVec.x, -gVec.z) * rad2Deg;
    } else {
        return undefined;
    }
}

// Z axes rotated in the ZY plane from Z axes towards Y axes 
export const calculateTiltZY = (gVec) => {
    if (
        (gVec.y**2 + gVec.z**2) * tiltAngleTresholdMultiplier > gVec.x**2
    ) {
        return atan2(-gVec.y, gVec.z) * rad2Deg;
    } else {
        return undefined;
    }
}

// -Z axes rotated in the ZY plane from -Z axes towards Y axes 
export const calculateTiltNZY = (gVec) => {
    if (
        (gVec.y**2 + gVec.z**2) * tiltAngleTresholdMultiplier > gVec.x**2
    ) {
        return atan2(-gVec.y, -gVec.z) * rad2Deg;
    } else {
        return undefined;
    }
}

// --------------------------------------------------------


export const calculateTiltYX = (gVec) => {
    if (
        (gVec.x**2 + gVec.y**2)*tiltAngleTresholdMultiplier > gVec.z**2
    ) {
        return atan2(-gVec.x, gVec.y) * rad2Deg;
    } else {
        return undefined;
    }
}

export const calculateTiltNYX = (gVec) => {
    if (
        (gVec.x**2 + gVec.y**2)*tiltAngleTresholdMultiplier > gVec.z**2
    ) {
        return atan2(-gVec.x, -gVec.y) * rad2Deg;
    } else {
        return undefined;
    }
}

export const calculateTiltYZ = (gVec) => {
    if (
        (gVec.z**2 + gVec.y**2)*tiltAngleTresholdMultiplier > gVec.x**2
    ) {
        return atan2(-gVec.z, gVec.y) * rad2Deg;
    } else {
        return undefined;
    }
}

export const calculateTiltNYZ = (gVec) => {
    if (
        (gVec.z**2 + gVec.y**2)*tiltAngleTresholdMultiplier > gVec.x**2
    ) {
        return atan2(-gVec.z, -gVec.y) * rad2Deg;
    } else {
        return undefined;
    }
}


// --------------------------------------------------------
// X

export const calculateTiltXY = (gVec) => {
    if (
        (gVec.y**2 + gVec.x**2)*tiltAngleTresholdMultiplier > gVec.z**2
    ) {
        return atan2(-gVec.y, gVec.x) * rad2Deg;
    } else {
        return undefined;
    }
}

export const calculateTiltNXY = (gVec) => {
    if (
        (gVec.y**2 + gVec.x**2)*tiltAngleTresholdMultiplier > gVec.z**2
    ) {
        return atan2(-gVec.y, -gVec.x) * rad2Deg;
    } else {
        return undefined;
    }
}

export const calculateTiltXZ = (gVec) => {
    if (
        (gVec.z**2 + gVec.x**2)*tiltAngleTresholdMultiplier > gVec.y**2
    ) {
        return atan2(-gVec.z, gVec.x) * rad2Deg;
    } else {
        return undefined;
    }
}

export const calculateTiltNXZ = (gVec) => {
    if (
        (gVec.z**2 + gVec.x**2)*tiltAngleTresholdMultiplier > gVec.y**2
    ) {
        return atan2(-gVec.z, -gVec.x) * rad2Deg;
    } else {
        return undefined;
    }
}

// --------------------------------------------------------





let cmdResBuffer = [];

function onSysAccShowCommandResponse(event) {
    
    let resTextChunk = new TextDecoder("ascii").decode(event.target.value);
    cmdResBuffer.push(resTextChunk);

    log(resTextChunk);

    if ( !resTextChunk.includes('OK') ) { return }

    const resText = cmdResBuffer.join('');
    cmdResBuffer = [];

    // console.log(resText);

    const xIndex = resText.indexOf('x:');
    const yIndex = resText.indexOf('y:');
    const zIndex = resText.indexOf('z:');

    if ( !(xIndex && yIndex && zIndex) ) { return }

    const vec = {
        x: parseInt(resText.slice(xIndex+3, yIndex-2)),
        y: parseInt(resText.slice(yIndex+3, zIndex-2)),
        z: parseInt(resText.slice(zIndex+3)),
    }

    let largestComponent;
    if ((abs(vec.x) > abs(vec.y))) {
        if ((abs(vec.z) > abs(vec.x))) {
            if (vec.z > 0) {
                largestComponent = 'z';
            } else {
                largestComponent = 'nz';
            }
        } else {
            if (vec.x > 0) {
                largestComponent = 'x';
            } else {
                largestComponent = 'nx';
            }
        }
    } else {
        if ((abs(vec.z) > abs(vec.y))) {
            if (vec.z > 0) {
                largestComponent = 'z';
            } else {
                largestComponent = 'nz';
            }
        } else {
            if (vec.y > 0) {
                largestComponent = 'y';
            } else {
                largestComponent = 'ny';
            }
        }
    }

    let anglesHTML = '';
    switch (largestComponent) {

        case 'x':

            let xy = calculateTiltXY(vec);
            xy = (xy ? xy.toFixed(1)+'°' : '').padEnd(7, ' ');
            let xz = calculateTiltXZ(vec);
            xz = (xz ? xz.toFixed(1)+'°' : '').padEnd(7, ' ');
            // const anglesX = ` X↷Y: ${xy} X↷Z: ${xz}`;
            anglesHTML = `tilt Y<sub>x</sub>: ${xy} tilt Z<sub>x</sub>: ${xz}`;

            break;

        case 'y':

            let yx = calculateTiltYX(vec);
            yx = (yx ? yx.toFixed(1)+'°' : '').padEnd(7, ' ');
            let yz = calculateTiltYZ(vec);
            yz = (yz ? yz.toFixed(1)+'°' : '').padEnd(7, ' ');
            // const anglesY = ` Y↷X: ${yx} Y↷Z: ${yz}`;
            anglesHTML = `tilt Z<sub>y</sub>: ${yz} tilt X<sub>y</sub>: ${yx}`;

            break;

        case 'z':

            let zx = calculateTiltZX(vec);
            zx = (zx ? zx.toFixed(1)+'°' : '').padEnd(7, ' ');
            let zy = calculateTiltZY(vec);
            zy = (zy ? zy.toFixed(1)+'°' : '').padEnd(7, ' ');
            // const anglesZ = ` Z↷X: ${zx} Z↷Y: ${zy}`;
            anglesHTML = `tilt X<sub>z</sub>: ${zx} tilt Y<sub>z</sub>: ${zy}`;

            break;

        case 'nx':

            let nxy = calculateTiltNXY(vec);
            nxy = (nxy ? nxy.toFixed(1)+'°' : '').padEnd(7, ' ');
            let nxz = calculateTiltNXZ(vec);
            nxz = (nxz ? nxz.toFixed(1)+'°' : '').padEnd(7, ' ');
            // const anglesNX = `-X↷Y: ${nxy} -X↷Z: ${nxz}`;
            anglesHTML = `tilt Z<sub>-x</sub>: ${nxz} tilt Y<sub>-x</sub>: ${nxy}`;

            break;

        case 'ny':

            let nyx = calculateTiltNYX(vec);
            nyx = (nyx ? nyx.toFixed(1)+'°' : '').padEnd(7, ' ');
            let nyz = calculateTiltNYZ(vec);
            nyz = (nyz ? nyz.toFixed(1)+'°' : '').padEnd(7, ' ');
            // const anglesNY = `-Y↷X: ${nyx} -Y↷Z: ${nyz}`;
            anglesHTML = `tilt X<sub>-y</sub>: ${nyx} tilt Z<sub>-y</sub>: ${nyz}`;    

            break;

        case 'nz':

            let nzx = calculateTiltNZX(vec);
            nzx = (nzx ? nzx.toFixed(1)+'°' : '').padEnd(7, ' ');
            let nzy = calculateTiltNZY(vec);
            nzy = (nzy ? nzy.toFixed(1)+'°' : '').padEnd(7, ' ');
            // const anglesNZ = `-Z↷X: ${nzx} -Z↷Y: ${nzy}`;
            anglesHTML = `tilt Y<sub>-z</sub>: ${nzy} tilt X<sub>-z</sub>: ${nzx}`;

            break;

    }

    // let angles = `[${vec.x}, ${vec.y}, ${vec.z}]<br/>`
    // let angles = `${anglesZ}        ${anglesNZ}<br/>${anglesY}        ${anglesNY}<br/>${anglesX}        ${anglesNX}`;

    // log(angles);
    tilt_angles_div.innerHTML = anglesHTML;
    tilt_angles_img.src = `./images/${largestComponent}1.png`;

} 


export async function onTiltMonitoringButtonClick() {

    try {

        // if (!gblIsAT2) { log('ALERT: This function is not supported on AT3 yet!'); return; }

        loader_div.style.display = 'block';
        await setBLESpeed(abw.WR_VERY_FAST_CONN);

        const chr_configuration = abw.services.abeeway_primary.chars.configuration.obj;
        const chr_custom_send_cli_cmd = abw.services.abeeway_primary.chars.custom_send_cli_cmd.obj;
        const chr_custom_rcv_serial_data = abw.services.abeeway_primary.chars.custom_rcv_serial_data.obj;


        log(`Starting the CLI...`);

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
                // res = await confEventReader.read();

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

            const chr_custom_simple_cmd = abw.services.abeeway_primary.chars.custom_simple_cmd.obj;
            await chr_custom_simple_cmd.writeValueWithoutResponse(Uint8Array.of(abw.WR_ENABLE_BLE_CLI));
            log("> BLE CLI has been turned on.");

        }

        await chr_custom_rcv_serial_data.startNotifications();
        log(`> Serial Data notifications have been started`);

        chr_custom_rcv_serial_data.addEventListener('characteristicvaluechanged', onSysAccShowCommandResponse);

        // These two lines are needed as a workaround to show the login prompt at start
        await new Promise(r => setTimeout(r, 300));
        await chr_custom_send_cli_cmd.writeValueWithoutResponse(Uint8Array.of(
            13, // '\r'.charCodeAt(0) & 0xff,
            10, // '\n'.charCodeAt(0) & 0xff,
        ));

        const encoder = new TextEncoder("ascii");

        await chr_custom_send_cli_cmd.writeValueWithoutResponse(encoder.encode( CLI_PASSWD + "\r\n" ));
        await chr_custom_send_cli_cmd.writeValueWithoutResponse(encoder.encode( LOG_OFF_CMD + "\r\n" ));

        const angleReqCmd = encoder.encode( 
            (gblIsAT2 ? ANGLE_REQ_CMD_AT2 : ANGLE_REQ_CMD_AT3) + "\r\n"
        );

        tilt_monitoring_modal.style.display = "block";
        loader_div.style.display = 'none';

        await chr_custom_send_cli_cmd.writeValueWithoutResponse(angleReqCmd);
        gblGetTiltParamsInterval = setInterval( async () => {
            await chr_custom_send_cli_cmd.writeValueWithoutResponse(angleReqCmd);
        }, 1000);

    } catch(error) {
        log('Argh! ' + error);
        // await setBLESpeed(abw.WR_FAST_CONN);
        loader_div.style.display = 'none';
    }

}