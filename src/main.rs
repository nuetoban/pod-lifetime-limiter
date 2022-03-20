use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, DeleteParams, ListParams, ResourceExt},
    Client,
};
use tokio::time::{sleep, Duration};

const LIFETIME_LABEL: &str = "pod.kubernetes.io/lifetime";
const DESCRIPTION: &str =
    "This program restarts all pods which has a label 'pod.kubernetes.io/lifetime'.
Label value should be in seconds, like 'pod.kubernetes.io/lifetime=86400' - 24 hours.
Candidates to delete are determined by the following approach:

1. The operator iterates over all containers inside the pod.
2. It founds the container with maximum lifetime.
3. It compares (start time + label value) to current time.
4. If the first expression is less than second, the pod will be deleted.";
const SLEEP_SECONDS: u64 = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=info");
    println!("{}", DESCRIPTION);
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let list_params = ListParams::default().labels(LIFETIME_LABEL);
    let pods: Api<Pod> = Api::all(client.clone());

    loop {
        let pods_to_delete = pods
            .list(&list_params)
            .await?
            .into_iter()
            .filter(|pod| Some("Running".to_string()) == pod.clone().status.and_then(|s| s.phase))
            .filter(|pod| {
                pod.clone()
                    .status
                    .and_then(|status| status.container_statuses)
                    .and_then(|statuses| {
                        statuses
                            .into_iter()
                            .filter_map(|cs| cs.state?.running?.started_at)
                            .min()
                    })
                    .map(|time| {
                        pod.labels()
                            .get(LIFETIME_LABEL)
                            .and_then(|label| match label.parse::<u64>() {
                                Ok(0) => None,
                                Ok(v) => Some(v),
                                Err(_) => None,
                            })
                            .map(|d| {
                                time.0 + chrono::Duration::seconds(d as i64) < chrono::Utc::now()
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .map(|pod| (pod.name(), pod.namespace()));

        // Delete pods
        for pod in pods_to_delete {
            let pods_api_namespaced: Api<Pod>;
            match pod.1 {
                Some(v) => pods_api_namespaced = Api::namespaced(client.clone(), &v),
                None => pods_api_namespaced = Api::default_namespaced(client.clone()),
            }
            tracing::info!("Deleting pod {}", pod.0);
            pods_api_namespaced
                .delete(pod.0.as_str(), &DeleteParams::default())
                .await?
                .map_left(|pdel| {
                    assert_eq!(pdel.name(), pod.0);
                });
        }

        tracing::info!("Sleeping {} second", SLEEP_SECONDS);
        sleep(Duration::from_secs(SLEEP_SECONDS)).await;
    }
}
