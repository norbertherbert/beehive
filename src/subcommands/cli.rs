
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{self, Write};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use anyhow::{anyhow, Result, Context};
use futures::StreamExt;
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Peripheral as _, WriteType};
use log::*;

use crate::abw_ble::{
    abw_srv,
    find_dev::find_abw_device,
};
use crate::abw_params;

pub async fn cli (
    device_name: &String, 
    ble_adapter: &Adapter, 
) -> Result<()>
{

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

    println!("Starting the CLI connection...");
    match cli_visitor(&device).await {
        Ok(_) => {
            println!("");
            println!("CLI conection terminated.");
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

pub async fn cli_visitor(device: &Peripheral) -> Result<()> {

    let characteristics = device.characteristics();

    // Getting the CUSTOM_SEND_CLI_CMD service characteristic
    let Some(chr_custom_send_cli_cmd) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_CUSTOM_SEND_CLI_CMD} ) 
    else {
        return Err(
            anyhow!(
                "The CUSTOM_SEND_CLI_CMD characteristic ({}) cannot be found on the device...", 
                abw_srv::CHR_CUSTOM_SEND_CLI_CMD.as_hyphenated()
            )
        );
    };

    // Getting the CUSTOM_RCV_SERIAL_DATA service characteristic
    let Some(chr_custom_rcv_serial_data) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_CUSTOM_RCV_SERIAL_DATA} ) 
    else {
        return Err(
            anyhow!(
                "The CUSTOM_RCV_SERIAL_DATA characteristic ({}) cannot be found on the device...", 
                abw_srv::CHR_CUSTOM_RCV_SERIAL_DATA.as_hyphenated()
            )
        );
    };

    // // Getting the CUSTOM_CMD service characteristic
    // let Some(chr_custom_cmd) = characteristics
    //     .iter()
    //     .find(|chr| { chr.uuid == abw_srv::CHR_CUSTOM_CMD} ) 
    // else {
    //     return Err(
    //         anyhow!(
    //             "The CUSTOM_CMD characteristic ({}) cannot be found on the device...", 
    //             abw_srv::CHR_CUSTOM_CMD.as_hyphenated()
    //         )
    //     );
    // };

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
        .with_context(||"couldn't subscribe to BLE configuration notifications (CONFIGURATION characteristics)")?;

    device.subscribe(&chr_custom_rcv_serial_data).await
        .with_context(||"couldn't subscribe to CLI command responses (CUSTOM_RCV_SERIAL_DATA characteristics)")?;


    // Spawn a task that manages notifications through channels 
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
    match device.write(chr_configuration, &vec![abw_srv::WR_READ_CONF, abw_params::UL_PERIOD], WriteType::WithResponse).await {
        Ok(_) => {},
        Err(e) => {
            error!("{}", e); debug!("{:?}", e);
            println!("{}", abw_srv::FIX_FOR_NOT_PAIRED);
            return Ok(())
        }
    }
    // consuming the response
    rx_configuration.recv()
        .with_context(||"Cannot read the configuration notification channel.")?;
    info!("Paring verified.");

    // // Set BLE connection to 'Very Fast'!
    // device.write(chr_custom_cmd, &vec![abw_srv::WR_VERY_FAST_CONN], WriteType::WithoutResponse)
    //     .await.with_context(||"couldn't set BLE connection speed to Very Fast")?;
    // info!("BLE connection is set to 'Very Fast'!");


    // Send the read config_flags command
    device.write(chr_configuration, &vec![abw_srv::WR_READ_CONF, abw_params::CONFIG_FLAGS], WriteType::WithoutResponse).await
        .with_context(||"couldn't send the 'read config_flags' BLE commaand")?;
    // Receive the actual config_flags value
    let res_value = rx_configuration.recv()
        .with_context(||"Cannot read the response to read config_flags command from the notification channel.")?;
    // Check if BLE CLI is enabled in config_flags. Check if bit 4 (20) is turned on.
    if res_value[3] & 1<<4 == 0 {
        // Enable BLE CLI in config_flags. Write new config_flags (set bit 20 to 1).
        device.write(chr_configuration, &vec![
            abw_srv::WR_WRITE_CONF, abw_params::CONFIG_FLAGS, res_value[2], res_value[3] | 1<<4, res_value[4], res_value[5]
        ], WriteType::WithoutResponse).await
            .with_context(||"couldn't send the 'write config_flags' BLE commaand")?;
        info!("BLE CLI (bit 20) has been enabled in config_flags.");
    } else {
        info!("BLE CLI (bit 20) is already enabled in config_flags.");
    }

    // Turn on BLE CLI
    device.write(chr_configuration, &vec![abw_srv::WR_WRITE_CONF, abw_params::BLE_CLI_ACTIVE, 0, 0, 0, 1], WriteType::WithoutResponse).await
        .with_context(||"couldn't send the 'Turn on BLE CLI' commaand")?;


    println!("Press Ctrl+C to leave the CLI interface!");

    // These two lines are needed as a workaround to show the loging prompt at start
    tokio::time::sleep(Duration::from_millis(300)).await;
    device.write(&chr_custom_send_cli_cmd, b"\r\n", WriteType::WithoutResponse).await
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
                    &chr_custom_send_cli_cmd, 
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

    device.unsubscribe(&chr_custom_rcv_serial_data).await
        .with_context(||"couldn't unsubscribe from CLI command responses")?;

    // Turn off BLE CLI
    device.write(chr_configuration, &vec![abw_srv::WR_WRITE_CONF, abw_params::BLE_CLI_ACTIVE, 0, 0, 0, 0], WriteType::WithoutResponse).await
        .with_context(||"couldn't send the 'turn off BLE CLI' command")?;

    // // Set BLE connection back to 'Slow'!
    // device.write(chr_custom_cmd, &vec![abw_srv::WR_SLOW_CONN], WriteType::WithoutResponse).await
    //     .with_context(||"couldn't set the BLE connection speed back to Slow")?;
    // info!("BLE connection is set back to 'Slow'!");

    device.unsubscribe(&chr_configuration).await
        .with_context(||"couldn't unsubscribe from BLE configuration responses")?;

    notification_task.abort();

    Ok(())

}