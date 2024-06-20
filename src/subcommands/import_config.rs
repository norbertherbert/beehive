
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use anyhow::{anyhow, Result, Context};
use futures::StreamExt;
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Peripheral as _, WriteType, ValueNotification};
use log::*;

use crate::abw_ble_utils::{
    abw,
    find_dev::find_abw_device,
};
use crate::abw_params::get_param_name_to_id;


pub async fn import_config (
    device_name: &String, 
    file_path: &String, 
    ble_adapter: &Adapter, 
) -> Result<()>
{

    // Try to open the file
    let mut file = match OpenOptions::new()
        .read(true)
        .open(file_path)
    {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e); debug!("{:?}", e);
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!("The file '{}' was not found!", file_path);
                    return Ok(());
                },
                _ => {
                    error!("{}", e); debug!("{:?}", e);
                    println!("The file '{}' cannot be opened!", file_path);
                    return Ok(());
                }
            }
        }

    };

    // Looking for the specified Abeeway device
    println!("Scanning...");
    let device = match find_abw_device(&ble_adapter, device_name).await {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e); debug!("{:?}", e);
            println!("Cannot find the selected Abeeway Device.");
            println!("{}", abw::FIX_FOR_NOT_ADVERTIZING);
            return Ok(())
        }
    };
    let Some(device) = device else {
        println!("Device {} was not found", device_name);
        println!("{}", abw::FIX_FOR_NOT_ADVERTIZING);
        return Ok(())
    };

    // Making sure that the device is connected

    let is_connected = device.is_connected().await
        .with_context(||"error while checking if the selected device is connected")?;

    println!("The Device was found:\n    {} - {}", 
        device_name, 
        if is_connected { "Connected" } else { "Not Connected" }
    );
    if !is_connected {
        println!("Connecting...");
        match device.connect().await {
            Ok(_) => {},
            Err(e) => {
                error!("{}", e); debug!("{:?}", e);
                println!("{}", abw::FIX_FOR_CORRUPTED_PAIRING);
                return Ok(())
            }
        };
    } 
    println!("Connected.");

    // Discovering services/characteristics on the device

    info!("Discovering BLE Services...");

    device.discover_services().await
        .with_context(||"error while discovering BLE Services")?;
    
    let characteristics = device.characteristics();

    info!("BLE Services are discovered.");

    // Getting the CUSTOM COMMAND service characteristic
    let Some(chr_cust_cmd) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw::CHR_CUSTOM_CMD} ) 
    else {
        return Err(
            anyhow!(
                "The CUSTOM_COMMAND characteristic ({}) cannot be found on the device...", 
                abw::CHR_CUSTOM_CMD.as_hyphenated()
            )
        );
    };

    // Set BLE connection to 'Very Fast'!
    let _res = device.write(chr_cust_cmd, &vec![abw::WR_VERY_FAST_CONN], WriteType::WithResponse)
        .await.with_context(||"couldn't set BLE connection speed to Very Fast")?;

    let res = device.read(chr_cust_cmd)
        .await.with_context(||"couldn't read the result of setting BLE connection speed to Very Fast")?;
    if res.is_empty() || (res[0] != abw::WR_VERY_FAST_CONN) { // Failure value would be 0xaa
        return Err(
            anyhow!(
                "BLE Connection Speed was not set to Very Fast.", 
            )
        );
    }

    info!("BLE connection is set to 'Very Fast'!");


    // ********************************************************************************
    // *** START of Visitor Pattern
    // ********************************************************************************

    println!("Importing device configuration...");
    match import_config_visitor(&mut file, &device).await {
        Ok(_) => {
            println!("");
            println!("Device configuration has been imported ");
        },
        Err(e) => {
            println!("");
            error!("{}", e); debug!("{:?}", e);
        }
    }

    // ********************************************************************************
    // *** END of Visitor Pattern
    // ********************************************************************************


    // Set BLE connection back to 'Slow'!
    device.write(chr_cust_cmd, &vec![abw::WR_SLOW_CONN], WriteType::WithoutResponse).await
        .with_context(||"couldn't set the BLE connection speed back to Slow")?;

    info!("BLE connection is set back to 'Slow'!");

    // println!("Disconnecting...");
    // device.disconnect().await
    //     .with_context(||"error while disconnecting the device")?;
    // println!("Disconnected.");

    Ok(())

}



// ********************************************************************************
// *** The Visitor Function
// ********************************************************************************

pub async fn import_config_visitor(config_file: &mut File, device: &Peripheral) -> Result<()> {

    let characteristics = device.characteristics();

    // Getting the CONFIGURATION service characteristic
    let Some(chr_configuration) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw::CHR_CONFIGURATION} ) 
    else {
        return Err(
            anyhow!(
                "The CONFIGURATION characteristic ({}) cannot be found on the device...", 
                abw::CHR_CONFIGURATION.as_hyphenated()
            )
        );
    };

    // Subscribe to notifications
    let mut notification_stream = device.notifications().await
        .with_context(||"couldn't get BLE notification stream")?;

    device.subscribe(&chr_configuration).await
        .with_context(||"couldn't subscribe to BLE configuration notifications")?;

    let param_name_to_id = get_param_name_to_id();

    let config_file = BufReader::new(config_file);
        
    for line in config_file.lines() {

        let line = match line {
            Ok(line) => line,
            Err(e) => {
                error!("cannot read one of the lines of the configuration file: {}", e);
                debug!("{:?}", e);
                continue;
            }
        };

        // Remove comments, marked by #
        let line = line
            .split("#")
            .next()
            .expect("Cannot fail")
            .trim();
        
        if line == "" { 
            continue 
        }

        let mut words = line.split('=');

        if let (Some(param_name), Some(param_value)) = (words.next(), words.next()) {

            // If this is Some, then there were two '=' characters in the line
            if words.next().is_some() {
                warn!("Invalid line was ignored while parsing the config file: '{}'", line);
                continue;
            }

            // Parse parameter name
            let param_name = param_name.trim(); 
            let Some(param_id) = param_name_to_id.get(param_name) else {
                warn!("Invalid parameter name was ignored while parsing the config file: '{}'", line);
                continue;
            };

            // Parse parameter value
            let Ok(mut param_value) = param_value.trim().parse::<i32>() else {
                warn!("Invalid parameter value was ignored while parsing the config file: '{}'", line);
                continue;
            };




            // ********************************************
            // *** WORKAROUND for config_flags
            // ********************************************

            // It seems that bit 20 of coonfig_flags need to be 0 otherwise the process will stick...
            if param_name == "config_flags" && ( (param_value & (1<<20)) != 0 ) {
                param_value = param_value & !(1<<20);
                debug!("Bit 20 of the config_flags parameter has been temporarily set to 0 (it is needed to keep the BLE connection open).");
            }




            // If trying to set Special Parameter except the 'mode' parameter, then ignore the setting
            if (*param_id >= 245) && (param_name != "mode") {
                warn!("Setting of Special Parameter was ignored while parsing the config file: '{}'", line);
                continue;
            }

            let param_value_bytes = param_value.to_be_bytes();

            // Write 'Parameter Write Request'
            device.write(
                chr_configuration, 
                &vec![
                    abw::WR_WRITE_CONF, 
                    *param_id, 
                    param_value_bytes[0], param_value_bytes[1], param_value_bytes[2], param_value_bytes[3]
                ], 
                WriteType::WithoutResponse
            ).await
                .with_context(||format!("couldn't send the 'write config param: {:02x}' BLE command", param_id))?;





            
            // // Workaround!!!
            // if param_name == "config_flags" && ( (param_value & (1<<20)) != 0 ) {

            //     println!("HELLO!");

            //     // Getting the CUSTOM_RCV_SERIAL_DATA service characteristic
            //     let Some(chr_custom_rcv_serial_data) = characteristics
            //         .iter()
            //         .find(|chr| { chr.uuid == abw_srv::CHR_CUSTOM_RCV_SERIAL_DATA} ) 
            //     else {
            //         return Err(
            //             anyhow!(
            //                 "The CUSTOM_RCV_SERIAL_DATA characteristic ({}) cannot be found on the device...", 
            //                 abw_srv::CHR_CUSTOM_RCV_SERIAL_DATA.as_hyphenated()
            //             )
            //         );
            //     };
                
            //     if let Some(descriptor) = chr_custom_rcv_serial_data.descriptors.first() {
            //         device.write_descriptor(descriptor, &[0x00, 0x00]).await?;
            //     }

            //     // device.unsubscribe(&chr_custom_rcv_serial_data).await
            //     //     .with_context(||"couldn't disable notifications on CUSTOM_RCV_SERIAL_DATA characteristics")?;

            //     device.subscribe(&chr_configuration).await
            //         .with_context(||"couldn't subscribe to BLE configuration notifications")?;

            // }








            // Get response as value notification
            let Some(ValueNotification{uuid: _, value: param_val_vec}) = notification_stream.next().await else {
                warn!("No confirmation was received for config param write request: {}", line);
                continue;
            };

            // Evaluate the response
            match param_val_vec[0] {
                abw::NOTIF_CONF_SUCCESS => {
                    info!("Parameter sent and accepted: {}", line);
                    continue;
                }
                abw::NOTIF_CONF_INVALID => {
                    warn!("Invalid parameter value was not accepted by the device: '{}'", line);
                    continue;
                }
                _ => {
                    warn!("The device sent an invalid response to parameter set request: '{}'; Response: 0x{:02x}", &line, param_val_vec[0]);
                    continue;
                }
            }

        } else {
            warn!("A line of the configuration file cannot be read and was ignored while parsing: '{}'", &line); // Line number?
            continue;
        }

    }

    Ok(())

}