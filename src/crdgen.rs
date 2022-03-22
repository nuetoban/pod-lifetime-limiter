mod pll;

use kube::CustomResourceExt;
fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&pll::PodLifetimeLimit::crd()).unwrap()
    )
}
