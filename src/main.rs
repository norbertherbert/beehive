use std::io::Write;
use anyhow::{anyhow, Result, Context};
use btleplug::platform::Manager;
use btleplug::api::{Central, Manager as _};
use clap::{arg, command, Command};

#[macro_use] extern crate log;

use beehive::abw_ble::{
    abw_srv,
    find_dev::find_abw_device_names,
};
use beehive::subcommands;


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
            Command::new("firmware-update")
                .about("Upgrade MCU firmware.")
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
                println!("{}", abw_srv::FIX_FOR_NOT_ADVERTIZING);
                return Ok(())
            }
        };
        match found_abw_device_names.len() {
            0 => {
                println!("No Abeeway devices were found.");
                println!("{}", abw_srv::FIX_FOR_NOT_ADVERTIZING);
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
        if let Some(device_name) = sub_cmd_matches.get_one::<String>("device") {
        
            match subcommands::show(device_name, &ble_adapter).await {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                }
            };

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "cli"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("cli") {
        if let Some(device_name) = sub_cmd_matches.get_one::<String>("device") {
        
            match subcommands::cli(device_name, &ble_adapter).await {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                }
            };

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "remove-bond"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("remove-bond") {
        if let Some(device_name) = sub_cmd_matches.get_one::<String>("device") {
        
            match subcommands::remove_bond(device_name, &ble_adapter).await {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                }
            };

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "import-config"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("import-config") {
        if let (Some(device_name), Some(file_path)) = 
            (sub_cmd_matches.get_one::<String>("device"), sub_cmd_matches.get_one::<String>("file")) 
        {
        
            match subcommands::import_config(device_name, file_path, &ble_adapter).await {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                }
            };

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "export-config"
    // *********************************


    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("export-config") {
        if let Some(device_name) = sub_cmd_matches.get_one::<String>("device") 
        {

            let file_path = sub_cmd_matches.get_one::<String>("file");
        
            match subcommands::export_config(device_name, file_path, &ble_adapter).await {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                }
            };

        }

        return Ok(());

    }


    // *********************************
    // CLI Argument: "firmware-update"
    // *********************************

    if let Some(sub_cmd_matches) = cli_arg_matches.subcommand_matches("firmware-update") {
        if let (Some(device_name), Some(file_path)) = 
            (sub_cmd_matches.get_one::<String>("device"), sub_cmd_matches.get_one::<String>("file")) 
        {

            match subcommands::firmware_update(device_name, file_path, &ble_adapter).await {
                Ok(()) => (),
                Err(e) => {
                    error!("{}", e); debug!("{:?}", e);
                }
            };

        }

        return Ok(());

    }

    Ok(())

}


