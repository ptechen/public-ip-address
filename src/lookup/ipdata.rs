//! <https://ipdata.co> lookup provider

use super::Result;
use crate::{
    lookup::{LookupProvider, Provider},
    LookupResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

// https://docs.ipdata.co/docs
#[derive(Serialize, Deserialize, Debug)]
pub struct IpDataResponse {
    ip: String,
    is_eu: Option<bool>,
    city: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    region_type: Option<String>,
    country_name: Option<String>,
    country_code: Option<String>,
    continent_name: Option<String>,
    continent_code: Option<String>,
    longitude: Option<f64>,
    latitude: Option<f64>,
    postal: Option<String>,
    calling_code: Option<String>,
    asn: Option<Asn>,
    carrier: Option<Carrier>,
    time_zone: Option<Timezone>,
    threat: Option<Threat>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Threat {
    is_vpn: Option<bool>,
    is_tor: Option<bool>,
    is_proxy: Option<bool>,
    is_datacenter: Option<bool>,
    is_anonymous: Option<bool>,
    is_known_attacker: Option<bool>,
    is_known_abuser: Option<bool>,
    is_threat: Option<bool>,
    is_bogon: Option<bool>,
    blocklists: Option<Vec<Blocklist>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Blocklist {
    name: Option<String>,
    site: Option<String>,
    #[serde(rename = "type")]
    block_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Timezone {
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Asn {
    asn: Option<String>,
    name: Option<String>,
    domain: Option<String>,
    route: Option<String>,
    #[serde(rename = "type")]
    service_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Carrier {
    name: Option<String>,
    mcc: Option<String>,
    mnc: Option<String>,
}

impl IpDataResponse {
    pub fn parse(input: String) -> Result<IpDataResponse> {
        let deserialized: IpDataResponse = serde_json::from_str(&input)?;
        Ok(deserialized)
    }

    pub fn into_response(self) -> LookupResponse {
        let mut response = LookupResponse::new(
            self.ip
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            LookupProvider::IpData(None),
        );
        response.continent = self.continent_name;
        response.country = self.country_name;
        response.country_code = self.country_code;
        response.region = self.region;
        response.postal_code = self.postal;
        response.city = self.city;
        response.latitude = self.latitude;
        response.longitude = self.longitude;
        if let Some(time_zone) = self.time_zone {
            response.time_zone = time_zone.name;
        }
        if let Some(asn) = self.asn {
            response.asn_org = asn.name;
            response.asn = asn.asn;
        }
        if let Some(threat) = self.threat {
            response.is_proxy = threat.is_proxy;
        }

        response
    }
}

pub struct IpData {
    key: Option<String>,
}

impl IpData {
    /// Create a new IpData instance with an API key
    pub fn new(key: Option<String>) -> IpData {
        IpData { key }
    }
}

impl Provider for IpData {
    fn make_api_request(&self) -> Result<String> {
        let endpoint = format!(
            "https://api.ipdata.co/?api-key={}",
            self.key.as_ref().unwrap_or(&"".to_string())
        );
        let response = reqwest::blocking::get(endpoint);
        super::handle_response(response)
    }

    fn parse_reply(&self, json: String) -> Result<LookupResponse> {
        let response = IpDataResponse::parse(json)?;
        Ok(response.into_response())
    }

    fn get_type(&self) -> LookupProvider {
        LookupProvider::IpData(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = r#"
{
  "ip": "1.1.1.1",
  "is_eu": false,
  "city": "Syracuse",
  "region": "New York",
  "region_code": "NY",
  "region_type": "state",
  "country_name": "United States",
  "country_code": "US",
  "continent_name": "North America",
  "continent_code": "NA",
  "latitude": 43.0483,
  "longitude": -76.1468,
  "postal": "13261",
  "calling_code": "1",
  "flag": "https://ipdata.co/flags/us.png",
  "emoji_flag": "🇺🇸",
  "emoji_unicode": "U+1F1FA U+1F1F8",
  "asn": {
    "asn": "AS15169",
    "name": "Google LLC",
    "domain": "google.com",
    "route": "35.192.0.0/14",
    "type": "hosting"
  },
  "carrier": {
    "name": "T-Mobile",
    "mcc": "310",
    "mnc": "160"
  },
  "languages": [
    {
      "name": "English",
      "native": "English",
      "code": "en"
    }
  ],
  "currency": {
    "name": "Australian Dollar",
    "code": "AUD",
    "symbol": "AU$",
    "native": "$",
    "plural": "Australian dollars"
  },
  "time_zone": {
    "name": "America/Los_Angeles",
    "abbr": "PDT",
    "offset": "-0700",
    "is_dst": true,
    "current_time": "2019-03-27T01:13:48.930025-07:00"
  },
  "threat": {
    "is_tor": false,
    "is_icloud_relay": false,
    "is_proxy": false,
    "is_datacenter": false,
    "is_anonymous": false,
    "is_known_attacker": false,
    "is_known_abuser": false,
    "is_threat": false,
    "is_bogon": false,
    "blocklists": []
  },
  "count": "1"
}
"#;

    #[test]
    #[ignore]
    fn test_request() {
        use std::env;
        let key = env::var("IPDATA_APIKEY").ok();
        assert!(key.is_some(), "Missing APIKEY");

        let service = Box::new(IpData::new(key));
        let result = service.make_api_request();
        assert!(result.is_ok(), "Failed getting result {:#?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Result is empty");
        println!("IpData: {:#?}", result);

        let response = IpDataResponse::parse(result);
        assert!(response.is_ok(), "Failed parsing response {:#?}", response);
    }

    #[test]
    fn test_parse() {
        let response = IpDataResponse::parse(TEST_INPUT.to_string()).unwrap();
        assert_eq!(response.ip, "1.1.1.1", "IP address not matching");
        let lookup = response.into_response();
        assert_eq!(
            lookup.ip,
            "1.1.1.1".parse::<IpAddr>().unwrap(),
            "IP address not matching"
        );
    }
}
