Pod Lifetime Limiter
---

Hi! 👋 So you deal with a crappy application which stops working after some period of time
and you want to restart it every N hours instead of fixing bugs? You found the right
repository!

This program deletes all pods which has a label `pod.kubernetes.io/lifetime`.
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

# Usage

There are two ways to set max lifetime limit for pods.

1. The common way - you can create resource `PodLifetimeLimit`.
```shell
kubectl apply -f - <<EOF
apiVersion: de3.me/v1
kind: PodLifetimeLimit
metadata:
  name: example-limit   
spec:
  maxLifetime: 3600
  selector:
    matchLabels:
      name: example-app         
EOF
```

2. You can patch your deployments or pods via kubectl.

```shell
# Patch deployment
kubectl patch deployment YOUR-DEPLOYMENT --type json \
    -p='[{"op": "add",
          "path": "/spec/template/metadata/labels/pod.kubernetes.io~1lifetime",
          "value": "86400"}]'

# Or patch pod directly
kubectl patch pod YOUR-POD --type json \
    -p='[{"op": "add",
          "path": "/metadata/labels/pod.kubernetes.io~1lifetime",
          "value": "86400"}]'
```

# TODO
- [ ] Write unit tests
- [ ] Add ability to set lifetime limit for job

---

This project is inspired by https://github.com/ptagr/pod-reaper.
Idea of building the similar application looked like a good opportunity to learn Rust :)
