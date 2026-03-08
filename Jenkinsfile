// Fetchium — Production CI/CD Pipeline
// Agent: SSH build node on server.zuhabul.com (Rust + AWS CLI pre-installed)
// Triggers: GitHub webhook on push to `production` branch
// Notifications: Discord embeds on success / failure / rollback

pipeline {
  agent { label 'fetchium-build' }

  options {
    timestamps()
    ansiColor('xterm')
    disableConcurrentBuilds(abortPrevious: true)
    buildDiscarder(logRotator(numToKeepStr: '30', artifactNumToKeepStr: '10'))
    timeout(time: 40, unit: 'MINUTES')
  }

  environment {
    CARGO_BIN     = '/home/echo/.cargo/bin'
    PROJECT_PATH  = '/home/echo/projects/Fetchium'
    SERVICE_NAME  = 'fetchium-api'
    BACKUP_DIR    = '/home/echo/.fetchium/backups'
    S3_BUCKET     = 'fetchium'
    S3_PREFIX     = 'releases'
    AWS_REGION    = 'ap-southeast-1'
  }

  stages {

    // ── 1. Checkout ────────────────────────────────────────────────────────────
    stage('Checkout') {
      steps {
        checkout scm
        script {
          env.GIT_SHORT   = sh(script: 'git rev-parse --short HEAD', returnStdout: true).trim()
          env.PKG_VERSION = sh(
            script: "grep '^version' Cargo.toml | head -1 | sed 's/.*= \"//;s/\"//'",
            returnStdout: true
          ).trim()
          echo "Building v${env.PKG_VERSION} @ ${env.GIT_SHORT} on branch ${env.GIT_BRANCH}"
        }
      }
    }

    // ── 2. Lint ────────────────────────────────────────────────────────────────
    stage('Lint') {
      parallel {
        stage('Clippy') {
          steps {
            sh 'export PATH="$CARGO_BIN:$PATH" && cargo clippy -- -D warnings 2>&1'
          }
        }
        stage('Format') {
          steps {
            sh 'export PATH="$CARGO_BIN:$PATH" && cargo fmt --check 2>&1'
          }
        }
      }
    }

    // ── 3. Test ────────────────────────────────────────────────────────────────
    stage('Test') {
      steps {
        sh '''
          export PATH="$CARGO_BIN:$PATH"
          cargo test -- --skip research::pipeline 2>&1
        '''
      }
      post {
        always {
          sh 'export PATH="$CARGO_BIN:$PATH" && cargo test --no-run 2>&1 || true'
        }
      }
    }

    // ── 4. Build release binary ────────────────────────────────────────────────
    stage('Build') {
      steps {
        sh '''
          export PATH="$CARGO_BIN:$PATH"
          cargo build -p fetchium-cli --release 2>&1
          echo "✓ Binary size: $(du -sh target/release/fetchium | cut -f1)"
        '''
      }
    }

    // ── 5. Package & upload to S3 (production branch only) ────────────────────
    stage('Upload') {
      when { branch 'production' }
      environment {
        AWS_ACCESS_KEY_ID     = credentials('fetchium-aws-access-key')
        AWS_SECRET_ACCESS_KEY = credentials('fetchium-aws-secret-key')
      }
      steps {
        sh '''
          set -euo pipefail
          ARCHIVE="fetchium-linux-x64.tar.gz"
          CHECKSUM="${ARCHIVE}.sha256"

          cp target/release/fetchium fetchium-bin
          tar -czf "${ARCHIVE}" -C . fetchium-bin --transform 's/fetchium-bin/fetchium/'
          sha256sum "${ARCHIVE}" > "${CHECKSUM}"

          # Upload versioned + latest alias
          aws s3 cp "${ARCHIVE}"  "s3://${S3_BUCKET}/${S3_PREFIX}/v${PKG_VERSION}/${ARCHIVE}"  --region "${AWS_REGION}"
          aws s3 cp "${CHECKSUM}" "s3://${S3_BUCKET}/${S3_PREFIX}/v${PKG_VERSION}/${CHECKSUM}" --region "${AWS_REGION}"
          aws s3 cp "${ARCHIVE}"  "s3://${S3_BUCKET}/${S3_PREFIX}/latest/${ARCHIVE}"           --region "${AWS_REGION}"
          aws s3 cp "${CHECKSUM}" "s3://${S3_BUCKET}/${S3_PREFIX}/latest/${CHECKSUM}"          --region "${AWS_REGION}"

          rm -f fetchium-bin "${ARCHIVE}" "${CHECKSUM}"
          echo "✓ Uploaded v${PKG_VERSION} to S3 (${S3_BUCKET})"
        '''
      }
    }

    // ── 6. Deploy to production (production branch only) ──────────────────────
    stage('Deploy') {
      when { branch 'production' }
      steps {
        sh '''
          set -euo pipefail
          mkdir -p "${BACKUP_DIR}"

          # Backup running binary
          LIVE="${PROJECT_PATH}/target/release/fetchium"
          PREV="${BACKUP_DIR}/fetchium-prev"
          if [ -f "${LIVE}" ]; then
            cp "${LIVE}" "${PREV}"
            echo "✓ Backed up previous binary → ${PREV}"
          fi

          # Deploy new binary (systemd ExecStart reads from project target/release/)
          cp "$(pwd)/target/release/fetchium" "${LIVE}"
          chmod 755 "${LIVE}"

          # Restart service
          sudo systemctl restart "${SERVICE_NAME}"
          sleep 3
          sudo systemctl is-active "${SERVICE_NAME}" || {
            echo "❌ Service failed to start — triggering rollback"
            exit 1
          }
          echo "✓ ${SERVICE_NAME} restarted successfully"
        '''
      }
      post {
        failure {
          sh '''
            PREV="${BACKUP_DIR}/fetchium-prev"
            LIVE="${PROJECT_PATH}/target/release/fetchium"
            if [ -f "${PREV}" ]; then
              echo "⚠️  Deploy failed — rolling back to previous binary"
              cp "${PREV}" "${LIVE}"
              sudo systemctl restart "${SERVICE_NAME}"
              sleep 3
              sudo systemctl is-active "${SERVICE_NAME}" && echo "✓ Rollback successful" || echo "❌ Rollback also failed — manual intervention required"
            else
              echo "⚠️  No backup found — cannot auto-rollback"
            fi
          '''
          discordNotify(
            title: "⚠️ Fetchium Deploy Rolled Back",
            color: 16744272,  // orange
            description: "Deploy failed and was automatically rolled back.\nBranch: ${GIT_BRANCH}\nCommit: ${GIT_SHORT}"
          )
        }
      }
    }

    // ── 7. Smoke test (production branch only) ────────────────────────────────
    stage('Smoke') {
      when { branch 'production' }
      steps {
        sh '''
          set -euo pipefail

          echo "── /v1/health ──"
          HEALTH=$(curl -sf --max-time 15 ***REMOVED***/v1/health)
          echo "${HEALTH}" | python3 -c "
import sys, json
d = json.load(sys.stdin)
status = d.get('status', 'unknown')
print(f'  status={status}  checks={d.get(\"checks\",{})}')
if status not in ('ok', 'degraded'):
    print('FAIL: unexpected status', status, file=sys.stderr)
    sys.exit(1)
"

          echo "── MCP initialize ──"
          MCP=$(curl -sf --max-time 15 \
            -H 'Content-Type: application/json' \
            -d '"'"'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'"'"' \
            ***REMOVED***/mcp)
          echo "${MCP}" | python3 -c "
import sys, json
d = json.load(sys.stdin)
name = d.get('result', {}).get('serverInfo', {}).get('name', '')
print(f'  serverInfo.name={name!r}')
if not name:
    print('FAIL: no serverInfo.name', file=sys.stderr)
    sys.exit(1)
"
          echo "✓ All smoke tests passed"
        '''
      }
    }
  }

  // ── Post-build notifications ────────────────────────────────────────────────
  post {
    success {
      script {
        if (env.GIT_BRANCH == 'production') {
          discordSend(
            webhookURL: credentials('fetchium-discord-webhook'),
            title: "✅ Fetchium v${env.PKG_VERSION} Deployed",
            description: "Branch: ${env.GIT_BRANCH}\nCommit: ${env.GIT_SHORT}\nBuild: [#${env.BUILD_NUMBER}](${env.BUILD_URL})",
            result: currentBuild.currentResult,
            link: env.BUILD_URL
          )
        }
      }
    }
    failure {
      withCredentials([string(credentialsId: 'fetchium-discord-webhook', variable: 'WEBHOOK')]) {
        sh '''
          curl -sf -X POST "${WEBHOOK}" \
            -H "Content-Type: application/json" \
            -d "{
              \\"embeds\\": [{
                \\"title\\": \\"❌ Fetchium Build #${BUILD_NUMBER} Failed\\",
                \\"description\\": \\"Branch: ${GIT_BRANCH}\\\\nCommit: ${GIT_SHORT}\\\\nFailed stage: ${STAGE_NAME}\\",
                \\"color\\": 15158332,
                \\"url\\": \\"${BUILD_URL}\\"
              }]
            }" || true
        '''
      }
    }
    always {
      cleanWs(
        cleanWhenSuccess: true,
        cleanWhenFailure: false,   // keep workspace on failure for debugging
        cleanWhenAborted: true,
        deleteDirs: true
      )
    }
  }
}
