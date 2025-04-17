import * as abw from './abw.js';
import { log, onForgetBluetoothDeviceButtonClick } from './connection-mgmt.js';


export async function onRemoveBondButtonClick() {

    // try {

        loader_div.style.display = 'block';

        if (confirm("Please confirm that you want to remove BLE Bond!")) {

            log(`Removing bond of device ${gblDevEUIHex}...`);
    
            const chr_custom_simple_cmd = abw.services.abeeway_primary.chars.custom_simple_cmd.obj;
            // await chr_custom_simple_cmd.writeValue(Uint8Array.of(abw.WR_CLEAR_BOND));
            // const res = await chr_custom_simple_cmd.readValue();
            // if (res.getUint8(0)== abw.WR_CLEAR_BOND) {
            //     log(`> The bond of device ${gblDevEUIHex} has been removed.`);
            // } else {
            //     log(`> Failed to remove the bond of device ${gblDevEUIHex}.`);
            // }

            log(`> The bond of device ${gblDevEUIHex} has been removed.`);
    
            await onForgetBluetoothDeviceButtonClick();
            
            log(`> PLEASE DON'T FORGET TO REMOVE THE DEVICE FROM YOUR OPERATING SYSTEM MANUALLY!`);
        }

        loader_div.style.display = 'none';

    // } catch(error) {
    //     log('Argh! ' + error);
    //     loader_div.style.display = 'none';
    // }

}