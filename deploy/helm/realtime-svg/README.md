# realtime-svg Helm Chart

Helm chart for deploying realtime-svg - Real-time SVG streaming application on Kubernetes.

## Prerequisites

- Kubernetes 1.24+
- Helm 3.x
- (Optional) Ingress controller (nginx, traefik, etc.)

## Installation

### Quick Start

```bash
# Install with default values (includes in-memory Redis)
helm install realtime-svg ./realtime-svg

# Install in specific namespace
helm install realtime-svg ./realtime-svg --namespace production --create-namespace

# Install with custom values
helm install realtime-svg ./realtime-svg -f my-values.yaml
```

### Common Installation Scenarios

#### Development (with in-cluster Redis)

```bash
helm install realtime-svg ./realtime-svg \
  --set redis.enabled=true \
  --set ingress.enabled=false \
  --set service.type=NodePort
```

#### Production (with external Redis - Recommended)

```bash
helm install realtime-svg ./realtime-svg \
  --set redis.enabled=false \
  --set redis.external.url=redis://:my-secret-password@redis.production.svc:6379/ \
  --set replicas=5 \
  --set ingress.enabled=true \
  --set ingress.host=realtime-svg.mycompany.com \
  --set ingress.tls.enabled=true \
  --set ingress.tls.secretName=realtime-svg-tls
```

#### Production (with in-cluster Redis + password)

```bash
helm install realtime-svg ./realtime-svg \
  --set redis.enabled=true \
  --set redis.password=secure-redis-password \
  --set replicas=5 \
  --set ingress.enabled=true \
  --set ingress.host=realtime-svg.mycompany.com \
  --set ingress.tls.enabled=true \
  --set ingress.tls.secretName=realtime-svg-tls
```

## Configuration

See [values.yaml](./values.yaml) for all available configuration options.

### Key Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `namespace` | Kubernetes namespace | `default` |
| `image.repository` | Container image repository | `ghcr.io/egoavara/realtime-svg` |
| `image.tag` | Image tag | `v0.1.4` |
| `replicas` | Number of replicas | `2` |
| `redis.enabled` | Deploy in-cluster Redis | `true` |
| `redis.password` | Redis password (in-cluster, optional) | `""` |
| `redis.external.url` | External Redis URL (required if enabled=false) | `""` |
| `ingress.enabled` | Enable Ingress | `true` |
| `ingress.host` | Ingress hostname | `realtime-svg.example.com` |
| `service.type` | Service type | `ClusterIP` |

### Full Values Example

```yaml
namespace: production
image:
  repository: ghcr.io/egoavara/realtime-svg
  tag: v0.1.5
replicas: 3
resources:
  requests:
    cpu: 200m
    memory: 256Mi
  limits:
    cpu: 1000m
    memory: 1Gi
redis:
  enabled: false
  external:
    url: redis://:my-password@redis.external.svc:6379/
ingress:
  enabled: true
  host: realtime-svg.example.com
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    enabled: true
    secretName: realtime-svg-tls
```

## Upgrading

```bash
# Upgrade with new values
helm upgrade realtime-svg ./realtime-svg -f my-values.yaml

# Upgrade with specific parameters
helm upgrade realtime-svg ./realtime-svg \
  --set replicas=5 \
  --set image.tag=v0.1.5

# Rollback to previous version
helm rollback realtime-svg
```

## Uninstallation

```bash
# Uninstall release
helm uninstall realtime-svg

# Uninstall from specific namespace
helm uninstall realtime-svg --namespace production
```

## Verification

```bash
# Check deployment status
kubectl get all -l app.kubernetes.io/name=realtime-svg

# Check pods
kubectl get pods -l app.kubernetes.io/name=realtime-svg

# View logs
kubectl logs -l app.kubernetes.io/name=realtime-svg -f

# Test application
kubectl port-forward svc/realtime-svg 8080:80
# Then visit http://localhost:8080
```

## Troubleshooting

### Pods not starting

```bash
# Check pod events
kubectl describe pod -l app.kubernetes.io/name=realtime-svg

# Common issues:
# - Image pull errors: Check image.repository and tag
# - Resource limits: Check resources.requests and limits
# - Redis connection: Verify redis.host and redis.password
```

### Ingress not working

```bash
# Check ingress status
kubectl describe ingress realtime-svg

# Verify ingress controller is installed
kubectl get pods -n ingress-nginx

# Check ingress annotations match your controller
```

### Redis connection errors

```bash
# If using in-cluster Redis (redis.enabled=true):
kubectl logs -l app.kubernetes.io/name=realtime-svg -l app.kubernetes.io/component=redis

# If using external Redis (redis.enabled=false):
# - Verify redis.external.url is correct (e.g., redis://host:6379/)
# - Include password in URL if required (e.g., redis://:password@host:6379/)
# - Check network policies allow connection
# - Test connectivity: kubectl run redis-test --rm -it --image=redis:7-alpine -- redis-cli -u <REDIS_URL> ping
```

## Values Validation

```bash
# Lint the chart
helm lint ./realtime-svg

# Template without installing
helm template realtime-svg ./realtime-svg

# Template with custom values
helm template realtime-svg ./realtime-svg -f my-values.yaml

# Dry-run install
helm install realtime-svg ./realtime-svg --dry-run --debug
```

## Notes

### Redis Configuration

**Development/Testing** (default):
- In-memory Redis using official `redis:8-alpine` image
- ⚠️ **Data will be lost** when the Redis Pod restarts
- No authentication by default
- Suitable for local testing only

**Production Options**:

1. **External Redis** (recommended):
   ```yaml
   redis:
     enabled: false
     external:
       url: redis://:secure-password@redis.production.svc:6379/
   ```
   
   For managed Redis services:
   ```yaml
   redis:
     enabled: false
     external:
       # AWS ElastiCache
       url: redis://my-cluster.cache.amazonaws.com:6379/
       
       # Azure Cache for Redis (with TLS)
       url: rediss://:password@my-cache.redis.cache.windows.net:6380/
       
       # Google Cloud Memorystore
       url: redis://10.0.0.3:6379/
   ```

2. **In-cluster Redis with password** (not recommended for production):
   ```yaml
   redis:
     enabled: true
     password: secure-password  # Stored in ConfigMap as part of REDIS_URL
   ```
   
   ⚠️ **Warning**: In-cluster Redis without persistence is not suitable for production workloads.

### Security Considerations

- Store sensitive values (passwords, API keys) in Kubernetes Secrets, not in values.yaml
- Use `--set` with values from secure vaults during deployment
- Consider using sealed-secrets or external-secrets-operator for production

## Support

- GitHub: https://github.com/egoavara/realtime-svg
- Issues: https://github.com/egoavara/realtime-svg/issues
- Documentation: https://github.com/egoavara/realtime-svg/tree/main/docs
