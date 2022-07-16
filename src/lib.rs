mod config;

use std::net::IpAddr;
use bgpkit_parser::BgpkitParser;
use itertools::Itertools;
use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use tabled::{Table, Tabled};

pub use crate::config::MonocleConfig;

pub fn parser_with_filters(
    file_path: &str,
    origin_asn: &Option<u32>,
    prefix: &Option<String>,
    include_super: &bool,
    include_sub: &bool,
    peer_ip: &Vec<IpAddr>,
    peer_asn: &Option<u32>,
    elem_type: &Option<String>,
    start_ts: &Option<String>,
    end_ts: &Option<String>,
    as_path: &Option<String>,
) -> Result<BgpkitParser> {

    let mut parser = BgpkitParser::new(file_path).unwrap().disable_warnings();

    if let Some(v) = as_path {
        parser = parser.add_filter("as_path", v.to_string().as_str()).unwrap();
    }
    if let Some(v) = origin_asn {
        parser = parser.add_filter("origin_asn", v.to_string().as_str()).unwrap();
    }
    if let Some(v) = prefix {
        let filter_type = match (include_super, include_sub) {
            (false, false) => "prefix",
            (true, false) => "prefix_super",
            (false, true) => "prefix_sub",
            (true, true) => "prefix_super_sub",
        };
        parser = parser.add_filter(filter_type, v.as_str()).unwrap();
    }
    if !peer_ip.is_empty(){
        let v = peer_ip.iter().map(|p| p.to_string()).join(",").to_string();
        parser = parser.add_filter("peer_ips", v.as_str()).unwrap();
    }
    if let Some(v) = peer_asn {
        parser = parser.add_filter("peer_asn", v.to_string().as_str()).unwrap();
    }
    if let Some(v) = elem_type {
        parser = parser.add_filter("type", v.to_string().as_str()).unwrap();
    }
    if let Some(v) = start_ts {
        let ts = string_to_time(v.as_str())?;
        parser = parser.add_filter("start_ts", ts.to_string().as_str()).unwrap();
    }
    if let Some(v) = end_ts {
        let ts = string_to_time(v.as_str())?;
        parser = parser.add_filter("end_ts", ts.to_string().as_str()).unwrap();
    }
    return Ok(parser)
}

#[derive(Tabled)]
struct BgpTime{
    unix: i64,
    rfc3339: String,
}

pub fn string_to_time(time_string: &str) -> Result<i64> {
    let ts = match chrono::DateTime::parse_from_rfc3339(time_string) {
        Ok(ts) => {
            ts.timestamp()
        }
        Err(_) => {
            match time_string.parse::<i64>(){
                Ok(ts) => ts,
                Err(_) => return Err(anyhow!("Input time must be either Unix timestamp or time string compliant with RFC3339"))
            }
        }
    };

    Ok(ts)
}

pub fn time_to_table(time_string: &Option<String>) -> Result<String> {
    let unix = match time_string {
        None => {
            Utc::now().timestamp()
        },
        Some(ts) => {
            string_to_time(ts.as_str())?
        }
    };

    let rfc3339 = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(unix, 0), Utc).to_rfc3339();

    Ok( Table::new(vec![BgpTime{ unix, rfc3339 }]).to_string() )
}