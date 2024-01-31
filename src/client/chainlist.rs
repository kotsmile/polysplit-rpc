use std::collections::HashMap;

use anyhow::{anyhow, Result};
use regex_macro::regex;

const RESOURCE_URL: &'static str =
    "https://raw.githubusercontent.com/DefiLlama/chainlist/main/constants/extraRpcs.js";
const TARGET_START_LINE: &'static str = "export const extraRpcs = {";
const TARGET_STOP_LINE: &'static str = "const allExtraRpcs = mergeDeep(llamaNodesRpcs, extraRpcs);";

#[derive(Clone, Copy)]
pub struct ChainlistClient;

impl ChainlistClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn fetch_rpcs(&self) -> Result<HashMap<String, Vec<String>>> {
        let content = reqwest::get(RESOURCE_URL)
            .await
            .map_err(|err| anyhow!("failed to make resposne to github: {err}"))?
            .text()
            .await
            .map_err(|err| anyhow!("failed to parse response: {err}"))?;

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
            chain_to_rpc
                .entry(chain_id.to_owned())
                .or_insert_with(Vec::new)
                .push(url.to_owned());
        }

        Ok(chain_to_rpc)
    }
}
