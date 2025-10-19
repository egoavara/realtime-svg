# GitHub Actions Workflow Contract

**Feature**: 007-k8s-deployment-packaging  
**Version**: 1.0.0

## Purpose

배포 패키지의 검증과 릴리스를 자동화하기 위한 GitHub Actions 워크플로우 요구사항을 정의합니다.

## Workflow: Validate Deployment Packages

### Trigger

```yaml
on:
  push:
    paths:
      - 'deploy/**'
      - '.github/workflows/validate-deployments.yml'
  pull_request:
    paths:
      - 'deploy/**'
```

### Jobs

#### 1. validate-helm

**목적**: Helm 차트 검증

```yaml
jobs:
  validate-helm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Helm
        uses: azure/setup-helm@v3
        with:
          version: 'v3.13.0'
      
      - name: Lint Helm Chart
        run: helm lint deploy/helm/realtime-svg
      
      - name: Template Helm Chart
        run: |
          helm template test deploy/helm/realtime-svg \
            --values deploy/helm/realtime-svg/values.yaml \
            --namespace test
      
      - name: Validate Rendered Manifests
        run: |
          helm template test deploy/helm/realtime-svg | \
            kubectl apply --dry-run=client -f -
```

**성공 조건**:
- `helm lint` 경고 없음
- 템플릿 렌더링 성공
- 렌더링된 YAML이 Kubernetes API 스키마 준수

---

#### 2. validate-kubectl

**목적**: kubectl 매니페스트 검증

```yaml
  validate-kubectl:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Validate Manifests
        run: |
          kubectl apply --dry-run=client -f deploy/kubernetes/
      
      - name: Check YAML Syntax
        uses: instrumenta/kubeval-action@master
        with:
          files: deploy/kubernetes/
```

**성공 조건**:
- 모든 YAML 파일 파싱 성공
- `kubectl apply --dry-run` 통과
- kubeval 스키마 검증 통과

---

#### 3. validate-terraform

**목적**: Terraform 설정 검증

```yaml
  validate-terraform:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v2
        with:
          terraform_version: 1.6.0
      
      - name: Terraform Format Check
        run: terraform -chdir=deploy/terraform fmt -check
      
      - name: Terraform Init
        run: terraform -chdir=deploy/terraform init -backend=false
      
      - name: Terraform Validate
        run: terraform -chdir=deploy/terraform validate
```

**성공 조건**:
- 코드 포맷팅 규칙 준수
- `terraform init` 성공
- `terraform validate` 통과

---

#### 4. validate-pulumi

**목적**: Pulumi 프로그램 검증

```yaml
  validate-pulumi:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install Dependencies
        run: cd deploy/pulumi && npm install
      
      - name: TypeScript Check
        run: cd deploy/pulumi && npx tsc --noEmit
      
      - name: Setup Pulumi
        uses: pulumi/setup-pulumi@v2
      
      - name: Pulumi Preview (Dry Run)
        run: cd deploy/pulumi && pulumi preview --non-interactive
        env:
          PULUMI_ACCESS_TOKEN: ${{ secrets.PULUMI_ACCESS_TOKEN }}
```

**성공 조건**:
- npm 의존성 설치 성공
- TypeScript 컴파일 에러 없음
- Pulumi preview 실행 성공

---

#### 5. integration-test

**목적**: 실제 Kubernetes 클러스터에 배포 테스트

```yaml
  integration-test:
    runs-on: ubuntu-latest
    needs: [validate-helm, validate-kubectl, validate-terraform]
    steps:
      - uses: actions/checkout@v4
      
      - name: Start Minikube
        uses: medyagh/setup-minikube@latest
        with:
          kubernetes-version: 1.28.0
      
      - name: Build Images
        run: |
          eval $(minikube docker-env)
          docker build -t realtime-svg/backend:test ./crates/backend
          docker build -t realtime-svg/frontend:test ./crates/frontend
      
      - name: Deploy with Helm
        run: |
          helm install test deploy/helm/realtime-svg \
            --set image.backend.tag=test \
            --set image.frontend.tag=test \
            --set image.backend.pullPolicy=Never \
            --set image.frontend.pullPolicy=Never \
            --wait --timeout 5m
      
      - name: Check Deployment Health
        run: |
          kubectl wait --for=condition=ready pod -l app=backend --timeout=120s
          kubectl wait --for=condition=ready pod -l app=frontend --timeout=120s
      
      - name: Test Application
        run: |
          kubectl port-forward svc/frontend 8080:80 &
          sleep 5
          curl -f http://localhost:8080 || exit 1
      
      - name: Cleanup
        if: always()
        run: helm uninstall test
```

**성공 조건**:
- Helm 배포 성공
- 모든 Pod Ready 상태
- 애플리케이션 HTTP 응답 확인

---

## Workflow: Release Deployment Packages

### Trigger

```yaml
on:
  release:
    types: [published]
```

### Jobs

#### 1. package-helm

**목적**: Helm 차트를 패키징하고 릴리스에 첨부

```yaml
jobs:
  package-helm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Update Chart Version
        run: |
          sed -i "s/version: .*/version: ${{ github.ref_name }}/" \
            deploy/helm/realtime-svg/Chart.yaml
      
      - name: Package Chart
        run: |
          helm package deploy/helm/realtime-svg \
            --destination ./artifacts
      
      - name: Upload to Release
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./artifacts/realtime-svg-${{ github.ref_name }}.tgz
          asset_name: realtime-svg-helm-${{ github.ref_name }}.tgz
          asset_content_type: application/gzip
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

#### 2. publish-docs

**목적**: 배포 문서를 GitHub Pages에 게시

```yaml
  publish-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Documentation
        run: |
          mkdir -p _site/deployment
          cp docs/deployment/* _site/deployment/
          cp deploy/helm/realtime-svg/README.md _site/deployment/helm.md
          cp deploy/kubernetes/README.md _site/deployment/kubectl.md
          cp deploy/terraform/README.md _site/deployment/terraform.md
          cp deploy/pulumi/README.md _site/deployment/pulumi.md
      
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./_site
```

---

## Required Secrets

| Secret Name | Purpose | Required By |
|-------------|---------|-------------|
| `PULUMI_ACCESS_TOKEN` | Pulumi 미리보기 실행 | validate-pulumi |
| `GITHUB_TOKEN` | 릴리스 자산 업로드, Pages 배포 | package-helm, publish-docs |

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `HELM_VERSION` | v3.13.0 | Helm CLI 버전 |
| `TERRAFORM_VERSION` | 1.6.0 | Terraform CLI 버전 |
| `KUBERNETES_VERSION` | 1.28.0 | Minikube K8s 버전 |
| `NODE_VERSION` | 20 | Node.js 버전 (Pulumi) |

## Status Badges

배포 패키지 검증 상태를 README에 표시:

```markdown
[![Validate Deployments](https://github.com/username/realtime-svg/actions/workflows/validate-deployments.yml/badge.svg)](https://github.com/username/realtime-svg/actions/workflows/validate-deployments.yml)
```

## Workflow File Locations

```
.github/
└── workflows/
    ├── validate-deployments.yml  # 배포 패키지 검증
    └── release-deployments.yml   # 릴리스 자동화
```
