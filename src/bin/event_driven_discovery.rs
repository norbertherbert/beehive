// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use btleplug::api::{
    // bleuuid::BleUuid, 
    Central, Manager as _, Peripheral as _, CentralEvent, ScanFilter, WriteType
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;

use anyhow::{anyhow, Result, 
    Context
};
// use std::error::Error;

// use std::io::Write;

use tokio::time::timeout;
use std::time::Duration;


use beehive::abw_ble_utils::{
    abw,
    // find_dev::{find_abw_device_names, find_abw_device},
    // discover_srv::discover_chars,
};

// #[macro_use] extern crate log;

// const DEVICE_NAME: &'static str = "ABW1E10002BF";
// const DEVICE_NAME: &'static str = "ABW181000049";
const DEVICE_NAME: &'static str = "ABW421000FDF";
// const DEVICE_NAME: &'static str = "ABW421000FDE";


async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

pub async fn discover_device(adapter: &Adapter, device_name: &str) -> Result<Peripheral> {

    let mut events = adapter.events().await?;
    
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(peripheral_id) => {
                let peripheral = adapter.peripheral(&peripheral_id).await?;
                let properties = peripheral.properties().await?;
                if let Some(ref properties) = properties {
                    let name = properties.local_name.as_ref();
                    if let Some(name) = name {
                        if name == device_name {
                            return Ok(peripheral);
                        }
                    }
                }
            }
            _ => {}
        }
    };
    Err(anyhow!(""))

}

async fn discover_device_with_timeout(adapter: &Adapter, device_name: &str, duration: Duration) -> Result<Option<Peripheral>> {
    
    match timeout(duration, discover_device(&adapter, device_name)).await {
        Ok(res) => {
          match res {
                Ok(peripheral) => {
                    Ok(Some(peripheral))
                },
                Err(e) => {
                    Err(e)
                }
            }
        },
        Err(_e) => {
            Ok(None)
        }
    }


    // match timeout(duration, discover_device(&adapter, device_name)).await {
    //     Ok(res) => {
    //         match res {
    //             Ok(peripheral) => {
    //                 Ok(peripheral)
    //             },
    //             Err(e) => {
    //                 Err(anyhow!("error while discovering device {}: {}", device_name, e))
    //             }
    //         }
    //     },
    //     Err(_e) => { 
    //         Err(anyhow!("Device {} not found", device_name))
    //     }
    // }
}



#[tokio::main]
async fn main() -> Result<()> {

    // let log_level = log::LevelFilter::Trace;
    // env_logger::builder()
    //     .format(|f, record| {
    //         writeln!(f, "{}: {}", record.level(), record.args() )
    //     })
    //     .filter(Some("event_driven_discovery"), log_level)
    //     .init();

    let manager = Manager::new().await?;

    let central = get_central(&manager).await;
    central.start_scan(ScanFilter::default()).await?;

    println!("Looking for device: {}...", DEVICE_NAME);

    if let Some(device) = discover_device_with_timeout(&central, DEVICE_NAME, Duration::from_millis(5000)).await? {
        
        println!("The device was found.");
        
        let is_connected = device.is_connected().await?;
        if is_connected {
            println!("The device is already connnected");
        } else {
            println!("The device is not connnected yet");
            println!("Connecting...");
            device.connect().await.with_context(|| "cannot connect to device")?;
            println!("Connected.");
        }

        println!("Lookinng for characteristics...");
        device.discover_services().await?;
        let characteristics = device.characteristics();
        if characteristics.is_empty() {
            println!("No characteristics were found.");
            return Ok(());
        } else {
            // println!("Characteristics were found:\n{:?}", characteristics);
            println!("Characteristics were found.");


            println!("Lookinng for Custom Command characteristic...");
            if let Some(chr_custom_cmd) = characteristics.iter().find(|chr| {chr.uuid == abw::CHR_CUSTOM_CMD} ) {
                println!("Custom Command characteristic was found");

                println!("Removing ble bond...");
                device.write(chr_custom_cmd, &vec![0x99], WriteType::WithoutResponse).await
                    .with_context(||"couldn't remove BLE bond")?;
                println!("BLE bond has been removed!");
                println!("Please make it sure that it has been removed from our computer's OS too.");

            } else {
                println!("Custom Command characteristic was not found");
                return Ok(());
            }


        }

    } else {
        println!("The device was not found.");
        return Ok(());
    };

    Ok(())

}