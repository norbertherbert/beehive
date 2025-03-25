import './style.css';

import {
    log,
    onRequestBluetoothDeviceButtonClick,
    onForgetBluetoothDeviceButtonClick,
    onConnectBluetoothDeviceButtonClick,
    onDisconnectBluetoothDeviceButtonClick,
    onRefreshConnectionStatusButtonClick,
} from './connection-mgmt.js';

import { onGetBluetoothDeviceInfoButtonClick } from './get-device-info.js';
import { onImportConfigButtonClick } from './import-config.js';
import { onExportConfigButtonClick, onSaveConfigButtonClick } from './export-config.js';
import { onStartCliButtonClick, onStopCliButtonClick, onArrowUpDn } from './cli.js';
import { onFirmwareUpdateButtonClick } from './firmware-update.js';
import { onRemoveBondButtonClick } from './remove-bond.js';
import { onTiltMonitoringButtonClick } from './cli-tilt-monitoring.js';
// import { onTiltMonitoringButtonClick} from './tilt-monitoring.js';

import { 
    onUsbConnectDeviceButtonClick, onUsbDisconnectDeviceButtonClick, 
    onUsbStartCliButtonClick, onUsbStopCliButtonClick 
} from './usb-cli.js';
import { onUsbImportConfigButtonClick } from './usb-import-config.js';
import { onUsbExportConfigButtonClick } from './usb-export-config.js';

import {CONNECTION_ALERT, AT3_LIMITATIONS, API_ALERT} from './abw.js';


function isWebBluetoothEnabled() {
    if (('bluetooth' in navigator) && ('showOpenFilePicker' in window)) {
        log( AT3_LIMITATIONS );
        return true;
    } else {
        log( API_ALERT );
        return false;
    }
}

window.onload = () => {

    window.name="beehive";
    
    if (isWebBluetoothEnabled()) {


        if (/Chrome\/(\d+\.\d+.\d+.\d+)/.test(navigator.userAgent)){
            if (55 > parseInt(RegExp.$1)) {
                log('WARNING! This App works with Chrome ' + 55 + ' or later.');
            }
        }


        beequeen_link.addEventListener('click', function() {

            if (
                (window.opener !== null) &&
                (window.opener.name === 'beequeen')
            ) {
                // window.opener.focus();
                alert("Beequeen is the parent of this window. Please switch back manually using your mouse!");
            } else {
                if (!gblBeequeenTab) {
                    gblBeequeenTab = window.open('https://nano-things.net/beequeen/', 'beequeen');
                } else {
                    gblBeequeenTab.focus();
                }
            }

        });


        request_bluetooth_device_button.addEventListener('click', function() {
            onRequestBluetoothDeviceButtonClick();
        });
        forget_bluetooth_device_button.addEventListener('click', function() {
            onForgetBluetoothDeviceButtonClick();
        });
        connect_bluetooth_device_button.addEventListener('click', function() {
            onConnectBluetoothDeviceButtonClick();
        });
        disconnect_bluetooth_device_button.addEventListener('click', function() {
            onDisconnectBluetoothDeviceButtonClick();
        });
        refresh_connection_status_button.addEventListener('click', function() {
            onRefreshConnectionStatusButtonClick();
        });

        get_bluetooth_device_info_button.addEventListener('click', function() {
            onGetBluetoothDeviceInfoButtonClick();
        });
        import_config_button.addEventListener('click', function() {
            onImportConfigButtonClick();
        });
        export_config_button.addEventListener('click', function() {
            onExportConfigButtonClick();
        });
        save_config_button.addEventListener('click', function() {
            onSaveConfigButtonClick();
        });
        document.querySelector('#save-confirm-modal .modal-close').addEventListener('click', function() {
            document.querySelector('#save-confirm-modal').style.display = 'none';
        });
        start_cli_button.addEventListener('click', function() {
            onStartCliButtonClick();
        });
        // stop_cli_button.addEventListener('click', function() {
        //     onStopCliButtonClick();
        // });

        firmware_update_button.addEventListener('click', function() {
            onFirmwareUpdateButtonClick();
        });

        remove_bond_button.addEventListener('click', function() {
            onRemoveBondButtonClick();
        });

        tilt_monitoring_button.addEventListener('click', function() {
            onTiltMonitoringButtonClick();
        });

        document.querySelector('#tilt_monitoring_modal .modal-close').addEventListener('click', function() {
            clearInterval(gblGetTiltParamsInterval);
            tilt_monitoring_modal.style.display = 'none';
            onStopCliButtonClick();
        });

        command_input.addEventListener('keydown', function(event) { 
            onArrowUpDn(event, this);
        });

        usb_connect_device_button.addEventListener('click', function() {
            onUsbConnectDeviceButtonClick();
        })

        usb_disconnect_device_button.addEventListener('click', function() {
            onUsbDisconnectDeviceButtonClick();
        })

        usb_import_config_button.addEventListener('click', function() {
            onUsbImportConfigButtonClick();
        });
        usb_export_config_button.addEventListener('click', function() {
            onUsbExportConfigButtonClick();
        });
        usb_start_cli_button.addEventListener('click', function() {
            onUsbStartCliButtonClick();
        });

        log_div.addEventListener('mouseup', () => {
            const selection = window.getSelection();
            if (selection.isCollapsed) {
                command_input.focus();
            }
        });

        log(CONNECTION_ALERT);

    }
};
