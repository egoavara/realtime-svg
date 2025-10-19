import * as pulumi from "@pulumi/pulumi";
import * as k8s from "@pulumi/kubernetes";
import { RealtimeSvg } from "realtime-svg-pulumi";

const config = new pulumi.Config();
const kubeContext = config.require("kubeContext");

const provider = new k8s.Provider("k8s", {
    context: kubeContext,
});

const app = new RealtimeSvg("realtime-svg", {
    provider: provider,
    namespace: "default",
    replicas: 2,
    imageTag: "v0.1.4",
    redisEnabled: true,
    ingressEnabled: false,
    servicePort: 80,
});

export const deploymentName = app.deploymentName;
export const serviceName = app.serviceName;
export const ingressUrl = app.ingressUrl;
export const redisServiceName = app.redisServiceName;
