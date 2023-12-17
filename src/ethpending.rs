use serde::{Deserialize, Serialize};

use reth_rpc_types::{
    state::StateOverride,
    trace::geth::{GethDebugTracingOptions, GethTrace},
    BlockId, BlockOverrides, CallRequest, Log, TransactionReceipt,
};

/// Options for Emulation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmulateOptions {
    /// All the options
    pub tracing_options: Option<GethDebugTracingOptions>,
    /// The state overrides to apply
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_overrides: Option<StateOverride>,
    /// The block overrides to apply
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_overrides: Option<BlockOverrides>,
}

fn default_0x() -> String {
    "0x".to_string()
}

///
/// Custom EthPendingApi resp
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSimulationInfo {
    /// Trace Debug Info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_debug_info: Option<Vec<GethTrace>>,
    /// Total Gas Used
    pub total_gas_used: u64,
    /// Always must be 0x, proof of immutable state
    #[serde(default = "default_0x")]
    pub trie_hash_after: String,
    /// Always must be 0x, proof of immutable state
    #[serde(default = "default_0x")]
    pub trie_hash_before: String,
    /// All the logs emitted
    pub tx_logs: Vec<Log>,
    /// All the receipts emitted
    pub tx_receipts: Vec<TransactionReceipt>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthApiPayload<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthApiResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: u64,
}

pub async fn simulate_transactions_bundle(
    rpc_url: &str,
    txs_bundle: Vec<CallRequest>,
    block_id: Option<BlockId>,
    opts: EmulateOptions,
) -> Result<EthApiResponse<TransactionSimulationInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    let payload_json = EthApiPayload {
        jsonrpc: "2.0".to_string(),
        method: "cgp_simulateTransactionsBundle".to_string(),
        params: (
            txs_bundle,
            block_id,
            opts.block_overrides.clone(),
            opts.state_overrides.clone(),
            opts.tracing_options.clone(),
        ),
        id: 0,
    };
    let payload_json = serde_json::to_value(&payload_json)?;

    let request = client
        .request(reqwest::Method::POST, rpc_url)
        .headers(headers)
        .json(&payload_json);

    let response = request.send().await?;

    let body = response.text().await?;

    let body: EthApiResponse<TransactionSimulationInfo> = serde_json::from_str(&body)?;
    println!("{:#?}", body);

    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use reth_rpc_types::{
        state::AccountOverride,
        trace::geth::{GethDebugBuiltInTracerType, GethDebugTracerType},
        BlockNumberOrTag,
    };

    use alloy_primitives::U256;

    use super::*;

    const RETH_RPC_MAINNET: &'static str = "https://reth.cgp.xyz/";

    #[tokio::test]
    async fn test_simulate_txs_bundle_call_tracer() {
        let result = simulate_transactions_bundle(
            RETH_RPC_MAINNET,
            serde_json::from_value(serde_json::json!(
                [
                    {
                        "accessList": [],
                        "from": "0x3718ecd4e97f4332f9652d0ba224f222b55ec543",
                        "gasLimit": "0x092a1b00000000",
                        "maxFeePerGas":null,
                        "maxPriorityFeePerGas":null,
                        "to": null,
                        "value": "0x0",
                        "data": ""
                    }
                ]
            ))
            .unwrap(),
            Some(BlockId::Number(BlockNumberOrTag::Pending)),
            EmulateOptions {
                tracing_options: Some(GethDebugTracingOptions {
                    tracer: Some(GethDebugTracerType::BuiltInTracer(
                        GethDebugBuiltInTracerType::CallTracer,
                    )),
                    ..GethDebugTracingOptions::default()
                }),
                state_overrides: Some(
                    // don't forget to fund ETH to specified address
                    // 0x3718ecd4e97f4332f9652d0ba224f222b55ec543 in our case
                    HashMap::from([(
                        "0x3718ecd4e97f4332f9652d0ba224f222b55ec543"
                            .parse()
                            .unwrap(),
                        AccountOverride {
                            balance: Some(U256::from_str("0x5af3107a400fff0").unwrap()),
                            ..AccountOverride::default()
                        },
                    )]),
                ),
                ..EmulateOptions::default()
            },
        )
        .await
        .unwrap();

        // easy non empty check
        assert_ne!(result.result, TransactionSimulationInfo::default());
    }

    #[tokio::test]
    async fn test_simulate_txs_bundle_prestate_tracer() {
        // you can put a list of txs here
        let result = simulate_transactions_bundle(
            RETH_RPC_MAINNET,
            serde_json::from_value(serde_json::json!(
                [
                    {
                        "accessList": [],
                        "from": "0x3718ecd4e97f4332f9652d0ba224f222b55ec543",
                        "gasLimit": "0x092a1b00000000",
                        "maxFeePerGas":null,
                        "maxPriorityFeePerGas":null,
                        "to": null,
                        "value": "0x0",
                        "data": ""
                    }
                ]
            ))
            .unwrap(),
            Some(BlockId::Number(BlockNumberOrTag::Pending)),
            EmulateOptions {
                tracing_options: Some(GethDebugTracingOptions {
                    tracer: Some(GethDebugTracerType::BuiltInTracer(
                        GethDebugBuiltInTracerType::PreStateTracer,
                    )),
                    ..GethDebugTracingOptions::default()
                }),
                state_overrides: Some(
                    // don't forget to fund ETH to specified address
                    // 0x3718ecd4e97f4332f9652d0ba224f222b55ec543 in our case
                    HashMap::from([(
                        "0x3718ecd4e97f4332f9652d0ba224f222b55ec543"
                            .parse()
                            .unwrap(),
                        AccountOverride {
                            balance: Some(U256::from_str("0x5af3107a400fff0").unwrap()),
                            ..AccountOverride::default()
                        },
                    )]),
                ),
                ..EmulateOptions::default()
            },
        )
        .await
        .unwrap();

        // easy non empty check
        assert_ne!(result.result, TransactionSimulationInfo::default());
    }
}
