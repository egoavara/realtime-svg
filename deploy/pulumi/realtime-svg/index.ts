import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";

export interface RealtimeSvgArgs {
    provider?: k8s.Provider;
    namespace?: pulumi.Input<string>;
    replicas?: pulumi.Input<number>;
    imageRepository?: pulumi.Input<string>;
    imageTag?: pulumi.Input<string>;
    imagePullPolicy?: pulumi.Input<string>;
    
    resourcesRequestsCpu?: pulumi.Input<string>;
    resourcesRequestsMemory?: pulumi.Input<string>;
    resourcesLimitsCpu?: pulumi.Input<string>;
    resourcesLimitsMemory?: pulumi.Input<string>;
    
    redisEnabled?: pulumi.Input<boolean>;
    redisPassword?: pulumi.Input<string>;
    redisExternalUrl?: pulumi.Input<string>;
    
    ingressEnabled?: pulumi.Input<boolean>;
    ingressHost?: pulumi.Input<string>;
    ingressPath?: pulumi.Input<string>;
    ingressPathType?: pulumi.Input<string>;
    ingressTlsEnabled?: pulumi.Input<boolean>;
    ingressTlsSecretName?: pulumi.Input<string>;
    
    serviceType?: pulumi.Input<string>;
    servicePort?: pulumi.Input<number>;
    
    configLogLevel?: pulumi.Input<string>;
    configPort?: pulumi.Input<number>;
}

export class RealtimeSvg extends pulumi.ComponentResource {
    public readonly deploymentName: pulumi.Output<string>;
    public readonly serviceName: pulumi.Output<string>;
    public readonly serviceType: pulumi.Output<string>;
    public readonly ingressUrl?: pulumi.Output<string>;
    public readonly redisServiceName?: pulumi.Output<string>;

    constructor(name: string, args: RealtimeSvgArgs, opts?: pulumi.ComponentResourceOptions) {
        super("custom:app:RealtimeSvg", name, {}, opts);

        const appName = "realtime-svg";
        const namespace = args.namespace || "default";
        const replicas = args.replicas || 2;
        const imageRepository = args.imageRepository || "ghcr.io/egoavara/realtime-svg";
        const imageTag = args.imageTag || "v0.1.4";
        const imagePullPolicy = args.imagePullPolicy || "IfNotPresent";
        
        const resourcesRequestsCpu = args.resourcesRequestsCpu || "100m";
        const resourcesRequestsMemory = args.resourcesRequestsMemory || "128Mi";
        const resourcesLimitsCpu = args.resourcesLimitsCpu || "500m";
        const resourcesLimitsMemory = args.resourcesLimitsMemory || "512Mi";
        
        const redisEnabled = args.redisEnabled ?? true;
        const redisPassword = args.redisPassword || "";
        const redisExternalUrl = args.redisExternalUrl || "";
        
        const ingressEnabled = args.ingressEnabled ?? true;
        const ingressHost = args.ingressHost || "realtime-svg.example.com";
        const ingressPath = args.ingressPath || "/";
        const ingressPathType = args.ingressPathType || "Prefix";
        const ingressTlsEnabled = args.ingressTlsEnabled ?? false;
        const ingressTlsSecretName = args.ingressTlsSecretName || "";
        
        const serviceType = args.serviceType || "ClusterIP";
        const servicePort = args.servicePort || 80;
        
        const configLogLevel = args.configLogLevel || "info";
        const configPort = args.configPort || 8080;

        const labels = {
            "app.kubernetes.io/name": appName,
            "app.kubernetes.io/instance": appName,
            "app.kubernetes.io/version": imageTag,
            "app.kubernetes.io/managed-by": "pulumi",
        };

        const backendLabels = {
            ...labels,
            "app.kubernetes.io/component": "backend",
        };

        const redisLabels = {
            ...labels,
            "app.kubernetes.io/component": "redis",
        };

        const redisUrl = pulumi.all([redisEnabled, redisPassword, redisExternalUrl]).apply(([enabled, password, externalUrl]) => 
            enabled 
                ? (password 
                    ? `redis://:${password}@${appName}-redis:6379/` 
                    : `redis://${appName}-redis:6379/`)
                : externalUrl
        );

        const configMap = new k8s.core.v1.ConfigMap(`${appName}-config`, {
            metadata: {
                name: `${appName}-config`,
                namespace: namespace,
                labels: labels,
            },
            data: {
                REDIS_URL: redisUrl,
                LOG_LEVEL: configLogLevel,
                PORT: pulumi.output(configPort).apply(p => p.toString()),
            },
        }, { parent: this, provider: args.provider });

        const secret = new k8s.core.v1.Secret(`${appName}-secret`, {
            metadata: {
                name: `${appName}-secret`,
                namespace: namespace,
                labels: labels,
            },
            type: "Opaque",
            data: {},
        }, { parent: this, provider: args.provider });

        const deployment = new k8s.apps.v1.Deployment(appName, {
            metadata: {
                name: appName,
                namespace: namespace,
                labels: backendLabels,
            },
            spec: {
                replicas: replicas,
                strategy: {
                    type: "RollingUpdate",
                    rollingUpdate: {
                        maxUnavailable: 1,
                        maxSurge: 1,
                    },
                },
                selector: {
                    matchLabels: {
                        "app.kubernetes.io/name": appName,
                        "app.kubernetes.io/instance": appName,
                        "app.kubernetes.io/component": "backend",
                    },
                },
                template: {
                    metadata: {
                        labels: backendLabels,
                    },
                    spec: {
                        containers: [{
                            name: appName,
                            image: pulumi.interpolate`${imageRepository}:${imageTag}`,
                            imagePullPolicy: imagePullPolicy,
                            ports: [{
                                name: "http",
                                containerPort: configPort,
                                protocol: "TCP",
                            }],
                            env: [
                                { name: "HOST", value: "0.0.0.0" },
                                {
                                    name: "REDIS_URL",
                                    valueFrom: {
                                        configMapKeyRef: {
                                            name: configMap.metadata.name,
                                            key: "REDIS_URL",
                                        },
                                    },
                                },
                                {
                                    name: "LOG_LEVEL",
                                    valueFrom: {
                                        configMapKeyRef: {
                                            name: configMap.metadata.name,
                                            key: "LOG_LEVEL",
                                        },
                                    },
                                },
                                {
                                    name: "PORT",
                                    valueFrom: {
                                        configMapKeyRef: {
                                            name: configMap.metadata.name,
                                            key: "PORT",
                                        },
                                    },
                                },
                            ],
                            resources: {
                                requests: {
                                    cpu: resourcesRequestsCpu,
                                    memory: resourcesRequestsMemory,
                                },
                                limits: {
                                    cpu: resourcesLimitsCpu,
                                    memory: resourcesLimitsMemory,
                                },
                            },
                            livenessProbe: {
                                httpGet: {
                                    path: "/health",
                                    port: configPort,
                                },
                                initialDelaySeconds: 10,
                                periodSeconds: 10,
                                timeoutSeconds: 5,
                                failureThreshold: 3,
                            },
                            readinessProbe: {
                                httpGet: {
                                    path: "/ready",
                                    port: configPort,
                                },
                                initialDelaySeconds: 5,
                                periodSeconds: 5,
                                timeoutSeconds: 3,
                                failureThreshold: 3,
                            },
                        }],
                    },
                },
            },
        }, { parent: this, provider: args.provider });

        const service = new k8s.core.v1.Service(appName, {
            metadata: {
                name: appName,
                namespace: namespace,
                labels: backendLabels,
            },
            spec: {
                type: serviceType,
                selector: {
                    "app.kubernetes.io/name": appName,
                    "app.kubernetes.io/instance": appName,
                    "app.kubernetes.io/component": "backend",
                },
                ports: [{
                    name: "http",
                    port: servicePort,
                    targetPort: "http",
                    protocol: "TCP",
                }],
            },
        }, { parent: this, provider: args.provider });

        if (redisEnabled) {
            const redisDeployment = new k8s.apps.v1.Deployment(`${appName}-redis`, {
                metadata: {
                    name: `${appName}-redis`,
                    namespace: namespace,
                    labels: redisLabels,
                },
                spec: {
                    replicas: 1,
                    selector: {
                        matchLabels: {
                            "app.kubernetes.io/name": appName,
                            "app.kubernetes.io/instance": appName,
                            "app.kubernetes.io/component": "redis",
                        },
                    },
                    template: {
                        metadata: {
                            labels: redisLabels,
                        },
                        spec: {
                            containers: [{
                                name: "redis",
                                image: "redis:8-alpine",
                                imagePullPolicy: "IfNotPresent",
                                ports: [{
                                    name: "redis",
                                    containerPort: 6379,
                                    protocol: "TCP",
                                }],
                                args: pulumi.output(redisPassword).apply(pwd => 
                                    pwd
                                        ? ["--requirepass", pwd, "--save", "", "--appendonly", "no", "--maxmemory", "256mb", "--maxmemory-policy", "allkeys-lru"]
                                        : ["--save", "", "--appendonly", "no", "--maxmemory", "256mb", "--maxmemory-policy", "allkeys-lru"]
                                ),
                                resources: {
                                    requests: {
                                        cpu: "100m",
                                        memory: "128Mi",
                                    },
                                    limits: {
                                        cpu: "200m",
                                        memory: "256Mi",
                                    },
                                },
                            }],
                        },
                    },
                },
            }, { parent: this, provider: args.provider });

            const redisService = new k8s.core.v1.Service(`${appName}-redis`, {
                metadata: {
                    name: `${appName}-redis`,
                    namespace: namespace,
                    labels: redisLabels,
                },
                spec: {
                    type: "ClusterIP",
                    selector: {
                        "app.kubernetes.io/name": appName,
                        "app.kubernetes.io/instance": appName,
                        "app.kubernetes.io/component": "redis",
                    },
                    ports: [{
                        name: "redis",
                        port: 6379,
                        targetPort: "redis",
                        protocol: "TCP",
                    }],
                },
            }, { parent: this, provider: args.provider });

            this.redisServiceName = redisService.metadata.name;
        }

        if (ingressEnabled) {
            const tlsConfig = pulumi.all([ingressTlsEnabled, ingressTlsSecretName, ingressHost]).apply(
                ([tlsEnabled, tlsSecret, host]) => 
                    tlsEnabled && tlsSecret
                        ? [{
                            hosts: [host],
                            secretName: tlsSecret,
                        }]
                        : []
            );

            const ingress = new k8s.networking.v1.Ingress(appName, {
                metadata: {
                    name: appName,
                    namespace: namespace,
                    labels: labels,
                },
                spec: {
                    tls: tlsConfig,
                    rules: [{
                        host: ingressHost,
                        http: {
                            paths: [{
                                path: ingressPath,
                                pathType: ingressPathType,
                                backend: {
                                    service: {
                                        name: service.metadata.name,
                                        port: {
                                            number: servicePort,
                                        },
                                    },
                                },
                            }],
                        },
                    }],
                },
            }, { parent: this, provider: args.provider });

            this.ingressUrl = pulumi.all([ingressTlsEnabled, ingressHost]).apply(([tlsEnabled, host]) =>
                tlsEnabled ? `https://${host}` : `http://${host}`
            );
        }

        this.deploymentName = deployment.metadata.name;
        this.serviceName = service.metadata.name;
        this.serviceType = service.spec.type;

        this.registerOutputs({
            deploymentName: this.deploymentName,
            serviceName: this.serviceName,
            serviceType: this.serviceType,
            ingressUrl: this.ingressUrl,
            redisServiceName: this.redisServiceName,
        });
    }
}
