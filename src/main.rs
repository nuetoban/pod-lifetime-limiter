mod pll;

use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, DeleteParams, ListParams, ResourceExt},
    Client,
};
use std::collections::HashSet;
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

fn parse_lifetime_label_and_log_error(label: &str, pod_name: String) -> Option<u64> {
    match label.parse::<u64>() {
        Ok(0) => {
            tracing::error!("{}: label value should be greather than 0", pod_name);
            None
        }
        Err(e) => {
            tracing::error!("{}: cannot parse label value as u64: {}", pod_name, e,);
            None
        }
        Ok(v) if v < SLEEP_SECONDS => {
            tracing::warn!(
                "{}: it makes no sense to set max pod lifetime less than {}",
                pod_name,
                SLEEP_SECONDS,
            );
            Some(v)
        }
        Ok(v) => Some(v),
    }
}

fn pod_is_running(pod: &Pod) -> bool {
    Some("Running".to_string()) == pod.clone().status.and_then(|s| s.phase)
}

fn pod_lifetime_is_over(pod: &Pod) -> bool {
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
                .and_then(|label| parse_lifetime_label_and_log_error(label, pod.name()))
                .map(|d| time.0 + chrono::Duration::seconds(d as i64) < chrono::Utc::now())
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

async fn find_expired_pods_by_label(
    client: &kube::Client,
) -> impl Iterator<Item = (String, String)> {
    let list_params = ListParams::default().labels(LIFETIME_LABEL);
    let pods: Api<Pod> = Api::all(client.clone());

    pods.list(&list_params)
        .await
        .unwrap()
        .into_iter()
        .filter(pod_is_running)
        .filter(pod_lifetime_is_over)
        .map(|pod| (pod.name(), pod.namespace().unwrap()))
}

async fn find_expired_pods_by_crd(client: &kube::Client) -> impl Iterator<Item = (String, String)> {
    // Retrieve PorLifetimeLimit resources
    let plls: Api<manager::PodLifetimeLimit> = Api::all(client.clone());
    let list_params = ListParams::default();
    let all_plls = plls.list(&list_params).await.unwrap().into_iter();

    // Get all pods with theis labels
    let pods: Api<Pod> = Api::all(client.clone());
    pods
        .list(&list_params)
        .await
        .unwrap()
        .into_iter()
        .filter(pod_is_running)
        .filter(move |pod| {
            // Iterate over PLLs
            for pll in all_plls.clone() {
                let labels = pll.spec.selector.match_labels;

                // If Pod contains all labels which PLL selects, return true
                let mut result = true;
                for (k, v) in labels.into_iter() {
                    if !pod.labels().contains_key(&k) {
                        result = false;
                        break;
                    }
                    if pod.labels()[&k] != v {
                        result = false;
                        break;
                    }
                }
                if result {
                    // Compare lifetime
                    return pod
                        .clone()
                        .status
                        .and_then(|status| status.container_statuses)
                        .and_then(|statuses| {
                            statuses
                                .into_iter()
                                .filter_map(|cs| cs.state?.running?.started_at)
                                .min()
                        })
                        .map(|time| {
                            time.0 + chrono::Duration::seconds(pll.spec.max_lifetime as i64)
                                < chrono::Utc::now()
                        })
                        .unwrap_or(false);
                };
            }

            false
        })
        .map(|pod| (pod.name(), pod.namespace().unwrap()))
}

async fn delete_pods(
    client: &kube::Client,
    iter: impl IntoIterator<Item = (String, String)>,
) -> Result<i32, kube::Error> {
    let mut deleted_pods_count = 0;
    let mut deleted = HashSet::new();

    for (name, namespace) in iter {
        if deleted.contains(&(name.clone(), namespace.clone())) {
            continue;
        }
        let pods_api_namespaced: Api<Pod> = Api::namespaced(client.clone(), namespace.as_str());
        tracing::info!("Deleting pod {}/{}", namespace, name);
        pods_api_namespaced
            .delete(name.as_str(), &DeleteParams::default())
            .await?
            .map_left(|pdel| {
                assert_eq!(pdel.name(), name);
                assert_eq!(pdel.namespace(), Some(namespace.clone()));
            });
        deleted_pods_count += 1;
        deleted.insert((name, namespace));
    }

    Ok(deleted_pods_count)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", DESCRIPTION);
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;

    loop {
        let pods_to_delete = find_expired_pods_by_label(&client)
            .await
            .chain(find_expired_pods_by_crd(&client).await);
        let delete_result = delete_pods(&client, pods_to_delete).await;

        match delete_result {
            Ok(n) => tracing::info!("Sleeping {} seconds; deleted {} pods", SLEEP_SECONDS, n),
            Err(e) => tracing::error!("Sleeping {} seconds; got error: {}", SLEEP_SECONDS, e),
        }

        sleep(Duration::from_secs(SLEEP_SECONDS)).await;
    }
}
