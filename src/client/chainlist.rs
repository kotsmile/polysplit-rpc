use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use regex_macro::regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const RESOURCE_URL: &'static str =
    "https://raw.githubusercontent.com/DefiLlama/chainlist/main/constants/extraRpcs.js";
const TARGET_START_LINE: &'static str = "export const extraRpcs = {";
const TARGET_STOP_LINE: &'static str = "const allExtraRpcs = mergeDeep(llamaNodesRpcs, extraRpcs);";
const CHAINS_URL: &'static str = "https://chainid.network/chains.json";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    pub name: String,
    pub chain: String,
    pub icon: Option<String>,
    pub rpc: Vec<String>,
    #[serde(default)]
    pub features: Vec<Feature>,
    pub faucets: Vec<String>,
    pub native_currency: NativeCurrency,
    #[serde(rename = "infoURL")]
    pub info_url: String,
    pub short_name: String,
    pub chain_id: i64,
    pub network_id: i64,
    pub slip44: Option<i64>,
    pub ens: Option<Ens>,
    #[serde(default)]
    pub explorers: Vec<Explorer>,
    pub title: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub red_flags: Vec<String>,
    pub parent: Option<Parent>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NativeCurrency {
    pub name: String,
    pub symbol: String,
    pub decimals: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Ens {
    pub registry: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Explorer {
    pub name: String,
    pub url: String,
    pub standard: String,
    pub icon: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Parent {
    #[serde(rename = "type")]
    pub type_field: String,
    pub chain: String,
    #[serde(default)]
    pub bridges: Vec<Bridge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bridge {
    pub url: String,
}

#[derive(Clone, Copy)]
pub struct ChainlistClient;

impl ChainlistClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn fetch_rpcs(&self) -> Result<HashMap<String, Vec<String>>> {
        let content = reqwest::get(RESOURCE_URL)
            .await
            .context("failed to make resposne to github")?
            .text()
            .await
            .context("failed to parse response")?;

        let lines: Vec<&str> = content.split('\n').collect();

        let start_line = lines
            .iter()
            .position(|&value| value == TARGET_START_LINE)
            .ok_or(anyhow!("failed to locate start line"))?;
        let stop_line = lines
            .iter()
            .position(|&value| value == TARGET_STOP_LINE)
            .ok_or(anyhow!("failed to locate stop line"))?;

        let target_object = &lines[start_line..stop_line];
        let mut chain_to_rpc: HashMap<String, Vec<String>> = HashMap::new();

        let chain_id_re = regex!(r"^(\d+): \{$");
        let url_re = regex!(
            r#""(https:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*))""#
        );

        let mut chain_id = "";
        for &line in target_object {
            if let Some(caps) = chain_id_re.captures(line.trim()) {
                chain_id = &caps.get(1).map(|m| m.as_str()).expect("unreachable");
                continue;
            }

            let Some(caps) = url_re.captures(line.trim()) else {
                continue;
            };

            let url: &str = &caps.get(1).map(|m| m.as_str()).expect("unreachable");

            if String::from(url).contains("polysplit") {
                continue;
            }
            chain_to_rpc
                .entry(chain_id.to_owned())
                .or_insert_with(Vec::new)
                .push(url.to_owned());
        }

        Ok(chain_to_rpc)
    }

    pub async fn fetch_chains(&self) -> Result<Vec<ChainConfig>> {
        reqwest::get(CHAINS_URL)
            .await
            .context("failed to make resposne to chains source")?
            .json::<Vec<ChainConfig>>()
            .await
            .context("failed to parse response")
    }
}
