use anyhow::Result;
use serde_json::Value;

pub fn parse_code_id(response: &Value) -> Result<u64> {
    let code_id_event = response["tx_response"]["logs"].as_array().and_then(|logs| {
        logs.iter().find_map(|log| {
            log["events"].as_array().and_then(|events| {
                events
                    .iter()
                    .find(|event| event["type"].as_str() == Some("store_code"))
            })
        })
    });
    let code_id = code_id_event
        .and_then(|event| {
            event["attributes"].as_array().and_then(|attributes| {
                attributes.iter().find_map(|attr| {
                    if attr["key"].as_str() == Some("code_id") {
                        attr["value"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
        })
        .map(|s| s.parse())
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("Failed to parse code id"))?;
    Ok(code_id)
}

pub fn parse_instantiated_address(response: &Value) -> Result<String> {
    let instantiate_event = response["tx_response"]["logs"].as_array().and_then(|logs| {
        logs.iter().find_map(|log| {
            log["events"].as_array().and_then(|events| {
                events
                    .iter()
                    .find(|event| event["type"].as_str() == Some("instantiate"))
            })
        })
    });
    let address = instantiate_event
        .and_then(|event| {
            event["attributes"].as_array().and_then(|attributes| {
                attributes.iter().find_map(|attr| {
                    if attr["key"].as_str() == Some("_contract_address") {
                        attr["value"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to parse contract address"))?;
    Ok(address)
}
