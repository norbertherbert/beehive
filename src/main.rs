use std::time::Duration;
use std::io::{self, Write};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use anyhow::{Result, anyhow};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use futures::StreamExt;
use tokio::time;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType };

use btleplug::platform::{Manager, Adapter, Peripheral};
use clap::{
    arg, command, ArgGroup
    // value_parser, ArgAction, Command
};

use beehive::abw::{self, AbwChars};

use pretty_env_logger;
use log::*;


async fn get_ble_adapter(manager: &Manager) -> Result<Adapter> {
    let adapters = manager.adapters().await?;
    adapters.into_iter().nth(0).ok_or(anyhow!("No Bluetooth adapters found"))
}

async fn find_abw_device_names(manager: &Manager) -> Result<Vec<(String, bool)>> {

    let adapter = get_ble_adapter(manager).await?;

    let adapter_info = adapter.adapter_info().await?;

    info!("BLE Adapter was found: {}", &adapter_info);
    println!("Scanning...");

    adapter.start_scan(ScanFilter::default()).await?;

    time::sleep(Duration::from_secs(5)).await;
    let mut peripherals = adapter.peripherals().await?;

    let mut found_abw_devices: Vec<(String, bool)> = Vec::new();

    for _i in 0..10 {

        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            if local_name.starts_with(abw::PERIPHERAL_NAME_MATCH_FILTER) {
                let is_connected = peripheral.is_connected().await?;
                found_abw_devices.push((local_name, is_connected));
            }
        }

        if found_abw_devices.len() > 0 { break }

        time::sleep(Duration::from_secs(1)).await;
        peripherals = adapter.peripherals().await?;

    };


    Ok(found_abw_devices)

}


async fn find_abw_device(manager: &Manager, selected_device: &String) -> Result<Option<Peripheral>> {

    let adapter = get_ble_adapter(manager).await?;

    let adapter_info = adapter.adapter_info().await?;

    info!("BLE Adapter was found: {}", &adapter_info);
    println!("Scanning...");

    adapter.start_scan(ScanFilter::default()).await?;

    let mut found_device: Option<Peripheral> = None;

    time::sleep(Duration::from_secs(5)).await;

    'outher: for _i in 0..10 {

        let peripherals = adapter.peripherals().await?;

        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            if &local_name == selected_device {
                found_device = Some(peripheral.clone());
                break 'outher;
            }
        }

        time::sleep(Duration::from_secs(1)).await;

    };

    Ok(found_device)

}


async fn discover_chars(device: &Peripheral) -> Result<AbwChars> {

    let mut device_chars: abw::AbwChars = Default::default();

    device.discover_services().await?;

    for char in device.characteristics() {
        match char.uuid {

            abw::CHR_SYSTEM_EVENT => {
                // println!("CHR_SYSTEM_EVENT");
                device_chars.system_event = Some(char);
            },
            abw::CHR_CONFIGURATION => {
                // println!("CHR_CONFIGURATION");
                device_chars.configuration = Some(char);
            },
            abw::CHR_CUSTOM_CMD => {
                // println!("CHR_CUSTOM_CMD");
                device_chars.custom_cmd = Some(char);
            },
            abw::CHR_CUSTOM_MCU_FW_UPDATE => {
                // println!("CHR_CUSTOM_MCU_FW_UPDATE");
                device_chars.custom_mcu_fw_update = Some(char);
            },
            abw::CHR_CUSTOM_SEND_CLI_CMD => {
                // println!("CHR_CUSTOM_SEND_CLI_CMD");
                device_chars.custom_send_cli_cmd = Some(char);
            },
            abw::CHR_CUSTOM_RCV_SERIAL_DATA => {
                // println!("CHR_CUSTOM_RCV_SERIAL_DATA");
                device_chars.custom_rcv_serial_data = Some(char);
            },


            abw::CHR_MODEL_NUMBER => {
                // println!("CHR_MODEL_NUMBER");
                device_chars.model_number = Some(char);
            },
            abw::CHR_SERIAL_NUMBER => {
                // println!("CHR_SERIAL_NUMBER");
                device_chars.serial_number = Some(char);
            },
            abw::CHR_FIRMWARE_REVISION => {
                // println!("CHR_FIRMWARE_REVISION");
                device_chars.firmware_revision = Some(char);
            },
            abw::CHR_SOFTWARE_REVISION => {
                // println!("CHR_SOFTWARE_REVISION");
                device_chars.software_revision = Some(char);
            },
            abw::CHR_MANUFACTURER_NAME => {
                // println!("CHR_MANUFACTURER_NAME");
                device_chars.manufacturer_name = Some(char);
            },
            abw::CHR_TX_POWER_LEVEL => {
                // println!("CHR_TX_POWER_LEVEL");
                device_chars.tx_power_level = Some(char);
            },
            abw::CHR_BATTERY_LEVEL => {
                // println!("CHR_BATTERY_LEVEL");
                device_chars.battery_level = Some(char);
            },
            abw::CHR_BATTERY_POWER_STATE => {
                // println!("CHR_BATTERY_POWER_STATE");
                device_chars.battery_state = Some(char);
            },
            abw::CHR_TEMPERATURE_CELSIUS => {
                // println!("CHR_TEMPERATURE_CELSIUS");
                device_chars.temperature_celsius = Some(char);
            },
            abw::CHR_ALERT_LEVEL => {
                // println!("CHR_ALERT_LEVEL");
                device_chars.alert_level = Some(char);
            },

            _ => {

            },

        }
    }

    Ok(device_chars)

}



#[tokio::main]
async fn main() -> Result<()> {

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nCtrl+C button pressed...\nPress 'Enter' to exit!");
    }).expect("Error setting Ctrl-C handler");


    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(
                -l --list "Lists advertizing Abeeway devices"
            )
        )
        .arg(
            arg!(
                --show <DEVICE> "Shows details of the selected device"
            )
        )
        .arg(
            arg!(
                --cli <DEVICE> "Opens a Command Line Interface for the selected device"
            )
        )
        .arg(
            arg!(
                --unpair <DEVICE> "Remove previously set BLE bond"
            )
        )
        .arg(
            arg!(
                -d --debug ... "Turn debugging information on"
            )
        )
        .group(
            ArgGroup::new("group1")
                .required(true)
                .args(["list", "cli", "show", "unpair"]),
        )
        .get_matches();

    let manager = Manager::new().await?;
   
    let log_level = match matches.get_one::<u8>("debug").expect("Count's are defaulted") {
        0 => log::LevelFilter::Warn,
        _ => log::LevelFilter::Info,
    };

    pretty_env_logger::formatted_builder().filter_level(log_level).init();

    // trace!("a trace example");
    // debug!("deboogging");
    // info!("such information");
    // warn!("o_O");
    // error!("boom");



    if matches.get_flag("list") {

        let found_abw_device_names = find_abw_device_names(&manager).await?;
        match found_abw_device_names.len() {
            0 => {
                println!("No Abeeway devices were found.");
                println!("Make sure that the device you are looking for is advertizing and try again.")
            },
            1 => {
                println!("One Abeeway device was found:\n    {} - {}", 
                    &found_abw_device_names[0].0, 
                    if found_abw_device_names[0].1 { "Connected" } else { "Not Connected" }
                );
            }
            n => {
                println!("{} Abeeway devices were found:", n);
                for device_name in found_abw_device_names {
                    println!("    {} - {}", 
                        &device_name.0, 
                        if device_name.1 { "Connected" } else { "Not Connected" }
                    );
                }
            }
        }

    } 

    if let Some(selected_device) = matches.get_one::<String>("show") {

        let device = find_abw_device(&manager, selected_device).await?;

        if let Some(device) = device {










            let is_connected = device.is_connected().await?;
            println!("The Device was found:\n    {} - {}", 
                selected_device, 
                if is_connected { "Connected" } else { "Not Connected" }
            );
            if !is_connected {
                info!("Connecting...");
                match device.connect().await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Device cannot be connected: {}", err);
                        match err {
                            btleplug::Error::NotConnected => {
                                println!("Device cannot be connected. A possible reason of this error is that pairing is corrupted.");
                                println!("You can fix this in two steps:");
                                println!("    1. Remove the BLE bond on your OS by using your OS's GUI");
                                println!("    2. Remove the BLE bond on your device by executing the following command:");
                                println!("        abeehive --unpair <DEVICE>");
                            },
                            _ => {}
                        }
                        return Ok(())
                    }
                };
            } 

            let device_chars = match discover_chars(&device).await {
                Ok(device_chars) => device_chars,
                Err(err) => {
                    error!("Error while discovering device characteristics: {:?}", err);
                    Default::default()
                }
            };

            // workaround to test if device is paired
            if let Some(ref chr_conf) = device_chars.configuration {
                match device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithResponse).await {
                    Err(err) => {
                        match err {
                            btleplug::Error::Other( ref e ) => {
                                let err_msg = "The attribute requires authentication before it can be read or written.";
                                if e.to_string().contains(err_msg) {
                                    error!("{}", err_msg);

                                    print!  ("It seems that your device is not paired while the requested action requires authentication. ");
                                    println!("Please pair your device using your OS's GUI and try again. ");
                                    print!  ("The device may have an old bond to this or another computer. In such a case the OS will not find the device when you try to add. ");
                                    println!("You can fix this by executing the following command: ");
                                    println!("    beehive --unpair <DEVICE> ");

                                    return Ok(());
                                } else {
                                    return Err(anyhow!("{}", err));
                                }
                            }
                            _ => {
                                return Err(anyhow!("{}", err));
                            }
                        }
                    }
                    _ => {}
                };
            }





            



            if let Some(ref c) = device_chars.custom_cmd {
                // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                device.write(c, &vec![0x02], WriteType::WithoutResponse).await?;
                info!("Connection is set to Very Fast!");
            }

            info!("Retreiving device details...");

            println!("Device details:");

            if let Some(ref c) = device_chars.model_number {

                let v = device.read(c).await?;

                // let v = match device.read(c).await { 
                //     Ok(v) => v,
                //     Err(err) => {
                //         match err {
                //             btleplug::Error::Other( ref e ) => {
                //                 error!("{}", e);
                //                 println!("It seems that your device is not paired.");
                //                 println!("The requested action requires authentication.");
                //                 println!("Please pair your device and try again.");
                //                 return Ok(())
                //             }
                //             _ => {
                //                 return Err(anyhow!("{}", err));
                //             }
                //         }
                //     }
                // };

                let s = String::from_utf8_lossy(&v);
                println!("    Model Number: {}", s);

            }

            if let Some(ref c) = device_chars.serial_number {
                let v = device.read(c).await?;
                let s = hex::encode(&v);
                println!("    Serial Number: {}", s);
            }

            if let Some(ref c) = device_chars.firmware_revision {
                let v = device.read(c).await?;
                let s = String::from_utf8_lossy(&v);
                println!("    Firmware Revision: {}", s);
            }

            if let Some(ref c) = device_chars.software_revision {
                let v = device.read(c).await?;
                let s = String::from_utf8_lossy(&v);
                println!("    Software Revision: {}", s);
            }

            if let Some(ref c) = device_chars.manufacturer_name {
                let v = device.read(c).await?;
                let s = String::from_utf8_lossy(&v);
                println!("    Manufacturer Name: {}", s);
            }

            if let Some(ref c) = device_chars.tx_power_level {
                let v = device.read(c).await?;
                // let s = String::from_utf8_lossy(&v);
                println!("    TX Power Level: {}", v[0]);
            }

            if let Some(ref c) = device_chars.battery_level {
                let v = device.read(c).await?;
                println!("    Battery Level: {}%", v[0]);
            }

            if let Some(ref c) = device_chars.battery_state {
                let v = device.read(c).await?;
                let s = match v[0] {
                    0x77 => "Charger present and charging.",
                    0x67 => "Charger present but not charging.",
                    0x66 => "Charger not present and discharging.",
                    _ => "Unknown"
                };
                println!("    Battery Power State: {}", s);
            }

            if let Some(ref c) = device_chars.temperature_celsius {
                let v = device.read(c).await?;
                let t = u16::from_le_bytes([v[0], v[1]]) as f32 / 10_f32;
                println!("    Temperature: {} C", t);
            }

            if let Some(ref c) = device_chars.alert_level {
                let v = device.read(c).await?;
                let s = match v[0] {
                    0x00 => "No Alert",
                    0x01 => "Mild Alert",
                    0x02 => "High Alert",
                    _ => "Unknown"
                };
                println!("    Alert Level: {}", s);
            }

            if let Some(ref c) = device_chars.custom_cmd {
                // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                device.write(c, &vec![0x00], WriteType::WithoutResponse).await?;
                info!("BLE connection is set back to 'Fast'!");
            }

            info!("Disconnecting...");
            device.disconnect().await?;
            info!("Disconnected.");
    
        } else {
            println!("Device {} was not found", selected_device);
            println!("Make sure that the device is advertizing and try again.")
        }

    }

    if let Some(selected_device) = matches.get_one::<String>("cli") {

        let device = find_abw_device(&manager, selected_device).await?;

        if let Some(device) = device {










            let is_connected = device.is_connected().await?;
            println!("The Device was found:\n    {} - {}", 
                selected_device, 
                if is_connected { "Connected" } else { "Not Connected" }
            );
            if !is_connected {
                info!("Connecting...");
                match device.connect().await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Device cannot be connected: {}", err);
                        match err {
                            btleplug::Error::NotConnected => {
                                println!("Device cannot be connected. A possible reason of this error is that pairing is corrupted.");
                                println!("You can fix this in two steps:");
                                println!("    1. Remove the BLE bond on your OS by using your OS's GUI");
                                println!("    2. Remove the BLE bond on your device by executing the following command:");
                                println!("        abeehive --unpair <DEVICE>");
                            },
                            _ => {}
                        }
                        return Ok(())
                    }
                };
            } 

            let device_chars = match discover_chars(&device).await {
                Ok(device_chars) => device_chars,
                Err(err) => {
                    error!("Error while discovering device characteristics: {:?}", err);
                    Default::default()
                }
            };

            // workaround to test if device is paired
            if let Some(ref chr_conf) = device_chars.configuration {
                match device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithResponse).await {
                    Err(err) => {
                        match err {
                            btleplug::Error::Other( ref e ) => {
                                let err_msg = "The attribute requires authentication before it can be read or written.";
                                if e.to_string().contains(err_msg) {
                                    error!("{}", err_msg);

                                    print!  ("It seems that your device is not paired while the requested action requires authentication. ");
                                    println!("Please pair your device using your OS's GUI and try again. ");
                                    print!  ("The device may have an old bond to this or another computer. In such a case the OS will not find the device when you try to add. ");
                                    println!("You can fix this by executing the following command: ");
                                    println!("    beehive --unpair <DEVICE> ");

                                    return Ok(());
                                } else {
                                    return Err(anyhow!("{}", err));
                                }
                            }
                            _ => {
                                return Err(anyhow!("{}", err));
                            }
                        }
                    }
                    _ => {}
                };
            }












            if let Some(ref chr_cmd) = device_chars.custom_send_cli_cmd {
                if let Some(ref chr_res) = device_chars.custom_rcv_serial_data {
                    if let Some(ref chr_cust_cmd) = device_chars.custom_cmd {
                        if let Some(ref chr_conf) = device_chars.configuration {

                            let mut notification_stream = device.notifications().await?;
                            let (tx_configuration, rx_configuration): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

                            tokio::spawn(async move {
                                while let Some(event) = notification_stream.next().await {
                                    match event.uuid {
                                        abw::CHR_CUSTOM_RCV_SERIAL_DATA => {
                                            let mut stdout = io::stdout().lock();
                                            let _ = stdout.write_all(&event.value);
                                            let _ = stdout.flush();
                                        },
                                        abw::CHR_CONFIGURATION => {
                                            tx_configuration.send(event.value).unwrap();
                                        },
                                        _ => {},
                                    }
                                }
                            });

                            device.subscribe(&chr_conf).await?;
                            device.subscribe(&chr_res).await?;


                            // Set BLE connection to 'Very Fast'!
                            device.write(chr_cust_cmd, &vec![0x02], WriteType::WithoutResponse).await?;
                            info!("BLE connection is set to 'Very Fast'!");
                            
                            // Send the read config_flags command
                            device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithoutResponse).await?;
                            // Receive the actual config_flags value
                            let res_value = rx_configuration.recv()?;
                            // Write new config_flags (set bit 20 to 1)
                            device.write(chr_conf, &vec![
                                0x01, 0x0d, res_value[2], res_value[3] | 1<<4, res_value[4], res_value[5]
                            ], WriteType::WithoutResponse).await?;

                            // Turn on BLE CLI
                            device.write(chr_conf, &vec![0x01, 0xf5, 0, 0, 0, 1], WriteType::WithoutResponse).await?;



                            println!("Press Ctrl+C to exit!");

                            // These two lines are needed as a workaround to show the loging prompt at start
                            time::sleep(Duration::from_millis(300)).await;
                            device.write(&chr_cmd, b"\r\n", WriteType::WithoutResponse).await?;

                            while running.load(Ordering::SeqCst) {
                                let mut input = String::new();
                                match std::io::stdin().read_line(&mut input) {
                                    Ok(_n) => {
                                        device.write(
                                            &chr_cmd, 
                                            input.as_bytes(), 
                                            WriteType::WithoutResponse
                                        ).await?;
                                    }
                                    Err(err) => error!("input error: {}", err),
                                }
                            }


                            device.unsubscribe(&chr_res).await?;

                            // Send the read config_flags command
                            device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithoutResponse).await?;
                            // Receive the actual config_flags value
                            let res_value = rx_configuration.recv()?;
                            // Write new config_flags (set bit 20 to 0)
                            device.write(chr_conf, &vec![
                                0x01, 0x0d, res_value[2], res_value[3] & !(1<<4), res_value[4], res_value[5]
                            ], WriteType::WithoutResponse).await?;
        
                            // Turn off BLE CLI
                            device.write(chr_conf, &vec![0x01, 0xf5, 0, 0, 0, 0], WriteType::WithoutResponse).await?;

                            // Set BLE connection to 'Fast'!
                            device.write(chr_cust_cmd, &vec![0x00], WriteType::WithoutResponse).await?;

                            info!("BLE connection is set back to 'Fast'!");

                        } else {
                            return Err(anyhow!("The CONFIGURATION characteristic ({}) cannot be found on the device...", abw::CHR_CONFIGURATION.as_hyphenated()));
                        }
                    } else {
                        return Err(anyhow!("The CUSTOM_COMMAND characteristic ({}) cannot be found on the device...", abw::CHR_CUSTOM_CMD.as_hyphenated()));
                    } 
                } else {
                    return Err(anyhow!("The CUSTOM_RECEIVE_SERIAL_DATA characteristic ({}) cannot be found on the device...", abw::CHR_CUSTOM_RCV_SERIAL_DATA.as_hyphenated()));
                }
            } else {
                return Err(anyhow!("The CUSTOM_SEND_CLI_COMMAND characteristic ({}) cannot be found on the device...", abw::CHR_CUSTOM_SEND_CLI_CMD.as_hyphenated()));
            }


            info!("Disconnecting...");
            device.disconnect().await?;
            info!("Disconnected.");

        } else {
            println!("Device {} was not found", selected_device);
            println!("Make sure that the device is advertizing and try again.")
        }

    }




    if let Some(selected_device) = matches.get_one::<String>("unpair") {

        let device = find_abw_device(&manager, selected_device).await?;

        if let Some(device) = device {












            let is_connected = device.is_connected().await?;
            println!("The Device was found:\n    {} - {}", 
                selected_device, 
                if is_connected { "Connected" } else { "Not Connected" }
            );
            if !is_connected {
                info!("Connecting...");
                match device.connect().await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Device cannot be connected: {}", err);
                        match err {
                            btleplug::Error::NotConnected => {
                                println!("Device cannot be connected. A possible reason of this error is that pairing is corrupted.");
                                println!("You can fix this in two steps:");
                                println!("    1. Remove the BLE bond on your OS by using your OS's GUI");
                                println!("    2. Remove the BLE bond on your device by executing the following command:");
                                println!("        abeehive --unpair <DEVICE>");
                            },
                            _ => {}
                        }
                        return Ok(())
                    }
                };
            } 

            let device_chars = match discover_chars(&device).await {
                Ok(device_chars) => device_chars,
                Err(err) => {
                    error!("Error while discovering device characteristics: {:?}", err);
                    Default::default()
                }
            };













            if let Some(ref c) = device_chars.custom_cmd {
                device.write(c, &vec![0x99], WriteType::WithoutResponse).await?;
                println!("BLE bond has been removed!");
                println!("Please make it sure that it has been removed from our computer's OS too.");
            }





            info!("Disconnecting...");
            device.disconnect().await?;
            info!("Disconnected.");
    
        } else {
            println!("Device {} was not found", selected_device);
            println!("Make sure that the device is advertizing and try again.")
        }

    }







    Ok(())

}
