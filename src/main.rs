use std::fs::OpenOptions;
// use std::path::PathBuf;
use std::time::Duration;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use anyhow::{anyhow, Result, Context};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use futures::StreamExt;
use tokio::time;

use btleplug::platform::Manager;
use btleplug::api::{Central, Manager as _, Peripheral as _, ValueNotification, WriteType};

use clap::{arg, command, Command};

#[macro_use] extern crate log;

use beehive::abw_ble::{
    abw_srv,
    find_dev::{find_abw_device_names, find_abw_device},
    discover_srv::discover_chars,
};
use beehive::abw_params::{
    get_param_name_to_id, 
    // get_param_id_to_name, 
    PARAMS
};

const FIX_FOR_NOT_ADVERTIZING: &'static str  = "
Make sure that the device you are looking for is advertizing and try again.
- Abeeway Smart Badges and Microtrackers will start advertizing for a few min right after they are turned on.
- Abeeway Compact trackers will start advertizing after placing and removing a magnet at their marked sides multiple times.
";


const FIX_FOR_CORRUPTED_PAIRING: &'static str  = "
Device cannot be connected. It is either not advertizing or its pairing is corrupted.
- Abeeway Smart Badges and Microtrackers will start advertizing for a few min right after they are turned on.
- Abeeway Compact trackers will start advertizing after placing and removing a magnet at their marked sides multiple times.
You can fix corrupted pairing in the following way:
1.   Turn OFF and then ON Bluetooth on your computer. 
1.1      Try to connect your device again. If this does not fix your issue, contunie with the next step. 
2.   Make a new pairing for your device: 
2.1.   Remove the BLE bond on your OS by using your OS's GUI
2.2    Make sure your device is advertizing
2.3.   Remove the BLE bond on your device by executing the following command:
           abeehive --unpair <DEVICE>
2.4    Pair your device with your computer again.
";

const FIX_FOR_NOT_PAIRED: &'static str  = "
It seems that your device is not paired while the requested action requires authentication.
Please pair your device using your OS's GUI and try again.
The device may have an old bond to this or another computer. In such a case the OS will not find the device when you try to add.
You can fix this by executing the following command
    beehive --unpair <DEVICE>
";



#[tokio::main]
async fn main() -> Result<()> {


    // Defining Command Line Options
    let cli_arg_matches = command!() // requires `cargo` feature
        .arg_required_else_help(true)
        .arg(
            arg!(
                -v --verbose ... "Show logs for debugging (-v|-vv|-vvv)"
            )
        )
        .subcommand(
            Command::new("scan")
                .about("Scan for Abeeway devices.")
        )
        .subcommand(
            Command::new("show")
                .about("Show device details.")
                .arg(
                    arg!(
                        [device] "Device name."
                    )
                    .required(true)
                )
        )
        .subcommand(
            Command::new("cli")
                .about("Open Command Line Interface.")
                .arg(
                    arg!(
                        [device] "Device name."
                    )
                    .required(true)
                )
        )
        .subcommand(
            Command::new("remove-bond")
                .about("Remove BLE bond.")
                .arg(
                    arg!(
                        [device] "Device name."
                    )
                    .required(true)
                )
        )
        .subcommand(
            Command::new("export-config")
                .about("Export configuration.")
                .arg(
                    arg!(
                        [device] "The device to export configuration from."
                    )
                    .required(true)
                )
                .arg(
                    arg!(
                        -f --file  <FILE> "The file to export configuration to."
                    )
                    .required(false)
                )
        )
        .subcommand(
            Command::new("import-config")
                .about("Import configuration.")
                .arg(
                    arg!(
                        [device] "The device to import configuration to."
                    )
                    .required(true)
                )
                .arg(
                    arg!(
                        -f --file  <FILE> "The file to import configuration from."
                    )
                    .required(true)
                )
        )
        .subcommand(
            Command::new("firmware-upgrade")
                .about("COMMING SOON - Upgrade MCU firmware.")
                .arg(
                    arg!(
                        [device] "The device to import configuration to."
                    )
                    .required(true)
                )
                .arg(
                    arg!(
                        -f --file <FILE> "The file to import configuration from."
                    )
                    .required(true)
                )
        )
        .get_matches();


    // *********************************
    // CLI Argument: "--verbose"
    // *********************************

    let log_level = match cli_arg_matches.get_one::<u8>("verbose").with_context(||"couldn't get cli args")? {
        // 0 => log::LevelFilter::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::builder()
    // pretty_env_logger::formatted_builder()
        .format(|f, record| {
            writeln!(f, "{}: {}", record.level(), record.args() )
        })
        .filter(Some("beehive"), log_level)
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
    // CLI Argument: "scan"
    // *********************************

    if let Some(_sub_cmd_matches) = cli_arg_matches.subcommand_matches("scan") {

        println!("Scanning...");
        let found_abw_device_names = match find_abw_device_names(&ble_adapter).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e); debug!("{:?}", e);
                println!("No Abeeway devices were found.");
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
                return Ok(())
            }
        };
        match found_abw_device_names.len() {
            0 => {
                println!("No Abeeway devices were found.");
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
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

        return Ok(());

    } 

    
    
    // *********************************
    // CLI Argument: "show"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("show") {
        if let Some(selected_device) = sub_cmd_matches.get_one::<String>("device") {

            println!("Scanning...");
            let device = match find_abw_device(&ble_adapter, selected_device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                    println!("Cannot find the selected Abeeway Device.");
                    println!("{}", FIX_FOR_NOT_ADVERTIZING);
                    return Ok(())
                }
            };

            if let Some(device) = device {

                let is_connected = device.is_connected().await
                    .with_context(||"error while checking if the selected device is connected")?;

                println!("The Device was found:\n    {} - {}", 
                    selected_device, 
                    if is_connected { "Connected" } else { "Not Connected" }
                );

                if !is_connected {
                    println!("Connecting...");
                    match device.connect().await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", e); debug!("{:?}", e);
                            println!("{}", FIX_FOR_CORRUPTED_PAIRING);
                            return Ok(())
                        }
                    };
                } 
                println!("Connected.");
                info!("Discovering BLE service characteristics...");

                let device_chars = discover_chars(&device).await
                    .with_context(||"error while discovering device characteristics")?;
                
                info!("BLE service characteristics discovered.");

                info!("Verifying the existence and validity of existing pairing.");
                

                // workaround to test if device is paired
                if let Some(ref chr_conf) = device_chars.configuration {
                    match device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithResponse).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", e); debug!("{:?}", e);
                            println!("{}", FIX_FOR_NOT_PAIRED);
                            return Ok(())
                        }
                    }
                }
                info!("Paring verified.");






                



                if let Some(ref c) = device_chars.custom_cmd {
                    // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                    device.write(c, &vec![0x02], WriteType::WithoutResponse).await
                        .with_context(||"couldn't set BLE connection speed to Very Fast")?;
                    info!("Connection is set to Very Fast!");
                }

                info!("Retreiving device details...");

                println!("Device details:");

                if let Some(ref c) = device_chars.model_number {

                    let v = device.read(c).await
                        .with_context(||"error while reading the MODEL_NUMBER of the device")?;

                    let s = String::from_utf8_lossy(&v);
                    println!("    Model Number: {}", s);

                }

                if let Some(ref c) = device_chars.serial_number {

                    let v = device.read(c).await
                        .with_context(||"error while reading the SERIAL_NUMBER of the device")?;

                    let s = hex::encode(&v);
                    println!("    Serial Number: {}", s);
                }

                if let Some(ref c) = device_chars.firmware_revision {

                    let v = device.read(c).await
                        .with_context(||"error while reading the FIRMWARE_REVISION of the device")?;

                    let s = String::from_utf8_lossy(&v);
                    println!("    Firmware Revision: {}", s);
                }

                if let Some(ref c) = device_chars.software_revision {

                    let v = device.read(c).await
                        .with_context(||"error while reading the SOFTWARE_REVISION of the device")?;
                    
                    let s = String::from_utf8_lossy(&v);
                    println!("    Software Revision: {}", s);
                }

                if let Some(ref c) = device_chars.manufacturer_name {

                    let v = device.read(c).await
                        .with_context(||"error while reading the MANUFACTURER_NAME of the device")?;

                    let s = String::from_utf8_lossy(&v);
                    println!("    Manufacturer Name: {}", s);
                }

                if let Some(ref c) = device_chars.tx_power_level {

                    let v = device.read(c).await
                        .with_context(||"error while reading the TX_POWER_LEVEL of the device")?;
                    
                    // let s = String::from_utf8_lossy(&v);
                    println!("    TX Power Level: {}", v[0]);
                }

                if let Some(ref c) = device_chars.battery_level {

                    let v = device.read(c).await
                        .with_context(||"error while reading the BATTERY_LEVEL of the device")?;
                    
                    println!("    Battery Level: {}%", v[0]);
                }

                if let Some(ref c) = device_chars.battery_state {
                    
                    let v = device.read(c).await
                        .with_context(||"error while reading the BATTERY_STATE of the device")?;
                    
                    let s = match v[0] {
                        0x77 => "Charger present and charging.",
                        0x67 => "Charger present but not charging.",
                        0x66 => "Charger not present and discharging.",
                        _ => "Unknown"
                    };
                    println!("    Battery Power State: {}", s);
                }

                if let Some(ref c) = device_chars.temperature_celsius {
                    let v = device.read(c).await
                        .with_context(||"error while reading the TEMPERATURE_CELSIUS value of the device")?;

                    let t = u16::from_le_bytes([v[0], v[1]]) as f32 / 10_f32;
                    println!("    Temperature: {} C", t);
                }

                if let Some(ref c) = device_chars.alert_level {

                    let v = device.read(c).await
                        .with_context(||"error while reading the ALERT_LEVEL of the device")?;

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
                    device.write(c, &vec![0x01], WriteType::WithoutResponse).await
                        .with_context(||"error while setting the BLE connection speed back to 'Slow'")?;

                    info!("BLE connection is set back to 'Slow'!");
                }

                // println!("Disconnecting...");
                // device.disconnect().await
                //     .with_context(||"error while disconnecting the device")?;
                // println!("Disconnected.");
            } else {
                println!("Device {} was not found", selected_device);
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
            }

        }

        return Ok(());

    }



    // *********************************
    // CLI Argument: "cli"
    // *********************************


    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("cli") {
        if let Some(selected_device) = sub_cmd_matches.get_one::<String>("device") {

            println!("Scanning...");
            let device = match find_abw_device(&ble_adapter, selected_device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                    println!("Cannot find the selected Abeeway Device.");
                    println!("{}", FIX_FOR_NOT_ADVERTIZING);
                    return Ok(())
                }
            };

            if let Some(device) = device {

                let is_connected = device.is_connected().await
                    .with_context(||"error while checking if the selected device is connected")?;

                println!("The Device was found:\n    {} - {}", 
                    selected_device, 
                    if is_connected { "Connected" } else { "Not Connected" }
                );

                if !is_connected {
                    println!("Connecting...");
                    match device.connect().await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", e); debug!("{:?}", e);
                            println!("{}", FIX_FOR_CORRUPTED_PAIRING);
                            return Ok(())
                        }
                    };
                } 
                println!("Connected.");
                info!("Discovering BLE service characteristics...");

                let device_chars =  discover_chars(&device).await
                    .with_context(||"error while discovering device characteristics")?;

                info!("BLE service characteristics discovered.");




                if let Some(ref chr_cmd) = device_chars.custom_send_cli_cmd {
                    if let Some(ref chr_res) = device_chars.custom_rcv_serial_data {
                        if let Some(ref chr_cust_cmd) = device_chars.custom_cmd {
                            if let Some(ref chr_conf) = device_chars.configuration {

                                let mut notification_stream = device.notifications().await
                                    .with_context(||"couldn't get BLE notification stream")?;

                                device.subscribe(&chr_conf).await
                                    .with_context(||"couldn't subscribe to BLE configuration notifications")?;
                                device.subscribe(&chr_res).await
                                    .with_context(||"couldn't subscribe to CLI command responses")?;

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
                                                        error!("cannot send configuration notification through the selected async channel: {}", e);
                                                        debug!("{:?}", e);
                                                    }
                                                }
                                            },
                                            _ => {},
                                        }
                                    }
                                });
                                // // clean the channel before start
                                // let mut garbage = rx_configuration.try_iter();
                                // while let Some(_) = garbage.next() {}; 


                                info!("Verifying the existence and validity of existing pairing.");
                                // new workaround to test if device is paired (request a parameter value)
                                match device.write(chr_conf, &vec![0x00, 0x00], WriteType::WithResponse).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!("{}", e); debug!("{:?}", e);
                                        println!("{}", FIX_FOR_NOT_PAIRED);
                                        return Ok(())
                                    }
                                }
                                // consuming the response
                                rx_configuration.recv()
                                    .with_context(||"Cannot read the configuration notification channel.")?;
                                info!("Paring verified.");


                                // Set BLE connection to 'Very Fast'!
                                // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                                device.write(chr_cust_cmd, &vec![0x02], WriteType::WithoutResponse)
                                    .await.with_context(||"couldn't set BLE connection speed to Very Fast")?;
                                info!("BLE connection is set to 'Very Fast'!");
                                

                                // Send the read config_flags command
                                device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithoutResponse).await
                                    .with_context(||"couldn't send the 'read config_flags' BLE commaand")?;
                                // Receive the actual config_flags value
                                let res_value = rx_configuration.recv()
                                    .with_context(||"Cannot read the response to read config_flags command from the notification channel.")?;
                                // Check if BLE CLI is enabled in config_flags. Check if bit 4 (20) is turned on.
                                if res_value[3] & 1<<4 == 0 {
                                    // Enable BLE CLI in config_flags. Write new config_flags (set bit 20 to 1).
                                    device.write(chr_conf, &vec![
                                        0x01, 0x0d, res_value[2], res_value[3] | 1<<4, res_value[4], res_value[5]
                                    ], WriteType::WithoutResponse).await
                                        .with_context(||"couldn't send the 'write config_flags' BLE commaand")?;
                                    info!("BLE CLI (bit 20) has been enabled in config_flags.");
                                } else {
                                    info!("BLE CLI (bit 20) is already enabled in config_flags.");
                                }

                                // Turn on BLE CLI
                                device.write(chr_conf, &vec![0x01, 0xf5, 0, 0, 0, 1], WriteType::WithoutResponse).await
                                    .with_context(||"couldn't send the 'Turn on BLE CLI' commaand")?;



                                println!("Press Ctrl+C to leave the CLI interface!");

                                // These two lines are needed as a workaround to show the loging prompt at start
                                time::sleep(Duration::from_millis(300)).await;
                                device.write(&chr_cmd, b"\r\n", WriteType::WithoutResponse).await
                                    .with_context(||"couldn't send the first empty CLI command")?;





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
                                                    error!("cannot write CLI COMMAND to device: {}", e);
                                                    debug!("{:?}", e);
                                                } 
                                            }

                                        }
                                        Err(e) => {
                                            error!("cannot read line from stdin: {}", e);
                                            debug!("{:?}", e);
                                        }

                                    }
                                }

                                device.unsubscribe(&chr_res).await
                                    .with_context(||"couldn't unsubscribe from CLI command responses")?;

                                // // Disable BLE CLI in config flags
                                // // Send the read config_flags command
                                // device.write(chr_conf, &vec![0x00, 0x0d], WriteType::WithoutResponse).await
                                //     .with_context(||"couldn't send the 'read config_flags' command")?;
                                // // Receive the actual config_flags value
                                // let res_value = rx_configuration.recv()
                                //     .with_context(||"couldn't receive the config_flags parameter from async channel")?;
                                // // Write new config_flags (set bit 20 to 0)
                                // device.write(chr_conf, &vec![
                                //     0x01, 0x0d, res_value[2], res_value[3] & !(1<<4), res_value[4], res_value[5]
                                // ], WriteType::WithoutResponse).await
                                //     .with_context(||"coulddn't write the new config_flags value")?;
            
                                // Turn off BLE CLI
                                device.write(chr_conf, &vec![0x01, 0xf5, 0, 0, 0, 0], WriteType::WithoutResponse).await
                                    .with_context(||"couldn't send the 'turn off BLE CLI' command")?;

                                // Set BLE connection back to 'Slow'!
                                // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                                device.write(chr_cust_cmd, &vec![0x01], WriteType::WithoutResponse).await
                                    .with_context(||"couldn't set the BLE connection speed back to Slow")?;

                                info!("BLE connection is set back to 'Slow'!");

                                device.unsubscribe(&chr_conf).await
                                    .with_context(||"couldn't unsubscribe from BLE configuration responses")?;

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


                // println!("Disconnecting...");
                // device.disconnect().await
                //     .with_context(||"error while disconnecting the device")?;
                // println!("Disconnected.");
            } else {
                println!("Device {} was not found", selected_device);
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
            }

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "remove-bond"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("remove-bond") {
        if let Some(selected_device) = sub_cmd_matches.get_one::<String>("device") {

            println!("Scanning...");
            let device = match find_abw_device(&ble_adapter, selected_device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                    println!("Cannot find the selected Abeeway Device.");
                    println!("{}", FIX_FOR_NOT_ADVERTIZING);
                    return Ok(())
                }
            };

            if let Some(device) = device {

                let is_connected = device.is_connected().await
                    .with_context(||"error while checking if the selected device is connected")?;

                println!("The Device was found:\n    {} - {}", 
                    selected_device, 
                    if is_connected { "Connected" } else { "Not Connected" }
                );

                if !is_connected {
                    println!("Connecting...");
                    match device.connect().await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", e); debug!("{:?}", e);
                            println!("{}", FIX_FOR_CORRUPTED_PAIRING);
                            return Ok(())
                        }
                    };
                } 
                println!("Connected.");
                info!("Discovering BLE service characteristics...");
                let device_chars = match discover_chars(&device).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("error while discovering device characteristics: {}", e);
                        debug!("{:?}", e);
                        Default::default()
                    }
                };
                info!("BLE service characteristics discovered.");








                // Remove thee BLE bond
                if let Some(ref c) = device_chars.custom_cmd {
                    device.write(c, &vec![0x99], WriteType::WithoutResponse).await
                        .with_context(||"couldn't remove BLE bond")?;
                    println!("BLE bond has been removed!");
                    println!("Please make it sure that it has been removed from our computer's OS too.");
                }








                // println!("Disconnecting...");
                // device.disconnect().await
                //     .with_context(||"error while disconnecting the device")?;
                // println!("Disconnected.");
            } else {
                println!("Device {} was not found", selected_device);
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
            }

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "import-config"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("import-config") {
        if let (Some(selected_device), Some(config_path)) = 
            (sub_cmd_matches.get_one::<String>("device"), sub_cmd_matches.get_one::<String>("file")) 
        {

            println!("Scanning...");
            let device = match find_abw_device(&ble_adapter, selected_device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                    println!("Cannot find the selected Abeeway Device.");
                    println!("{}", FIX_FOR_NOT_ADVERTIZING);
                    return Ok(())
                }
            };

            if let Some(device) = device {

                let is_connected = device.is_connected().await
                    .with_context(||"error while checking if the selected device is connected")?;

                println!("The Device was found:\n    {} - {}", 
                    selected_device, 
                    if is_connected { "Connected" } else { "Not Connected" }
                );

                if !is_connected {
                    println!("Connecting...");
                    match device.connect().await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", e); debug!("{:?}", e);
                            println!("{}", FIX_FOR_CORRUPTED_PAIRING);
                            return Ok(())
                        }
                    };
                } 
                println!("Connected.");
                info!("Discovering BLE service characteristics...");

                let device_chars =  discover_chars(&device).await
                    .with_context(||"error while discovering device characteristics")?;

                info!("BLE service characteristics discovered.");

                if let Some(ref chr_cust_cmd) = device_chars.custom_cmd {
                    if let Some(ref chr_conf) = device_chars.configuration {

                        let mut notification_stream = device.notifications().await
                            .with_context(||"couldn't get BLE notification stream")?;
                        
                        device.subscribe(&chr_conf).await
                            .with_context(||"couldn't subscribe to BLE configuration notifications")?;


                        // let (tx_configuration, rx_configuration): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
                        // let notification_task = tokio::spawn(async move {
                        //     while let Some(event) = notification_stream.next().await {
                        //         match event.uuid {
                        //             abw_srv::CHR_CONFIGURATION => {
                        //                 match tx_configuration.send(event.value) {
                        //                     Ok(v) => v,
                        //                     Err(e) => {
                        //                         error!("cannot send configuration notification through the selected async channel: {}", e);
                        //                         debug!("{:?}", e);
                        //                     }
                        //                 }
                        //             },
                        //             _ => {},
                        //         }
                        //     }
                        // });
                        // // clean the channel before start
                        // let mut garbage = rx_configuration.try_iter();
                        // while let Some(_) = garbage.next() {}; 


                        info!("Verifying the existence and validity of existing pairing.");
                        // new workaround to test if device is paired (request a parameter value)
                        match device.write(chr_conf, &vec![0x00, 0x00], WriteType::WithResponse).await {
                            Ok(_) => {},
                            Err(e) => {
                                error!("{}", e); debug!("{:?}", e);
                                println!("{}", FIX_FOR_NOT_PAIRED);
                                return Ok(())
                            }
                        }
                        // consuming the response
                        if let Some(_) = notification_stream.next().await {};
                        info!("Paring verified.");




                        // Set BLE connection to 'Very Fast'!
                        // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                        device.write(chr_cust_cmd, &vec![0x02], WriteType::WithoutResponse)
                            .await.with_context(||"couldn't set BLE connection speed to Very Fast")?;
                        info!("BLE connection is set to 'Very Fast'!");
                        

                        match OpenOptions::new()
                            .read(true)
                            .open(config_path)
                        {
                            Ok(config_file) => {

                                let config_file = BufReader::new(config_file);

                                println!("Importing device configuration...");
        
                                let param_name_to_id = get_param_name_to_id();
        
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
        
                                        // If ithis is Some, then there were two '=' characters in the line
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
                                            chr_conf, 
                                            &vec![
                                                0x01, 
                                                *param_id, 
                                                param_value_bytes[0], param_value_bytes[1], param_value_bytes[2], param_value_bytes[3]
                                            ], 
                                            WriteType::WithoutResponse
                                        ).await
                                            .with_context(||format!("couldn't send the 'write config param: {:02x}' BLE command", param_id))?;
        
                                        // Get response as value notification
                                        let Some(ValueNotification{uuid: _, value: param_val_vec}) = notification_stream.next().await else {
                                            warn!("No confirmation was received for config param write request: {}", line);
                                            continue;
                                        };
        
                                        // Evaluate the response
                                        match param_val_vec[0] {
                                            0x00 => {
                                                info!("Parameter sent and accepted: {}", line);
                                                continue;
                                            }
                                            0x01 => {
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
        
                                println!("Device configuration has been imported ");

                            },

                            Err(e) => {
                                match e.kind() {
                                    std::io::ErrorKind::NotFound => {
                                        println!("The file '{}' was not found!", config_path);
                                    },
                                    _ => {
                                        Err(e).with_context(||"The configuration file cannot be opened.")?;
                                    }
                                }
                            }

                        };

                        // Set BLE connection back to 'Slow'!
                        // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                        device.write(chr_cust_cmd, &vec![0x01], WriteType::WithoutResponse).await
                            .with_context(||"couldn't set the BLE connection speed back to Slow")?;

                        info!("BLE connection is set back to 'Slow'!");

                        device.unsubscribe(&chr_conf).await
                            .with_context(||"couldn't unsubscribe from BLE configuration responses")?;

                        // notification_task.abort();

                    } else {
                        return Err(anyhow!("The CONFIGURATION characteristic ({}) cannot be found on the device...", abw_srv::CHR_CONFIGURATION.as_hyphenated()));
                    }
                } else {
                    return Err(anyhow!("The CUSTOM_COMMAND characteristic ({}) cannot be found on the device...", abw_srv::CHR_CUSTOM_CMD.as_hyphenated()));
                } 


                // println!("Disconnecting...");
                // device.disconnect().await
                //     .with_context(||"error while disconnecting the device")?;
                // println!("Disconnected.");
            } else {
                println!("Device {} was not found", selected_device);
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
            }

        }

        return Ok(());

    }




    // *********************************
    // CLI Argument: "export-config"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("export-config") {

        if let Some(selected_device) = sub_cmd_matches.get_one::<String>("device") {

            println!("Scanning...");
            let device = match find_abw_device(&ble_adapter, selected_device).await {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                    println!("Cannot find the selected Abeeway Device.");
                    println!("{}", FIX_FOR_NOT_ADVERTIZING);
                    return Ok(())
                }
            };

            if let Some(device) = device {

                let is_connected = device.is_connected().await
                    .with_context(||"error while checking if the selected device is connected")?;

                println!("The Device was found:\n    {} - {}", 
                    selected_device, 
                    if is_connected { "Connected" } else { "Not Connected" }
                );

                if !is_connected {
                    println!("Connecting...");
                    match device.connect().await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{}", e); debug!("{:?}", e);
                            println!("{}", FIX_FOR_CORRUPTED_PAIRING);
                            return Ok(())
                        }
                    };
                } 
                println!("Connected.");
                info!("Discovering BLE service characteristics...");

                let device_chars =  discover_chars(&device).await
                    .with_context(||"error while discovering device characteristics")?;

                info!("BLE service characteristics discovered.");

                if let Some(ref chr_cust_cmd) = device_chars.custom_cmd {
                    if let Some(ref chr_conf) = device_chars.configuration {

                        let mut notification_stream = device.notifications().await
                            .with_context(||"couldn't get BLE notification stream")?;

                        device.subscribe(&chr_conf).await
                            .with_context(||"couldn't subscribe to BLE configuration notifications")?;


                        // let (tx_configuration, rx_configuration): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
                        // let notification_task = tokio::spawn(async move {
                        //     while let Some(event) = notification_stream.next().await {
                        //         match event.uuid {
                        //             abw_srv::CHR_CONFIGURATION => {
                        //                 match tx_configuration.send(event.value) {
                        //                     Ok(v) => v,
                        //                     Err(e) => {
                        //                         error!("cannot send configuration notification through the selected async channel: {}", e);
                        //                         debug!("{:?}", e);
                        //                     }
                        //                 }
                        //             },
                        //             _ => {},
                        //         }
                        //     }
                        // });
                        // // clean the channel before start
                        // let mut garbage = rx_configuration.try_iter();
                        // while let Some(_) = garbage.next() {}; 


                        info!("Verifying the existence and validity of existing pairing.");
                        // new workaround to test if device is paired (request a parameter value)
                        match device.write(chr_conf, &vec![0x00, 0x00], WriteType::WithResponse).await {
                            Ok(_) => {},
                            Err(e) => {
                                error!("{}", e); debug!("{:?}", e);
                                println!("{}", FIX_FOR_NOT_PAIRED);
                                return Ok(())
                            }
                        }
                        // consuming the response
                        if let Some(_) = notification_stream.next().await {};
                        info!("Paring verified.");




                        // Set BLE connection to 'Very Fast'!
                        // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                        device.write(chr_cust_cmd, &vec![0x02], WriteType::WithoutResponse)
                            .await.with_context(||"couldn't set BLE connection speed to Very Fast")?;
                        info!("BLE connection is set to 'Very Fast'!");
                        













                        if let Some(config_path) = sub_cmd_matches.get_one::<String>("file") {

                            match OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .open(config_path)
                            {
                                Ok(config_file) => {

                                    let mut config_file = BufWriter::new(config_file);

                                    println!("Exporting device configuration...");

                                    for param in PARAMS {

                                        // Write 'Parameter Read Request'
                                        device.write(
                                            chr_conf, 
                                            &vec![
                                                0x00, 
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

                                    println!("Device configuration has been exported ");

                                }
                                Err(e) => {
                                    match e.kind() {
                                        std::io::ErrorKind::AlreadyExists => {
                                            println!("The file '{}' already exists!", config_path);
                                        },
                                        _ => {
                                            Err(e).with_context(||"The new configuration file cannot be created.")?
                                        }
                                    }
                                }
                            }

                        } else {

                            for param in PARAMS {

                                device.write(
                                    chr_conf, 
                                    &vec![
                                        0x00, 
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

                        };













                        // Set BLE connection back to 'Slow'!
                        // 0x00 - Fast, 0x01 - Slow, 0x02 - Very Fast
                        device.write(chr_cust_cmd, &vec![0x01], WriteType::WithoutResponse).await
                            .with_context(||"couldn't set the BLE connection speed back to Slow")?;

                        info!("BLE connection is set back to 'Slow'!");

                        device.unsubscribe(&chr_conf).await
                            .with_context(||"couldn't unsubscribe from BLE configuration responses")?;

                        // notification_task.abort();

                    } else {
                        return Err(anyhow!("The CONFIGURATION characteristic ({}) cannot be found on the device...", abw_srv::CHR_CONFIGURATION.as_hyphenated()));
                    }
                } else {
                    return Err(anyhow!("The CUSTOM_COMMAND characteristic ({}) cannot be found on the device...", abw_srv::CHR_CUSTOM_CMD.as_hyphenated()));
                } 


                // println!("Disconnecting...");
                // device.disconnect().await
                //     .with_context(||"error while disconnecting the device")?;
                // println!("Disconnected.");
            } else {
                println!("Device {} was not found", selected_device);
                println!("{}", FIX_FOR_NOT_ADVERTIZING);
            }

        }

        return Ok(());
    }

    Ok(())

}
