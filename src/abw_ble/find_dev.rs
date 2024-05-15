use std::time::Duration;
use anyhow::{Context, Result};
use tokio::time;
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Central, Peripheral as _, ScanFilter};

use super::abw_srv;

pub async fn find_abw_device_names(adapter: &Adapter) -> Result<Vec<(String, bool)>> {

    adapter.start_scan(ScanFilter::default()).await.with_context(||"scan on ble adapter failed")?;

    time::sleep(Duration::from_secs(5)).await;
    let mut peripherals = adapter.peripherals().await?;

    let mut found_abw_devices: Vec<(String, bool)> = Vec::new();

    for _i in 0..10 {

        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await.with_context(||"cannot get the properies of a found peripheral")?;
            let local_name = properties
                .unwrap()
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            if local_name.starts_with(abw_srv::PERIPHERAL_NAME_MATCH_FILTER) {
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


pub async fn find_abw_device(adapter: &Adapter, selected_device: &String) -> Result<Option<Peripheral>> {

    adapter.start_scan(ScanFilter::default()).await.with_context(||"scan on ble adapter failed")?;

    let mut found_device: Option<Peripheral> = None;

    time::sleep(Duration::from_secs(5)).await;

    'outher: for _i in 0..10 {

        let peripherals = adapter.peripherals().await.with_context(||"cannot get ble peripherals")?;

        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await.with_context(||"cannot get the properies of a found peripheral")?;
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
