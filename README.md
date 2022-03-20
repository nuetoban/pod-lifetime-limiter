Pod Lifetime Limiter
---

This program restarts all pods which has a label `pod.kubernetes.io/lifetime`.
Label value should be in seconds, like `pod.kubernetes.io/lifetime=86400` - 24 hours.
Candidates to delete are determined by the following approach:

1. The operator iterates over all containers inside the pod.
2. It founds the container with maximum lifetime.
3. It compares (start time + label value) to current time.
4. If the first expression is less than second, the pod will be deleted.

# Installation

The operator can be installed via Helm.
```shell
git clone https://github.com/nuetoban/pod-lifetime-limiter.git
helm install pod-lifetime-limiter ./pod-lifetime-limiter/helm -n kube-system
```

---

This project is inspired by https://github.com/ptagr/pod-reaper.
Idea of building the similar application looked like a good opportunity to learn Rust :)
