use anyhow::{anyhow, Result, Context};
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Peripheral as _, WriteType};
use log::*;

use crate::abw_ble::{
    abw_srv,
    find_dev::find_abw_device,
};


pub async fn remove_bond (
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

    info!("BLE Services are discovered.");


    // ********************************************************************************
    // *** START of Visitor Pattern
    // ********************************************************************************

    println!("Removing BLE bond...");
    match remove_bond_visitor(&device).await {
        Ok(_) => {
            println!("BLE bond has been removed!");
            println!("Please make sure that it has been removed from our computer's OS too.");
        },
        Err(e) => {
            println!("");
            error!("{}", e); debug!("{:?}", e);
        }
    }

    // ********************************************************************************
    // *** END of Visitor Pattern
    // ********************************************************************************


    // println!("Disconnecting...");
    // device.disconnect().await
    //     .with_context(||"error while disconnecting the device")?;
    // println!("Disconnected.");

    Ok(())

}


// ********************************************************************************
// *** The Visitor Function
// ********************************************************************************

pub async fn remove_bond_visitor(device: &Peripheral) -> Result<()> {

    let characteristics = device.characteristics();

    // Getting the CUSTOM_COMMAND service characteristic
    let Some(chr_custom_cmd) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_CUSTOM_CMD} ) 
    else {
        return Err(
            anyhow!("cannot find the CUSTOM_COMMAND service characteristics")
        );
    };

    device.write(chr_custom_cmd, &vec![abw_srv::WR_CLEAR_BOND], WriteType::WithoutResponse).await
        .with_context(||"couldn't remove BLE bond")?;

    // TODO: Check if write was successful!

    Ok(())

}