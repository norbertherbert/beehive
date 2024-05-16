use std::time::Duration;
use std::io::{self, Write};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use anyhow::{anyhow, Result, Context};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use futures::StreamExt;
use tokio::time;

use btleplug::platform::Manager;
use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType};

use clap::{arg, command, ArgGroup};
use pretty_env_logger;
use log::*;


use beehive::abw_ble::{
    abw_srv,
    find_dev::{find_abw_device_names, find_abw_device},
    discover_srv::discover_chars,
};





#[tokio::main]
async fn main() -> Result<()> {


    // Defining Command Line Options
    let cli_arg_matches = command!() // requires `cargo` feature
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


    // *********************************
    // CLI Argument: "--debug"
    // *********************************

    let log_level = match 1+cli_arg_matches.get_one::<u8>("debug").expect("Count's are defaulted") {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    // logger::builder()
    pretty_env_logger::formatted_builder()
        // .format(|f, record| {
        //     writeln!(f, "{}: {}", record.level(), record.args() )
        // })
        .filter_level(log_level)
        .init();


    // *********************************
    // Get the BLE Manager and Adapter
    // *********************************

    let ble_manager = Manager::new().await
        .with_context(||"cannot get BLE manager")?;
    let ble_adapters = ble_manager.adapters().await
        .with_context(||"cannot get BLE adapters")?;
    let ble_adapter = ble_adapters.into_iter().nth(0)
        .ok_or(anyhow!("no bluetooth adapters found"))?;
    match ble_adapter.adapter_info().await {
        Ok(ble_adapter_info) => {
            info!("BLE Adapter was found: {}", &ble_adapter_info);
        },
        Err(e) => {
            warn!("BLE adapter was found but cannot get adapter info: {}", e);
        }
    }


    // *********************************
    // CLI Argument: "--list"
    // *********************************

    if cli_arg_matches.get_flag("list") {

        println!("Scanning...");
        let found_abw_device_names = match find_abw_device_names(&ble_adapter).await {
            Ok(v) => v,
            Err(e) => {
                error!("error while searching for devices: {}", e);
                println!("No Abeeway devices were found.");
                println!("Make sure that the device you are looking for is advertizing and try again.");
                return Ok(())
            }
        };
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


    // *********************************
    // CLI Argument: "--show"
    // *********************************

    if let Some(selected_device) = cli_arg_matches.get_one::<String>("show") {

        println!("Scanning...");
        let device = match find_abw_device(&ble_adapter, selected_device).await {
            Ok(v) => v,
            Err(e) => {
                error!("error while searching for the selected device: {}", e);
                println!("Cannot find the selected Abeeway Device.");
                println!("Make sure that the device you are looking for is advertizing and try again.");
                return Ok(())
            }
        };

        if let Some(device) = device {

            let is_connected = match device.is_connected().await {
                Ok(v) => v,
                Err(e) => {
                    error!("error while checking if the selected device is connected: {}", e);
                    return Ok(())
                }
            };

            println!("The Device was found:\n    {} - {}", 
                selected_device, 
                if is_connected { "Connected" } else { "Not Connected" }
            );

            if !is_connected {
                println!("Connecting...");
                match device.connect().await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("error while connecting the device: {}", e);
                        println!("Device cannot be connected. A possible reason of this error is that pairing is corrupted.");
                        println!("You can fix this in two steps:");
                        println!("    1. Remove the BLE bond on your OS by using your OS's GUI");
                        println!("    2. Remove the BLE bond on your device by executing the following command:");
                        println!("        abeehive --unpair <DEVICE>");
                        return Ok(())
                    }
                };
            } 
            println!("Connected.");
            println!("Discovering BLE service characteristics...");
            let device_chars = match discover_chars(&device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error while discovering device characteristics: {}", e);
                    Default::default()
                }
            };
            println!("BLE service characteristics discovered.");


            println!("Verifying the existence and validity of existing pairing.");
            // workaround to test if device is paired
            if let Some(ref chr_conf) = device_chars.configuration {
                match device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithResponse).await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("error while writing the CONFIGURATION caracteristic of the device: {}", e);
                        print!  ("It seems that your device is not paired while the requested action requires authentication. ");
                        println!("Please pair your device using your OS's GUI and try again. ");
                        print!  ("The device may have an old bond to this or another computer. In such a case the OS will not find the device when you try to add. ");
                        println!("You can fix this by executing the following command: ");
                        println!("    beehive --unpair <DEVICE> ");
                        return Ok(())
                    }
                }
            }
            println!("Peering verified.");






            



            if let Some(ref c) = device_chars.custom_cmd {
                // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                device.write(c, &vec![0x02], WriteType::WithoutResponse).await?;
                info!("Connection is set to Very Fast!");
            }

            info!("Retreiving device details...");

            println!("Device details:");

            if let Some(ref c) = device_chars.model_number {

                let v = match device.read(c).await { 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the MODEL_NUMBER of the device: {}", e);
                        return Ok(())
                    }
                };

                let s = String::from_utf8_lossy(&v);
                println!("    Model Number: {}", s);

            }

            if let Some(ref c) = device_chars.serial_number {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the SERIAL_NUMBER of the device: {}", e);
                        return Ok(())
                    }
                };
                let s = hex::encode(&v);
                println!("    Serial Number: {}", s);
            }

            if let Some(ref c) = device_chars.firmware_revision {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the FIRMWARE_REVISION of the device: {}", e);
                        return Ok(())
                    }
                };
                let s = String::from_utf8_lossy(&v);
                println!("    Firmware Revision: {}", s);
            }

            if let Some(ref c) = device_chars.software_revision {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the SOFTWARE_REVISION of the device: {}", e);
                        return Ok(())
                    }
                };
                let s = String::from_utf8_lossy(&v);
                println!("    Software Revision: {}", s);
            }

            if let Some(ref c) = device_chars.manufacturer_name {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the MANUFACTURER_NAME of the device: {}", e);
                        return Ok(())
                    }
                };
                let s = String::from_utf8_lossy(&v);
                println!("    Manufacturer Name: {}", s);
            }

            if let Some(ref c) = device_chars.tx_power_level {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the TX_POWER_LEVEL of the device: {}", e);
                        return Ok(())
                    }
                };
                // let s = String::from_utf8_lossy(&v);
                println!("    TX Power Level: {}", v[0]);
            }

            if let Some(ref c) = device_chars.battery_level {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the BATTERY_LEVEL of the device: {}", e);
                        return Ok(())
                    }
                };
                println!("    Battery Level: {}%", v[0]);
            }

            if let Some(ref c) = device_chars.battery_state {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the BATTERY_STATE of the device: {}", e);
                        return Ok(())
                    }
                };
                let s = match v[0] {
                    0x77 => "Charger present and charging.",
                    0x67 => "Charger present but not charging.",
                    0x66 => "Charger not present and discharging.",
                    _ => "Unknown"
                };
                println!("    Battery Power State: {}", s);
            }

            if let Some(ref c) = device_chars.temperature_celsius {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the TEMPERATURE_CELSIUS value of the device: {}", e);
                        return Ok(())
                    }
                };
                let t = u16::from_le_bytes([v[0], v[1]]) as f32 / 10_f32;
                println!("    Temperature: {} C", t);
            }

            if let Some(ref c) = device_chars.alert_level {
                let v = match device.read(c).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while reading the ALERT_LEVEL of the device: {}", e);
                        return Ok(())
                    }
                };
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
                match device.write(c, &vec![0x00], WriteType::WithoutResponse).await{ 
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while setting the BLE connection speed back to 'Fast': {}", e);
                        return Ok(())
                    }
                };
                info!("BLE connection is set back to 'Fast'!");
            }

            println!("Disconnecting...");
            match device.disconnect().await { 
                Ok(v) => v,
                Err(e) => {
                    error!("error while disconnecting the device: {}", e);
                    return Ok(())
                }
            };
            println!("Disconnected.");
    
        } else {
            println!("Device {} was not found", selected_device);
            println!("Make sure that the device is advertizing and try again.")
        }

    }


    // *********************************
    // CLI Argument: "--cli"
    // *********************************

    if let Some(selected_device) = cli_arg_matches.get_one::<String>("cli") {

        println!("Scanning...");
        let device = match find_abw_device(&ble_adapter, selected_device).await {
            Ok(v) => v,
            Err(e) => {
                error!("error while searching for the selected device: {}", e);
                println!("Cannot find the selected Abeeway Device.");
                println!("Make sure that the device you are looking for is advertizing and try again.");
                return Ok(())
            }
        };

        if let Some(device) = device {

            let is_connected = match device.is_connected().await {
                Ok(v) => v,
                Err(e) => {
                    error!("error while checking if the selected device is connected: {}", e);
                    return Ok(())
                }
            };

            println!("The Device was found:\n    {} - {}", 
                selected_device, 
                if is_connected { "Connected" } else { "Not Connected" }
            );

            if !is_connected {
                println!("Connecting...");
                match device.connect().await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("error while connecting the device: {}", e);
                        println!("Device cannot be connected. A possible reason of this error is that pairing is corrupted.");
                        println!("You can fix this in two steps:");
                        println!("    1. Remove the BLE bond on your OS by using your OS's GUI");
                        println!("    2. Remove the BLE bond on your device by executing the following command:");
                        println!("        abeehive --unpair <DEVICE>");
                        return Ok(())
                    }
                };
            } 
            println!("Connected.");
            println!("Discovering BLE service characteristics...");
            let device_chars = match discover_chars(&device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error while discovering device characteristics: {}", e);
                    Default::default()
                }
            };
            println!("BLE service characteristics discovered.");


            println!("Verifying the existence and validity of existing pairing.");
            // workaround to test if device is paired
            if let Some(ref chr_conf) = device_chars.configuration {
                match device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithResponse).await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("error while writing the CONFIGURATION caracteristic of the device: {}", e);
                        print!  ("It seems that your device is not paired while the requested action requires authentication. ");
                        println!("Please pair your device using your OS's GUI and try again. ");
                        print!  ("The device may have an old bond to this or another computer. In such a case the OS will not find the device when you try to add. ");
                        println!("You can fix this by executing the following command: ");
                        println!("    beehive --unpair <DEVICE> ");
                        return Ok(())
                    }
                }
            }
            println!("Peering verified.");











            if let Some(ref chr_cmd) = device_chars.custom_send_cli_cmd {
                if let Some(ref chr_res) = device_chars.custom_rcv_serial_data {
                    if let Some(ref chr_cust_cmd) = device_chars.custom_cmd {
                        if let Some(ref chr_conf) = device_chars.configuration {

                            let mut notification_stream = device.notifications().await?;
                            let (tx_configuration, rx_configuration): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

                            let notification_task = tokio::spawn(async move {
                                while let Some(event) = notification_stream.next().await {
                                    match event.uuid {
                                        abw_srv::CHR_CUSTOM_RCV_SERIAL_DATA => {
                                            let mut stdout = io::stdout().lock();
                                            let _ = stdout.write_all(&event.value);
                                            let _ = stdout.flush();
                                        },
                                        abw_srv::CHR_CONFIGURATION => {
                                            match tx_configuration.send(event.value) {
                                                Ok(v) => v,
                                                Err(e) => {
                                                    error!("cannot send configuration notification through the selected async channel: {}", e)
                                                }
                                            }
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



                            println!("Press Ctrl+C to leave the CLI interface!");

                            // These two lines are needed as a workaround to show the loging prompt at start
                            time::sleep(Duration::from_millis(300)).await;
                            device.write(&chr_cmd, b"\r\n", WriteType::WithoutResponse).await?;





                            // Handling Ctrl-C event
                            let cli_is_running = Arc::new(AtomicBool::new(true));
                            {
                                let cli_is_running = cli_is_running.clone();
                                ctrlc::set_handler(move || {
                                    cli_is_running.store(false, Ordering::SeqCst);
                                    println!("\nCtrl+C button pressed...\nPress 'Enter' to exit!");
                                })
                                .with_context(||"error while setting Ctrl-C handler")?;
                            }

                            cli_is_running.store(true, Ordering::SeqCst);

                            while cli_is_running.load(Ordering::SeqCst) {
                                let mut input = String::new();
                                match std::io::stdin().read_line(&mut input) {
                                    Ok(_n) => {

                                        // Making sure that the strings ends with "\r\n" on both Windows and Linux
                                        let mut input = input.into_bytes();
                                        let l = input.len();
                                        if l < 2 {
                                            input = vec!['\r' as u8, '\n' as u8];
                                        }
                                        if input[l-1] != '\n' as u8 {
                                            input.push('\r' as u8);
                                            input.push('\n' as u8);
                                        } 
                                        else if input[l-2] != '\r' as u8 {
                                            input[l-1] = '\r' as u8;
                                            input.push('\n' as u8);
                                        }

                                        match device.write(
                                            &chr_cmd, 
                                            &input, 
                                            WriteType::WithResponse
                                        ).await {
                                            Ok(v) => v,
                                            Err(e) => {
                                                error!("cannot write CLI COMMAND to device: {}", e)
                                            } 
                                        }

                                    }
                                    Err(e) => {
                                        error!("cannot read line from stdin: {}", e)
                                    }

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

                            device.unsubscribe(&chr_conf).await?;

                            notification_task.abort();





                        } else {
                            return Err(anyhow!("The CONFIGURATION characteristic ({}) cannot be found on the device...", abw_srv::CHR_CONFIGURATION.as_hyphenated()));
                        }
                    } else {
                        return Err(anyhow!("The CUSTOM_COMMAND characteristic ({}) cannot be found on the device...", abw_srv::CHR_CUSTOM_CMD.as_hyphenated()));
                    } 
                } else {
                    return Err(anyhow!("The CUSTOM_RECEIVE_SERIAL_DATA characteristic ({}) cannot be found on the device...", abw_srv::CHR_CUSTOM_RCV_SERIAL_DATA.as_hyphenated()));
                }
            } else {
                return Err(anyhow!("The CUSTOM_SEND_CLI_COMMAND characteristic ({}) cannot be found on the device...", abw_srv::CHR_CUSTOM_SEND_CLI_CMD.as_hyphenated()));
            }


            println!("Disconnecting...");
            match device.disconnect().await { 
                Ok(v) => v,
                Err(e) => {
                    error!("error while disconnecting the device: {}", e);
                    return Ok(())
                }
            };
            println!("Disconnected.");

        } else {
            println!("Device {} was not found", selected_device);
            println!("Make sure that the device is advertizing and try again.")
        }

    }


    // *********************************
    // CLI Argument: "--unpair"
    // *********************************

    if let Some(selected_device) = cli_arg_matches.get_one::<String>("unpair") {

        println!("Scanning...");
        let device = match find_abw_device(&ble_adapter, selected_device).await {
            Ok(v) => v,
            Err(e) => {
                error!("error while searching for the selected device: {}", e);
                println!("Cannot find the selected Abeeway Device.");
                println!("Make sure that the device you are looking for is advertizing and try again.");
                return Ok(())
            }
        };

        if let Some(device) = device {

            let is_connected = match device.is_connected().await {
                Ok(v) => v,
                Err(e) => {
                    error!("error while checking if the selected device is connected: {}", e);
                    return Ok(())
                }
            };

            println!("The Device was found:\n    {} - {}", 
                selected_device, 
                if is_connected { "Connected" } else { "Not Connected" }
            );

            if !is_connected {
                println!("Connecting...");
                match device.connect().await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("error while connecting the device: {}", e);
                        println!("Device cannot be connected. A possible reason of this error is that pairing is corrupted.");
                        println!("You can fix this in two steps:");
                        println!("    1. Remove the BLE bond on your OS by using your OS's GUI");
                        println!("    2. Remove the BLE bond on your device by executing the following command:");
                        println!("        abeehive --unpair <DEVICE>");
                        return Ok(())
                    }
                };
            } 
            println!("Connected.");
            println!("Discovering BLE service characteristics...");
            let device_chars = match discover_chars(&device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error while discovering device characteristics: {}", e);
                    Default::default()
                }
            };
            println!("BLE service characteristics discovered.");









            if let Some(ref c) = device_chars.custom_cmd {
                device.write(c, &vec![0x99], WriteType::WithoutResponse).await?;
                println!("BLE bond has been removed!");
                println!("Please make it sure that it has been removed from our computer's OS too.");
            }








            println!("Disconnecting...");
            match device.disconnect().await { 
                Ok(v) => v,
                Err(e) => {
                    error!("error while disconnecting the device: {}", e);
                    return Ok(())
                }
            };
            println!("Disconnected.");
    
        } else {
            println!("Device {} was not found", selected_device);
            println!("Make sure that the device is advertizing and try again.")
        }

    }



    Ok(())

}
