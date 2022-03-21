mod manager;

use kube::CustomResourceExt;
fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&manager::PodLifetimeLimit::crd()).unwrap()
    )
}
