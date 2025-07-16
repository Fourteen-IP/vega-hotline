use crate::args::{ConfigArgs, ScanArgs};
use crate::dialplans::DialPlanConfig;
use std::error::Error;
use log::{error};
use std::net::{IpAddr, Ipv4Addr};
use std::fs;

mod args;
mod config;
mod dialplans;

#[tokio::main]
async fn main() {
    let args = args::get_args();

    init_logger(args.verbose);

    match args.command {
        args::Commands::Scan(scan_args) => {
            if let Err(e) = handle_scan(scan_args).await {
                error!("Scan failed: {}", e);
            }
        }
        args::Commands::Config(config_args) => {
            if let Err(e) = handle_config(config_args).await {
                error!("Config parse failed: {}", e);
            }
        }
    }
}

async fn handle_scan(scan_args: ScanArgs) -> Result<(), Box<dyn Error>> {
    match (scan_args.start_ip, scan_args.end_ip) {
        (Some(start), Some(end)) => { //Both Addresses are handed

            match (start, end) {
                (IpAddr::V4(start_ip), IpAddr::V4(end_ip)) => {

                    let mut dial_plans = Vec::<DialPlanConfig>::new();
                    let start_ip_int = u32::from(start_ip);
                    let end_ip_int = u32::from(end_ip);

                    let ip_range: Vec<IpAddr> = (start_ip_int..=end_ip_int)
                        .map(|ip| IpAddr::V4(Ipv4Addr::from(ip)))
                        .collect();

                    for ip in ip_range {
                        let config = config::fetch_config(ip, &scan_args.username, &scan_args.password).await?;
                        let parsed_config = config::format_config(&config).await?;
                        let dial_plan = dialplans::extract_dial_plans(parsed_config).await?;

                        dial_plans.push(dial_plan);

                    }

                    if let Some(excel_file) = scan_args.excel_file {
                        dialplans::dial_plan_config_to_excel(
                            dial_plans.clone(),
                            &excel_file,
                        )?;
                    }

                    if let Some(json_file) = scan_args.json_file {
                        dialplans::dial_plan_config_to_json(dial_plans.clone(), &json_file)?;
                    }


                }
                _ => error!("Invalid IP Format"),
            }

        }
        (Some(start), None) => {
            //Only start address is handed
            let config =
                config::fetch_config(start, &scan_args.username, &scan_args.password).await?;
            let parsed_config = config::format_config(&config).await?;
            let dial_plan = dialplans::extract_dial_plans(parsed_config).await?;

            if let Some(excel_file) = scan_args.excel_file {
                dialplans::dial_plan_config_to_excel(
                    vec![dial_plan.clone()],
                    &excel_file,
                )?;
            }

            if let Some(json_file) = scan_args.json_file {
                dialplans::dial_plan_config_to_json(vec![dial_plan.clone()], &json_file)?;
            }
        }
        _ => {
            error!("Start IP is required")
        }
    }

    Ok(())
}

async fn handle_config(config_args: ConfigArgs) -> Result<(), Box<dyn Error>> {

    let file_path = config_args.config_file;

    let config_file: String = fs::read_to_string(file_path).expect("Failed to read config file.");

    let parsed_config = config::format_config(&config_file)
        .await
        .expect("Failed to parse config file.");

    let dial_plan = dialplans::extract_dial_plans(parsed_config)
        .await
        .expect("Failed to parse dial plans");

    if let Some(excel_file) = config_args.excel_file {
        dialplans::dial_plan_config_to_excel(
            vec![dial_plan.clone()],
                 &excel_file,
            )?;
        }

    if let Some(json_file) = config_args.json_file {
        dialplans::dial_plan_config_to_json(vec![dial_plan.clone()], &json_file)?;
    }

    Ok(())
}

fn init_logger(verbosity: u8) {
    let log_level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    unsafe {
        std::env::set_var("RUST_LOG", log_level);
    }
    env_logger::init();
}