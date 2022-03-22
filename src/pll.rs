use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    kind = "PodLifetimeLimit",
    group = "de3.me",
    version = "v1",
    shortname = "pll",
    namespaced
)]
#[kube(status = "PodLifetimeLimitStatus")]
pub struct PodLifetimeLimitSpec {
    pub selector: PodLifetimeLimitSelector,
    pub max_lifetime: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodLifetimeLimitSelector {
    pub match_labels: std::collections::BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodLifetimeLimitStatus {
    pub related_pods_count: u64,
}
