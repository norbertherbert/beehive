use std::time::Duration;
use std::io::{self, Write};
use anyhow::{Context, Result};
use tokio::time;
use btleplug::platform::{Adapter, Peripheral};
use btleplug::api::{Central, Peripheral as _, ScanFilter};

use super::abw_srv;

const TIMEOUT: Duration = Duration::from_millis(100);

pub async fn find_abw_device_names(adapter: &Adapter) -> Result<Vec<(String, bool)>> {

    adapter.start_scan(ScanFilter::default()).await.with_context(||"scan on ble adapter failed")?;

    let mut peripherals = adapter.peripherals().await?;

    let mut found_abw_devices: Vec<(String, bool)> = Vec::new();


    let mut stdout = io::stdout().lock();
    let progress_length: usize = 100;

    for progress in 0..progress_length {

        let bar = create_progress_bar(progress_length, progress);

        let _ = stdout.write(&bar);
        let _ = stdout.write(b"\r");
        let _ = stdout.flush();

        time::sleep(TIMEOUT).await;
        if (progress < 40) || (progress % 10) != 0 { continue }

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

        peripherals = adapter.peripherals().await?;

    };

    let _ = stdout.write(b"                                                                                        \r");

    Ok(found_abw_devices)

}

use crate::progress_bar::create_progress_bar;

pub async fn find_abw_device(adapter: &Adapter, selected_device: &String) -> Result<Option<Peripheral>> {

    adapter.start_scan(ScanFilter::default()).await.with_context(||"scan on ble adapter failed")?;

    let mut found_device: Option<Peripheral> = None;

    let mut stdout = io::stdout().lock();
    let progress_length: usize = 100;

    'outher: for progress in 0..progress_length {

        let bar = create_progress_bar(progress_length, progress);
        
        let _ = stdout.write(&bar);
        let _ = stdout.write(b"\r");
        let _ = stdout.flush();

        time::sleep(TIMEOUT).await;
        if progress % 10 != 0 { continue }

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

    };

    let _ = stdout.write(b"                                                                                        \r");

    Ok(found_device)

}
