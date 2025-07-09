import requests
import ipaddress
import argparse
import json
import regex
import pandas as pd
import logging
from collections import defaultdict


requests.packages.urllib3.disable_warnings()

parser = argparse.ArgumentParser(
    usage="hotline [-h] [-s START_IP] [-e END_IP] -u USERNAME -p PASSWORD [-x TO_EXCEL] [-j JSON] [-q QUIET] [-c]",
    description="CLI Tool designed to return hotline information from Sangoma Vegas",
    epilog="If you need the output of one device, just use --start-ip\nExample: hotline -s 192.168.1.10 -u admin -p secret -x output.xlsx",
    formatter_class=argparse.RawTextHelpFormatter,
)

required = parser.add_argument_group("required arguments")
optional = parser.add_argument_group("optional arguments")
creds = parser.add_argument_group("credentials")
optional.add_argument(
    "-s", "--start-ip", help="start IP address", required=False, metavar=""
)
optional.add_argument(
    "-e", "--end-ip", help="end IP address", required=False, metavar=""
)
optional.add_argument(
    "-v",
    "--verbose",
    help="enable verbose logging",
    required=False,
    action="store_true",
)
creds.add_argument("-u", "--username", help="vega Username", required=False, metavar="")
creds.add_argument("-p", "--password", help="vega Password", required=False, metavar="")
parser.add_argument(
    "-x",
    "--to-excel",
    help="file path to output to an excel spreadsheet",
    required=False,
    default="output.xlsx",
    metavar="",
)
parser.add_argument(
    "-j",
    "--json",
    help="file path to output to a json file",
    metavar="",
)
parser.add_argument(
    "-q", "--quiet", help="ignore warnings", required=False, action="store_true"
)
parser.add_argument(
    "-c",
    "--from-config",
    help="file path to use config file instead of ip address",
    required=False,
    metavar="",
)

args = parser.parse_args()

log_level = (
    logging.ERROR if args.quiet else (logging.DEBUG if args.verbose else logging.INFO)
)

logging.basicConfig(
    level=log_level, format="%(asctime)s - %(levelname)s - %(funcName)s - %(message)s"
)


def pull_backup(ip: ipaddress.IPv4Address) -> str:
    logging.info(f"Attempting to pull backup from {ip}")
    with requests.Session() as session:
        headers = {
            "User-Agent": "Mozilla/5.0",
            "Referer": f"https://{ip}/index.htm",
            "Origin": f"https://{ip}",
            "Content-Type": "application/x-www-form-urlencoded",
        }

        session.headers.update(headers)

        try:
            index_page = session.get(
                f"https://{ip}/index.htm", verify=False, timeout=10
            )
            index_page.raise_for_status()
            logging.debug(f"Successfully indexed page for {ip}")

            csrf = regex.search(r'name="csrf-token"\s+value="([^"]+)"', index_page.text)

            if not csrf:
                logging.error(f"No CSRF token found for {ip}")
                raise RuntimeError("No CSRF token found")

            csrf_token = csrf.group(1)
            logging.debug(f"CSRF token for {ip}: {csrf_token}")

            params = {
                "username": args.username,
                "password": args.password,
                "last": 1,
                "csrf-token": csrf_token,
            }

            login = session.post(
                f"https://{ip}/vs_login",
                data=params,
                verify=False,
                allow_redirects=True,
            )
            login.raise_for_status()
            logging.info(f"Successfully logged into Vega at {ip}")

            sid = session.cookies.get("sid") or regex.search(
                r"sid=(\d+)", login.headers.get("Location", "")
            )
            if not sid:
                logging.error(f"No SID cookie found for {ip}")
                raise RuntimeError("Authentication failed: No SID found")

            if isinstance(sid, regex.Match):
                sid = sid.group(1)

            config_response = session.get(
                f"https://{ip}/config.txt?sid={sid}", verify=False
            )
            config_response.raise_for_status()
            logging.info(f"Successfully retrieved config from {ip}")

            config = config_response.content.decode("utf-8")
            if not config:
                logging.warning(f"Config file from {ip} is empty.")
                raise RuntimeError("Empty config file")

            return config

        except requests.exceptions.RequestException as e:
            logging.error(f"Failed to connect to {ip}: {e}")
            raise
        finally:
            session.close()
            logging.debug(f"Session closed for {ip}")


def parse_config(config: str) -> dict:
    logging.debug("Parsing config file")
    config_dict = {}

    for command in config.splitlines():
        if "set" not in command:
            continue

        command = command.replace("set", "").strip()

        if "=" not in command:
            continue

        command = command.split("=", 1)

        config_dict[command[0].strip()] = command[1].replace('"', "").strip()
    logging.info("Successfully parsed config file")
    return config_dict


def format_match_dial_plans(config: dict) -> dict:
    logging.debug("Formatting and matching dial plans")
    commands_by_profile_plan = defaultdict(lambda: defaultdict(dict))
    subscribers = {}

    regex_list_dialplan = [
        r"\.planner\.profile\.[0-9]+\.plan\.[0-9]+\.dest",
        r"\.planner\.profile\.[0-9]+\.plan\.[0-9]+\.srce",
    ]

    regex_auth_subscriber = r"\.sip\.auth\.user\.[0-9]+\.subscriber"

    dial_plan = regex.compile("|".join(regex_list_dialplan))

    for command, value in config.items():
        if regex.match(dial_plan, command):
            command_split = command.split(".")

            profile_id = f"profile_{command_split[3]}"
            plan_id = f"plan_{command_split[5]}"
            key = command_split[6]

            commands_by_profile_plan[profile_id][plan_id][key] = value
        elif regex.match(regex_auth_subscriber, command):
            user_id = command.split(".")[4]
            subscribers[user_id] = {
                "subscriber": value,
                "username": config[f".sip.auth.user.{user_id}.username"],
            }

    for profile_id, plans in commands_by_profile_plan.items():
        for plan_id, plan_data in plans.items():
            for user_id, sub_info in subscribers.items():
                if plan_data.get("srce") == sub_info["subscriber"]:
                    plan_data["subscriber"] = sub_info["username"]
    logging.info("Successfully formatted and matched dial plans")
    return json.loads(json.dumps(commands_by_profile_plan))


def to_excel(config: dict, data: dict) -> bool:
    logging.debug(f"Preparing data for Excel export to {args.to_excel}")
    rows = []
    for profile_id, plans in data.items():
        for plan_id, plan_data in plans.items():
            subscriber = plan_data.get("subscriber")
            if not subscriber:
                continue

            dest = plan_data.get("dest", "")
            m = regex.search(r"TEL:(\d+)", dest)
            ext_number = m.group(1) if m else None

            row = {
                "Vega IP": config.get(".lan_profile.1.ip", "N/A"),
                "Vega Name": config.get(".quick.hostname", "N/A"),
                "Profile": profile_id,
                "Plan": plan_id,
                "Port": plan_data.get("srce", ""),
                "Destination Ext": ext_number,
                "User/Lineport": subscriber,
            }
            rows.append(row)

    df = pd.DataFrame(rows)
    df.to_excel(args.to_excel, index=False)
    logging.info(f"Successfully exported data to {args.to_excel}")
    return True


def main():
    if args.from_config:
        logging.info(f"Processing config file: {args.from_config}")
        process(args.from_config)
    else:
        if not args.username or not args.password:
            parser.error(
                "Username and password are required unless --from-config (-c) is used"
            )
        if args.start_ip and args.end_ip:
            logging.info(f"Processing IP range: {args.start_ip} - {args.end_ip}")
            start_ip = ipaddress.ip_address(args.start_ip)
            end_ip = ipaddress.ip_address(args.end_ip)

            for ip_int in range(int(start_ip), int(end_ip) + 1):
                ip = str(ipaddress.ip_address(ip_int))
                logging.info(f"Processing IP address: {ip}")
                try:
                    process(ip)
                except Exception as e:
                    logging.error(f"Failed to process {ip}: {e}")
        elif args.start_ip:
            logging.info(f"Processing single IP address: {args.start_ip}")
            try:
                process(args.start_ip)
            except Exception as e:
                logging.error(f"Failed to process {args.start_ip}: {e}")
        else:
            parser.error("No IP address or config file provided")


def process(target):
    try:
        config = load_config(target)
        parsed_config = parse_config(config)
        data = format_match_dial_plans(parsed_config)
        output_results(target, parsed_config, data)
    except Exception as e:
        logging.error(f"An error occurred during processing of {target}: {e}")


def load_config(target):
    if args.from_config:
        logging.info(f"Reading config from file: {target}")
        with open(target, "r") as f:
            return f.read()
    else:
        logging.info(f"Pulling config from Vega: {target}")
        return pull_backup(ipaddress.ip_address(target))


def output_results(target, parsed_config, data):
    if args.to_excel:
        logging.info(f"Outputting results for {target} to Excel: {args.to_excel}")
        to_excel(parsed_config, data)
    if args.json:
        logging.info(f"Outputting results for {target} to JSON: {args.json}")
        with open(args.json, "w") as f:
            json.dump(data, f, indent=4)


if __name__ == "__main__":
    main()
