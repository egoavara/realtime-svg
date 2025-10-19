# Helm Deployment Guide

This guide covers deploying realtime-svg using Helm, the package manager for Kubernetes.

## Prerequisites

- Kubernetes cluster (1.24+)
- Helm 3.x installed
- kubectl configured to access your cluster

## Installing Helm

If you don't have Helm installed:

```bash
# macOS
brew install helm

# Linux
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Windows
choco install kubernetes-helm

# Verify installation
helm version
```

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/egoavara/realtime-svg.git
cd realtime-svg
```

### 2. Update Dependencies

The chart uses Bitnami Redis as a dependency. Download it first:

```bash
cd deploy/helm/realtime-svg
helm dependency update
cd ../../..
```

### 3. Install with Default Values

```bash
helm install realtime-svg ./deploy/helm/realtime-svg
```

This will deploy:
- 2 application replicas
- In-cluster Redis (Bitnami chart, in-memory mode for development)
- ClusterIP service
- Ingress (you'll need an ingress controller)

### 4. Verify Deployment

```bash
# Check all resources
kubectl get all -l app.kubernetes.io/name=realtime-svg

# Check pods
kubectl get pods

# View logs
kubectl logs -l app.kubernetes.io/name=realtime-svg -f
```

### 5. Access the Application

**Option A: Port Forward (Quick Test)**
```bash
kubectl port-forward svc/realtime-svg 8080:80
# Visit http://localhost:8080
```

**Option B: Through Ingress**
```bash
# Get ingress IP/hostname
kubectl get ingress

# Add to /etc/hosts if using minikube/kind
echo "$(minikube ip) realtime-svg.example.com" | sudo tee -a /etc/hosts

# Visit http://realtime-svg.example.com
```

## Installation Scenarios

### Development Environment

```bash
# Update dependencies first
helm dependency update ./deploy/helm/realtime-svg

# Install for development
helm install realtime-svg ./deploy/helm/realtime-svg \
  --set redis.enabled=true \
  --set redis.auth.enabled=false \
  --set redis.master.persistence.enabled=false \
  --set ingress.enabled=false \
  --set service.type=NodePort \
  --set replicas=1
```

**Access**: 
```bash
# For minikube
minikube service realtime-svg --url

# For kind/k3s
kubectl port-forward svc/realtime-svg 8080:80
```

### Staging Environment

Create `staging-values.yaml`:
```yaml
namespace: staging

image:
  tag: v0.1.5  # Use specific version

replicas: 2

redis:
  enabled: false
  external:
    host: redis-staging.example.com
    password: staging-redis-password

ingress:
  enabled: true
  host: staging.realtime-svg.example.com
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
```

Deploy:
```bash
helm install realtime-svg ./deploy/helm/realtime-svg \
  --namespace staging \
  --create-namespace \
  -f staging-values.yaml
```

### Production Environment

Create `production-values.yaml`:
```yaml
namespace: production

image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.5  # Pin to specific stable version
  pullPolicy: IfNotPresent

replicas: 5  # High availability

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

redis:
  enabled: false  # Use external managed Redis
  external:
    host: redis.production.internal
    port: 6379
    password: secure-production-password  # Use from secret instead!

ingress:
  enabled: true
  host: realtime-svg.mycompany.com
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    enabled: true
    secretName: realtime-svg-tls

podAnnotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "8080"

affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchExpressions:
          - key: app.kubernetes.io/name
            operator: In
            values:
            - realtime-svg
        topologyKey: kubernetes.io/hostname
```

**Secure Deployment (using external secrets)**:
```bash
# Don't put passwords in values.yaml!
# Use Helm's --set flag with values from vault/secret manager

kubectl create secret generic realtime-svg-redis \
  --from-literal=password=$(vault read -field=password secret/redis/production)

helm dependency update ./deploy/helm/realtime-svg

helm install realtime-svg ./deploy/helm/realtime-svg \
  --namespace production \
  --create-namespace \
  -f production-values.yaml \
  --set redis.external.password="" \
  --set-string 'secrets.apiKey='
```

## Customization

### Custom Image

```bash
helm install realtime-svg ./deploy/helm/realtime-svg \
  --set image.repository=myregistry.io/realtime-svg \
  --set image.tag=custom-v1.0.0 \
  --set image.pullPolicy=Always
```

### Custom Resource Limits

```bash
helm install realtime-svg ./deploy/helm/realtime-svg \
  --set resources.requests.cpu=200m \
  --set resources.requests.memory=256Mi \
  --set resources.limits.cpu=1000m \
  --set resources.limits.memory=1Gi
```

### Enable TLS

```bash
# Requires cert-manager installed
helm install realtime-svg ./deploy/helm/realtime-svg \
  --set ingress.enabled=true \
  --set ingress.host=realtime-svg.example.com \
  --set ingress.tls.enabled=true \
  --set ingress.tls.secretName=realtime-svg-tls \
  --set ingress.annotations."cert-manager\.io/cluster-issuer"=letsencrypt-prod
```

### NodePort Service

```bash
helm install realtime-svg ./deploy/helm/realtime-svg \
  --set service.type=NodePort \
  --set service.nodePort=30080
```

## Upgrading

### Update Image Version

```bash
helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  --set image.tag=v0.2.0
```

### Scale Replicas

```bash
helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  --set replicas=10
```

### Upgrade with New Values File

```bash
helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  -f updated-values.yaml
```

### Upgrade Specific Values

```bash
helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  --reuse-values \
  --set image.tag=v0.2.0 \
  --set replicas=5
```

## Rollback

```bash
# View release history
helm history realtime-svg

# Rollback to previous version
helm rollback realtime-svg

# Rollback to specific revision
helm rollback realtime-svg 3
```

## Uninstallation

```bash
# Uninstall release (keeps history)
helm uninstall realtime-svg

# Uninstall and purge history
helm uninstall realtime-svg --wait

# Uninstall from specific namespace
helm uninstall realtime-svg --namespace production
```

## Validation and Testing

### Lint Chart

```bash
helm lint ./deploy/helm/realtime-svg
```

### Template Rendering

```bash
# Render templates locally
helm template realtime-svg ./deploy/helm/realtime-svg

# Render with custom values
helm template realtime-svg ./deploy/helm/realtime-svg \
  -f my-values.yaml

# Render and save to file
helm template realtime-svg ./deploy/helm/realtime-svg \
  -f my-values.yaml > rendered.yaml
```

### Dry Run

```bash
# Simulate installation
helm install realtime-svg ./deploy/helm/realtime-svg \
  --dry-run --debug

# Test upgrade
helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  --dry-run --debug
```

### Verify Installation

```bash
# Check release status
helm status realtime-svg

# Get values
helm get values realtime-svg

# Get all manifests
helm get manifest realtime-svg

# Get release notes
helm get notes realtime-svg
```

## Troubleshooting

### Release Already Exists

```bash
# Error: cannot re-use a name that is still in use

# Solution 1: Use different release name
helm install realtime-svg-v2 ./deploy/helm/realtime-svg

# Solution 2: Uninstall existing release
helm uninstall realtime-svg
```

### Image Pull Errors

```bash
# Check pod events
kubectl describe pod -l app.kubernetes.io/name=realtime-svg

# Common causes:
# - Image doesn't exist
# - Wrong image tag
# - Private registry without pullSecrets

# Solution: Add image pull secret
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=myuser \
  --docker-password=mytoken

helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  --set image.pullSecrets[0].name=regcred
```

### Redis Connection Failed

```bash
# Check Redis pod (if using in-cluster)
kubectl logs -l app.kubernetes.io/component=redis

# Check application logs
kubectl logs -l app.kubernetes.io/name=realtime-svg

# Verify ConfigMap
kubectl get configmap realtime-svg-config -o yaml

# If using external Redis, test connectivity
kubectl run redis-test --rm -it --image=redis:alpine -- \
  redis-cli -h <redis-host> -p 6379 ping
```

### Ingress Not Working

```bash
# Check ingress resource
kubectl describe ingress realtime-svg

# Check ingress controller logs
kubectl logs -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx

# Verify ingress controller is installed
kubectl get pods -n ingress-nginx

# For minikube, enable ingress addon
minikube addons enable ingress
```

### Pods Stuck in Pending

```bash
# Check pod events
kubectl describe pod -l app.kubernetes.io/name=realtime-svg

# Common causes:
# - Insufficient resources
# - Node selector/affinity not matching
# - PVC not available (shouldn't happen for realtime-svg)

# Solution: Adjust resource requests
helm upgrade realtime-svg ./deploy/helm/realtime-svg \
  --set resources.requests.cpu=50m \
  --set resources.requests.memory=64Mi
```

## Best Practices

### 1. Version Pinning

Always pin image versions in production:
```yaml
image:
  tag: v0.1.5  # Don't use 'latest'
```

### 2. Use Values Files

Don't use long `--set` commands. Use values files:
```bash
helm install realtime-svg ./deploy/helm/realtime-svg -f prod-values.yaml
```

### 3. Separate Namespaces

Use different namespaces for different environments:
```bash
helm install realtime-svg ./deploy/helm/realtime-svg \
  --namespace production \
  --create-namespace
```

### 4. External Secrets

Never commit sensitive data to values.yaml:
```bash
# Use Kubernetes secrets
kubectl create secret generic realtime-svg-secrets \
  --from-literal=redis-password=mypassword

# Reference in Helm
--set redis.password=""  # Leave empty, use secret instead
```

### 5. Resource Limits

Always set resource limits to prevent resource starvation:
```yaml
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi
```

### 6. Health Checks

Use the default liveness and readiness probes (already configured).

### 7. Monitoring

Add Prometheus annotations:
```yaml
podAnnotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "8080"
```

## Advanced Topics

### Helm Hooks

The chart currently doesn't use hooks, but you can add pre/post install hooks if needed.

### Multi-Cluster Deployment

```bash
# Deploy to multiple clusters
for CLUSTER in cluster1 cluster2 cluster3; do
  kubectl config use-context $CLUSTER
  helm upgrade --install realtime-svg ./deploy/helm/realtime-svg \
    -f values-$CLUSTER.yaml
done
```

### GitOps with ArgoCD

```yaml
# argocd-application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: realtime-svg
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/egoavara/realtime-svg
    targetRevision: main
    path: deploy/helm/realtime-svg
    helm:
      valueFiles:
      - values.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

## Next Steps

- [Configuration Reference](./configuration-reference.md)
- [Architecture Overview](./architecture.md)
- [kubectl Deployment Guide](./kubectl-guide.md) (alternative method)
- [Terraform Deployment Guide](./terraform-guide.md) (IaC alternative)
