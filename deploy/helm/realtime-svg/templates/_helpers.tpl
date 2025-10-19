{{/*
Expand the name of the chart.
*/}}
{{- define "realtime-svg.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "realtime-svg.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "realtime-svg.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "realtime-svg.labels" -}}
helm.sh/chart: {{ include "realtime-svg.chart" . }}
{{ include "realtime-svg.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "realtime-svg.selectorLabels" -}}
app.kubernetes.io/name: {{ include "realtime-svg.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "realtime-svg.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "realtime-svg.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Redis URL
*/}}
{{- define "realtime-svg.redisUrl" -}}
{{- if .Values.redis.enabled }}
{{- $host := printf "%s-redis" (include "realtime-svg.fullname" .) }}
{{- $password := .Values.redis.password }}
{{- if $password }}
{{- printf "redis://:%s@%s:6379/" $password $host }}
{{- else }}
{{- printf "redis://%s:6379/" $host }}
{{- end }}
{{- else }}
{{- required "redis.external.url is required when redis.enabled is false" .Values.redis.external.url }}
{{- end }}
{{- end }}

{{/*
Image name
*/}}
{{- define "realtime-svg.image" -}}
{{- printf "%s:%s" .Values.image.repository .Values.image.tag }}
{{- end }}
