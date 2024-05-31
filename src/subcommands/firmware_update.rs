
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use anyhow::{anyhow, Result, Context};
use futures::StreamExt;
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Peripheral as _, WriteType};
use log::*;

use crate::abw_ble_utils::{
    abw,
    find_dev::find_abw_device,
};


pub async fn firmware_update (
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

    println!("Firmware update has been started.");
    match firmware_update_visitor(&mut file, &device).await {
        Ok(_) => {
            println!("");
            println!("Firmware has been successfully Updated.");
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

pub async fn firmware_update_visitor(firmware_file: &mut File, device: &Peripheral) -> Result<()> {

    let characteristics = device.characteristics();

    let Some(chr_serial_number) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw::CHR_SERIAL_NUMBER} ) 
    else {
        return Err(
            anyhow!(
                "The SERIAL_NUMBER characteristic ({}) cannot be found on the device...", 
                abw::CHR_SERIAL_NUMBER.as_hyphenated()
            )
        )
    };

    let dev_eui_vec = device.read(chr_serial_number).await
        .with_context(||"error while reading the SERIAL_NUMBER/DevEUI of the device")?;

    info!("DevEUI found: {}", hex::encode(&dev_eui_vec));

    // Getting the CUSTOM MCU FW UPDATE service characteristic
    let Some(chr_cust_mcu_fw_update) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw::CHR_CUSTOM_MCU_FW_UPDATE } ) 
    else {
        return Err(
            anyhow!(
                "The CUSTOM MCU FW UPDATE characteristic ({}) cannot be found on the device...", 
                abw::CHR_CONFIGURATION.as_hyphenated()
            )
        );
    };

    // Subscribe to notifications
    let mut notification_stream = device.notifications().await
        .with_context(||"couldn't get BLE notification stream")?;

    device.subscribe(&chr_cust_mcu_fw_update).await
        .with_context(||"couldn't subscribe to CUSTOM_MCU_FW_UPDATE notifications")?;

    let mut enable_fw_update_data: Vec<u8> = Vec::with_capacity(9);
    enable_fw_update_data.push(abw::WR_ENABLE_DFU);
    // enable_fw_update_data.extend_from_slice(&dev_eui.to_be_bytes()); 
    enable_fw_update_data.extend_from_slice(&dev_eui_vec);
    let _res = device.write(chr_cust_mcu_fw_update, &enable_fw_update_data, WriteType::WithResponse)
        .await.with_context(||"couldn't enable firmware update")?;
    let Some(notif) = notification_stream.next().await else {
        return Err(
            anyhow!(
                "No notification came back as a response to Enable Firmware Update over BLE.", 
            )
        );                
    };
    if notif.value.len() != 2 || notif.value[0] != abw::WR_ENABLE_DFU || notif.value[1] != 0 {
        return Err(
            // 0x0013 is sent if DevEUI is invalid
            anyhow!(
                "Didn't receive proper value notification as response to Enable Firmware Update over BLE: {:?}",
                notif.value
            )
        );           
    }

    // Begin firmware update

    let firmware_metadata = firmware_file.metadata()
        .with_context(||"error while checking meta data of firmware file")?;

    let binary_size = firmware_metadata.len() as u32;

    let mut start_fw_update_data: Vec<u8> = Vec::with_capacity(5);
    start_fw_update_data.push(abw::WR_START_DFU);
    start_fw_update_data.extend_from_slice(&binary_size.to_be_bytes()); 
    let _res = device.write(chr_cust_mcu_fw_update, &start_fw_update_data, WriteType::WithResponse)
        .await.with_context(||"couldn't begin firmware update")?;
    let Some(notif) = notification_stream.next().await else {
        return Err(
            anyhow!(
                "No notification came back as a response to Start Firmware Update over BLE.", 
            )
        );           
    };
    if notif.value.len() != 2 || notif.value[0] != abw::WR_START_DFU || notif.value[1] != 0 {
        return Err(
            anyhow!(
                "Didn't receive proper value notification as response to Start Firmware Update over BLE.", 
            )
        );           
    }

    let mut crc_state = crc16::State::<crc16::XMODEM>::new();
    let mut chunk: [u8; 16] = [0; 16];
    let mut offset: u32 = 0x000000;
    let num_of_chunks = binary_size / 16;
    let mut chunk_index: u32 = 0;
    let mut bytes_sent: usize = 0;

    {
        let mut stdout = io::stdout().lock();

        match firmware_file.read(&mut chunk[..]) {
            Ok(n) => {

                if n > 0 {

                    crc_state.update(&chunk);

                    let mut data: Vec<u8> = Vec::with_capacity(20);
                    data.push(abw::WR_WRITE_BINARY_DATA);
                    data.extend_from_slice(&offset.to_be_bytes()[1..]); 
                    data.extend_from_slice(&chunk);

                    let _res = device.write(chr_cust_mcu_fw_update, &data, WriteType::WithoutResponse)
                        .await.with_context(||"couldn't send binary data chunk")?;

                    let s = format!("  FW Chunk: {} / {}\r", 
                        chunk_index, num_of_chunks
                    );
                    let _ = stdout.write_all(s.as_bytes());
                    let _ = stdout.flush();

                    offset += 16;
                    chunk_index += 1;
                    bytes_sent += n;

                }
            },
            Err(_e) => {}
        };

        while match firmware_file.read(&mut chunk[..]) {
            Ok(n) => {

                if n > 0 {

                    crc_state.update(&chunk[..n]);

                    let mut data: Vec<u8> = Vec::with_capacity(20);
                    data.push(abw::WR_WRITE_BINARY_DATA);
                    data.extend_from_slice(&(offset).to_be_bytes()[1..]); 
                    data.extend_from_slice(&chunk[..n]);

                    let _res = device.write(chr_cust_mcu_fw_update, &data, WriteType::WithoutResponse)
                        .await.with_context(||"couldn't send binary data chunk")?;

                    let Some(notif) = notification_stream.next().await else {
                        return Err(
                            anyhow!(
                                "No notification came back as a response to a Write Binary Data chunk.", 
                            )
                        );           
                    };
                    if (notif.value.len() != 5) || 
                        (notif.value[0] != abw::WR_WRITE_BINARY_DATA) || 
                        (notif.value[1]!=abw::FW_UPDATE_COMPLETED_SUCCESSFULLY) 
                    {
                        if notif.value.len() >= 2 && notif.value[1] <= 0x0f {
                            return Err(
                                anyhow!(
                                    "Error recevied as response to Write Binary Data chunk: {}",    
                                    abw::FW_ERRORS[notif.value[1] as usize]
                                )
                            );           
                        } else {
                            return Err(
                                anyhow!(
                                    "Didn't receive proper value notification as response to a Write Binary Data chunk: {:?}",
                                    notif.value
                                )
                            );
                        }
                    }

                    let expected_offset = u32::from_be_bytes([0, notif.value[2], notif.value[3], notif.value[4]]);
                    if expected_offset != offset {
                        return Err(
                            anyhow!(
                                "The next expected chunk offset is {}, while the sent one was {}.",
                                expected_offset,
                                offset
                            )
                        );                        
                    }
                    
                    let s = format!("  FW Chunk: {} / {}\r", 
                        chunk_index, num_of_chunks
                    );
                    let _ = stdout.write_all(s.as_bytes());
                    let _ = stdout.flush();

                    offset += 16;
                    chunk_index += 1;
                    bytes_sent += n;

                    true

                } else {
                    false
                }

            },
            Err(_e) => {
                false
            }
        } {};
        
        // Notification for the last chunk
        let Some(notif) = notification_stream.next().await else {
            return Err(
                anyhow!(
                    "No notification came back as a response to a Write Binary Data chunk.", 
                )
            );           
        };
        if (notif.value.len() != 5) || 
            (notif.value[0] != abw::WR_WRITE_BINARY_DATA) || 
            (notif.value[1]!=abw::FW_UPDATE_COMPLETED_SUCCESSFULLY) 
        {
            if notif.value.len() >= 2 && notif.value[1] <= 0x0f {
                return Err(
                    anyhow!(
                        "Error recevied as response to Write Binary Data chunk: {}",
                        abw::FW_ERRORS[notif.value[1] as usize]
                    )
                );           
            } else {
                return Err(
                    anyhow!(
                        "Didn't receive proper value notification as response to a Write Binary Data chunk: {:?}",
                        notif.value
                    )
                );
            }
        }

    }

    let crc = crc_state.get();

    println!("CRC: {}", crc);
    println!("binary_size: {}", binary_size);
    println!("bytes sent: {}", bytes_sent);

    // SEND CRC
    let _res = device.write(
        chr_cust_mcu_fw_update, 
        &vec![
            abw::WR_BINARY_DATA_CRC, 
            (crc >> 8) as u8, (crc & 0xff) as u8
        ], 
        WriteType::WithResponse
    )
        .await.with_context(||"couldn't send CRC")?;


    let Some(notif) = notification_stream.next().await else {
        return Err(
            anyhow!(
                "No notification came back as a response to a Write CRC.", 
            )
        );           
    };
    if (notif.value.len() != 2) || 
        (notif.value[0] != abw::WR_BINARY_DATA_CRC) || 
        (notif.value[1] != abw::FW_UPDATE_COMPLETED_SUCCESSFULLY) 
    {
        if notif.value.len() >= 2 && notif.value[1] <= 0x0f {
            return Err(
                anyhow!(
                    "Error recevied as response to CRC: {}",    
                    abw::FW_ERRORS[notif.value[1] as usize]
                )
            );           
        } else {
            return Err(
                anyhow!(
                    "Didn't receive proper value notification as response to a Write CRC: {:?}",
                    notif.value
                )
            );
        }
    }
    
    
    // // ABORT Firmware Update
    // let _res = device.write(chr_cust_mcu_fw_update, &vec![abw_srv::WR_ABORT_DFU], WriteType::WithResponse)
    //     .await.with_context(||"couldn't abort firmware update")?;

    // let res = device.read(chr_cust_mcu_fw_update)
    //     .await.with_context(||"couldn't read the result of ABORT DFU command")?;
    // if res.is_empty() || (res[0] != abw_srv::WR_ABORT_DFU) {
    //     return Err(
    //         anyhow!(
    //             "Failed to abort Firmware Updaate over BLE.", 
    //         )
    //     );
    // }


    Ok(())

}