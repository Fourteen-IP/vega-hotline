use regex::Regex;
use rust_xlsxwriter::Workbook;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Serialize)]
pub struct DialPlan {
    pub srce: Option<String>,
    pub dest_raw: Option<String>,
    pub dest_tel: Option<String>,
    pub subscriber: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub plans: HashMap<String, DialPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DialPlanConfig {
    pub profiles: HashMap<String, Profile>,
    pub ip_address: Option<String>,
    pub hostname: Option<String>
}

#[derive(Debug)]
pub struct ExportRow {
    pub vega_ip: String,
    pub vega_name: String,
    pub profile: String,
    pub plan: String,
    pub port: String,
    pub destination_ext: Option<String>,
    pub user_lineport: String,
}

pub async fn extract_dial_plans(
    config: HashMap<String, String>,
) -> Result<DialPlanConfig, Box<dyn Error>> {
    let mut profiles: HashMap<String, Profile> = HashMap::new();
    let mut subscribers: HashMap<String, HashMap<String, String>> = HashMap::new();

    let dialplan_dest_regex = Regex::new(r"planner_profile_[0-9]+_plan_[0-9]+_dest")?;
    let dialplan_srce_regex = Regex::new(r"planner_profile_[0-9]+_plan_[0-9]+_srce")?;
    let auth_subscriber_regex = Regex::new(r"sip_auth_user_[0-9]+_subscriber")?;
    let srce_regex = Regex::new(r"IF:[0-9]+")?;
    let tel_extract_regex = Regex::new(r"TEL:(\(\d+\)|\d+)")?;

    for (command, value) in &config {
        if dialplan_dest_regex.is_match(command) || dialplan_srce_regex.is_match(command) {
            let command_split: Vec<&str> = command.split("_").collect();

            let profile_id = format!("profile_{}", command_split[2]);
            let plan_id = format!("plan_{}", command_split[4]);
            let key = &command_split[5];

            let profile = profiles.entry(profile_id).or_insert(Profile {
                plans: HashMap::new(),
            });
            let plan = profile.plans.entry(plan_id).or_insert(DialPlan {
                srce: None,
                dest_raw: None,
                dest_tel: None,
                subscriber: None,
            });

            match *key {
                "srce" => {
                    if let Some(captures) = srce_regex.captures(&value) {
                        plan.srce = Some(captures[0].to_string());
                    }
                }
                "dest" => {
                    plan.dest_raw = Some(value.to_string());

                    if let Some(captures) = tel_extract_regex.captures(&value) {
                        plan.dest_tel = Some(captures[1].to_string());
                    }
                }
                _ => {}
            }
        } else if auth_subscriber_regex.is_match(command) {
            let command_split: Vec<&str> = command.split("_").collect();

            let user_id = &command_split[3];
            let mut sub_info = HashMap::new();

            sub_info.insert("subscriber".to_string(), value.to_owned());

            let username_key = format!("sip_auth_user_{}_username", user_id);

            sub_info.insert(
                "username".to_string(),
                config
                    .get(&username_key)
                    .cloned()
                    .unwrap_or_default(),
            );

            subscribers.insert(user_id.to_owned().to_string(), sub_info);
        }
    }

    for profile in profiles.values_mut() {
        for plan in profile.plans.values_mut() {
            if let Some(srce_value) = &plan.srce {
                for sub_info in subscribers.values() {
                    if let Some(sub_val) = sub_info.get("subscriber") {
                        if sub_val == srce_value {
                            plan.subscriber = sub_info.get("username").cloned();
                        }
                    }
                }
            }
        }
    }

    let ip_address = config.get("quick_lan_ip").cloned();
    let hostname = config.get("quick_hostname").cloned();

    Ok(DialPlanConfig { profiles, ip_address, hostname })
}

pub fn dial_plan_config_to_excel(
    dial_plans: Vec<DialPlanConfig>,
    file_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut rows = Vec::new();

    for dial_plan in dial_plans {
        for (profile_id, profile) in dial_plan.profiles {
            for (plan_id, plan) in profile.plans {
                if plan.srce.is_some() && plan.dest_tel.is_some() && plan.subscriber.is_some() {
                    let subscriber = plan.subscriber.as_deref().unwrap_or("N/A");

                    let row = ExportRow {
                        vega_ip: dial_plan.ip_address.clone().unwrap_or_default(),
                        vega_name: dial_plan.hostname.clone().unwrap_or_default(),
                        profile: profile_id.clone(),
                        plan: plan_id.clone(),
                        port: plan.srce.clone().unwrap_or_default(),
                        destination_ext: plan.dest_tel.clone(),
                        user_lineport: subscriber.to_string(),
                    };

                    rows.push(row);
                }
            }
        }
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Write headers
    let headers = [
        "Vega Name",
        "Vega IP",
        "Profile",
        "Plan",
        "Port",
        "Destination Ext",
        "User/Lineport",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col as u16, *header)?;
    }

    // Write data rows
    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32; // Start from row 1 (after header)

        worksheet.write_string(r, 0, &row.vega_name)?;
        worksheet.write_string(r, 1, &row.vega_ip)?;
        worksheet.write_string(r, 2, &row.profile)?;
        worksheet.write_string(r, 3, &row.plan)?;
        worksheet.write_string(r, 4, &row.port)?;
        worksheet.write_string(r, 5, row.destination_ext.as_deref().unwrap_or(""))?;
        worksheet.write_string(r, 6, &row.user_lineport)?;
    }

    workbook.save(file_path)?;

    Ok(())
}

pub fn dial_plan_config_to_json(
    dial_plans: Vec<DialPlanConfig>,
    file_path: &str,
) -> Result<(), Box<dyn Error>> {
    let json_out = serde_json::to_string_pretty(&dial_plans).expect("Failed to serialise to JSON");

    let mut file = File::create(file_path).unwrap_or_else(|_| {
        panic!(
            "{}",
            format!("Failed to open file at {}", &file_path).to_string()
        )
    });

    file.write_all(json_out.as_bytes()).unwrap();

    Ok(())
}
