
use std::fs::{File, OpenOptions};
use std::io::{
    BufWriter, 
    Write
};
use anyhow::{anyhow, Result, Context};
use futures::StreamExt;
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Peripheral as _, WriteType, ValueNotification};
use log::*;

use crate::abw_ble::{
    abw_srv,
    find_dev::find_abw_device,
};
use crate::abw_params::{
    // self, 
    // get_param_name_to_id, 
    PARAMS
};

pub async fn export_config (
    device_name: &String,
    file_path: Option<&String>,
    ble_adapter: &Adapter,
) -> Result<()>
{

    // Try to open the file
    let mut file = match file_path {
        Some(file_path) => {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file_path)
            {
                Ok(v) => Some(v),
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            Err(e).with_context(||
                                format!("The file '{}' already exists!", file_path)
                            )?
                        },
                        _ => {
                            Err(e).with_context(||
                                "The new configuration file cannot be created."
                            )?
                        }
                    }
                }
        
            }
        },
        None => None
    };

    // Looking for the specified Abeeway device
    println!("Scanning...");
    let device = match find_abw_device(&ble_adapter, device_name).await {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e); debug!("{:?}", e);
            println!("Cannot find the selected Abeeway Device.");
            println!("{}", abw_srv::FIX_FOR_NOT_ADVERTIZING);
            return Ok(())
        }
    };
    let Some(device) = device else {
        println!("Device {} was not found", device_name);
        println!("{}", abw_srv::FIX_FOR_NOT_ADVERTIZING);
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
                println!("{}", abw_srv::FIX_FOR_CORRUPTED_PAIRING);
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
        .find(|chr| { chr.uuid == abw_srv::CHR_CUSTOM_CMD} ) 
    else {
        return Err(
            anyhow!(
                "The CUSTOM_COMMAND characteristic ({}) cannot be found on the device...", 
                abw_srv::CHR_CUSTOM_CMD.as_hyphenated()
            )
        );
    };

    // Set BLE connection to 'Very Fast'!
    let _res = device.write(chr_cust_cmd, &vec![abw_srv::WR_VERY_FAST_CONN], WriteType::WithResponse)
        .await.with_context(||"couldn't set BLE connection speed to Very Fast")?;

    let res = device.read(chr_cust_cmd)
        .await.with_context(||"couldn't read the result of setting BLE connection speed to Very Fast")?;
    if res.is_empty() || (res[0] != abw_srv::WR_VERY_FAST_CONN) { // Failure value would be 0xaa
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

    println!("Exporting device configuration...");
    match export_config_visitor(&mut file, &device).await {
        Ok(_) => {
            println!("");
            println!("Device configuration has been exported ");
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
    device.write(chr_cust_cmd, &vec![abw_srv::WR_SLOW_CONN], WriteType::WithoutResponse).await
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

pub async fn export_config_visitor(config_file: &mut Option<File>, device: &Peripheral) -> Result<()> {

    let characteristics = device.characteristics();

    // Getting the CONFIGURATION service characteristic
    let Some(chr_configuration) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_CONFIGURATION} ) 
    else {
        return Err(
            anyhow!(
                "The CONFIGURATION characteristic ({}) cannot be found on the device...", 
                abw_srv::CHR_CONFIGURATION.as_hyphenated()
            )
        );
    };

    // Subscribe to notifications
    let mut notification_stream = device.notifications().await
        .with_context(||"couldn't get BLE notification stream")?;

    device.subscribe(&chr_configuration).await
        .with_context(||"couldn't subscribe to BLE configuration notifications")?;


    match config_file {
        Some(config_file) => {

            let mut config_file = BufWriter::new(config_file);

            for param in PARAMS {

                // Write 'Parameter Read Request'
                device.write(
                    chr_configuration, 
                    &vec![
                        abw_srv::WR_READ_CONF, 
                        param.1
                    ], 
                    WriteType::WithoutResponse
                ).await
                    .with_context(||format!("couldn't send the 'read config param: 0x{:02x}' BLE command", param.1))?;


                // Get response as value notification
                // let param_val_vec = rx_configuration.recv()?;
                let Some(ValueNotification{uuid: _, value: param_val_vec}) = notification_stream.next().await else {
                    warn!("No confirmation was received for read request of param '{}',", param.0);
                    continue;
                };
                
                let param_val = i32::from_be_bytes([param_val_vec[2], param_val_vec[3], param_val_vec[4], param_val_vec[5]]);
                
                let line = format!("{} = {}", param.0, param_val);

                config_file.write(&line.as_bytes())?;
                config_file.write(b"\r\n")?;
                info!("Config param received: {}", &line);

            }
            config_file.flush()?;

        },
        None => {

            for param in PARAMS {

                device.write(
                    chr_configuration, 
                    &vec![
                        abw_srv::WR_READ_CONF, 
                        param.1
                    ], 
                    WriteType::WithoutResponse
                ).await
                    .with_context(||format!("couldn't send the 'read config param: 0x{:02x}' BLE command", param.1))?;

                // Get response as value notification
                // let param_val_vec = rx_configuration.recv()?;
                let Some(ValueNotification{uuid: _, value: param_val_vec}) = notification_stream.next().await else {
                    warn!("No confirmation was received for read request of param '{}',", param.0);
                    continue;
                };

                let param_val = i32::from_be_bytes([param_val_vec[2], param_val_vec[3], param_val_vec[4], param_val_vec[5]]);
                
                println!("{} = {}", param.0, param_val);

            }

        }
    }

    Ok(())

}