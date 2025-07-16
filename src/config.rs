use regex::Regex;
use reqwest::{Client, redirect::Policy};
use scraper::{Html, Selector};
use std::error::Error;
use std::{collections::HashMap, net::IpAddr};
use log::{error, info};

const CSRF_TOKEN_SELECTOR: &str = "input[name=\"csrf-token\"]";
const LOGIN_ENDPOINT: &str = "/vs_login";
const CONFIG_ENDPOINT: &str = "/config.txt";

pub async fn fetch_config(
    vega_ip: IpAddr,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .cookie_store(true)
        .redirect(Policy::none())
        .danger_accept_invalid_certs(true)
        .build()?;

    info!("HTTP client built (cookies enabled, redirects disabled)");

    let vega_url = format!("https://{}", vega_ip);

    info!("Fetching index page from https://{}{}", vega_ip, "/index.htm");

    let index_page = client
        .get(format!("{}/index.htm", vega_url))
        .send()
        .await?
        .text()
        .await?;

    let index_document = Html::parse_document(&index_page);

    let csrf_token_selector = Selector::parse(CSRF_TOKEN_SELECTOR).unwrap();

    let csrf_token = index_document
        .select(&csrf_token_selector)
        .next()
        .and_then(|element| element.value().attr("value")) // And then, if you have an element, get its value
        .ok_or_else(|| {
            error!("CSRF Token not found for IP: {}", vega_ip);
            std::io::Error::other(format!("Missing CSRF TOKEN for {}", vega_ip))
        });

    info!("Extracted CSRF token");

    let params = [
        ("username", username),
        ("password", password),
        ("last", "1"),
        ("csrf_token", csrf_token?),
    ];

    info!("Attempting login for user '{}' at {}", username, vega_ip);

    let login_page = client
        .post(format!("{}{}", vega_url, LOGIN_ENDPOINT))
        .form(&params)
        .send()
        .await?;

    if login_page.status().is_redirection() {
        info!("Login HTTP status: {}", login_page.status());
        let sid_regex = Regex::new(r"sid=(-?\d+)")?;

        let sid = login_page
            .headers()
            .get_all("set-cookie")
            .iter()
            .find_map(|header_value| {
                let cookie_str = header_value.to_str().ok()?;
                sid_regex
                    .captures(cookie_str)
                    .and_then(|cap| cap.get(1))
                    .map(|m| m.as_str().to_string())
            });

        info!("Session ID (sid) extracted: {}", sid.as_ref().unwrap());


        info!("Pulling config from {}", vega_ip);
        let config = client
            .get(format!("https://{}{}", vega_ip, CONFIG_ENDPOINT))
            .query(&[("sid", &sid)])
            .send()
            .await?
            .text()
            .await?;

        Ok(config)
    } else {
        Err(format!("Failed with: {}", login_page.status()).into())
    }
}

pub async fn format_config(config: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut config_map = HashMap::new();

    info!("Parsing configuration text into HashMap");

    for line in config.lines() {
        let line = line.trim();

        if !(line.contains("=") & line.contains("set")) {
            continue;
        }

        let line = line.replace("set", "").trim().to_string();

        let mut key_value_pair: Vec<String> = line.split("=").map(|s| s.to_string()).collect();

        key_value_pair[0] = key_value_pair[0]
            .replace(".", "_")
            .trim_start_matches("_")
            .to_string();

        config_map.insert(
            key_value_pair[0].to_string(),
            key_value_pair[1].replace(r#"""#, "").trim().to_string(),
        );
    }

    Ok(config_map)
}
