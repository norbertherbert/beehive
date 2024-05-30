use anyhow::{anyhow, Result, Context};
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Peripheral as _, WriteType};
use log::*;

use crate::abw_ble::{
    abw_srv,
    find_dev::find_abw_device,
};


pub async fn show (
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

    info!("Retreiving device details...");


    // ********************************************************************************
    // *** START of Visitor Pattern
    // ********************************************************************************

    println!("Device details:");
    match show_visitor(&device).await {
        Ok(_) => {
            println!("");
            info!("All Device detailse are successfully retreived.");
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

pub async fn show_visitor(device: &Peripheral) -> Result<()> {

    let characteristics = device.characteristics();

    // Getting the MODEL_NUMBER service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_MODEL_NUMBER} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the MODEL_NUMBER of the device")?;

        let s = String::from_utf8_lossy(&v);
        println!("    Model Number: {}", s);
    };

    // Getting the SERIAL_NUMBER service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_SERIAL_NUMBER} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the SERIAL_NUMBER of the device")?;

        let s = hex::encode(&v);
        println!("    Serial Number: {}", s);
    };

    // Getting the FIRMWARE_REVISION service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_FIRMWARE_REVISION} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the FIRMWARE_REVISION of the device")?;

        let s = String::from_utf8_lossy(&v);
        println!("    Firmware Revision: {}", s);
    }

    // Getting the SOFTWARE_REVISION service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_SOFTWARE_REVISION} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the SOFTWARE_REVISION of the device")?;
        
        let s = String::from_utf8_lossy(&v);
        println!("    Software Revision: {}", s);
    }

    // Getting the MANUFACTURER_NAME service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_MANUFACTURER_NAME} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the MANUFACTURER_NAME of the device")?;

        let s = String::from_utf8_lossy(&v);
        println!("    Manufacturer Name: {}", s);
    }

    // Getting the TX_POWER_LEVEL service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_TX_POWER_LEVEL} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the TX_POWER_LEVEL of the device")?;
        
        // let s = String::from_utf8_lossy(&v);
        println!("    TX Power Level: {}", v[0]);
    }

    // Getting the BATTERY_LEVEL service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_BATTERY_LEVEL} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the BATTERY_LEVEL of the device")?;
        
        println!("    Battery Level: {}%", v[0]);
    }

    // Getting the BATTERY_POWER_STATE service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_BATTERY_POWER_STATE} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the BATTERY_POWER_STATE of the device")?;
        
        let s = match v[0] {
            abw_srv::CHARGER_PRESENT_AND_CHARGING => "Charger present and charging.",
            abw_srv::CHARGER_PRESENT_BUT_NOT_CHARGING => "Charger present but not charging.",
            abw_srv::CHARGER_NOT_PRESENT_AND_DISCHARGING => "Charger not present and discharging.",
            _ => "Unknown"
        };
        println!("    Battery Power State: {}", s);
    }

    // Getting the TEMPERATURE_CELSIUS service characteristic
    if let Some(c) = characteristics
        .iter()
        .find(|chr| { chr.uuid == abw_srv::CHR_TEMPERATURE_CELSIUS} ) 
    {
        let v = device.read(c).await
            .with_context(||"error while reading the TEMPERATURE_CELSIUS value of the device")?;

        let t = u16::from_le_bytes([v[0], v[1]]) as f32 / 10_f32;
        println!("    Temperature: {} C", t);
    }

    Ok(())

}