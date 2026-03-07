pipeline {
  agent any

  options {
    timestamps()
    ansiColor('xterm')
    disableConcurrentBuilds()
  }

  environment {
    DEPLOY_HOST = credentials('fetchium-deploy-host')
    DEPLOY_USER = credentials('fetchium-deploy-user')
    DEPLOY_PATH = credentials('fetchium-deploy-path')
    DEPLOY_ENV_FILE = credentials('fetchium-deploy-env-file')
  }

  stages {
    stage('Checkout') {
      steps {
        checkout scm
      }
    }

    stage('Verify') {
      parallel {
        stage('Rust') {
          steps {
            sh 'cargo test -p fetchium-api -p fetchium-mcp -p fetchium-cli'
            sh 'cargo clippy -p fetchium-api -p fetchium-mcp -p fetchium-cli -- -D warnings'
          }
        }

        stage('Packaging') {
          steps {
            sh 'python3 -m unittest adapters.langchain.tests.test_retriever adapters.crewai.tests.test_tool'
            sh 'npm pack --dry-run ./packages/npm'
            sh 'bash -n scripts/deploy.sh'
          }
        }
      }
    }

    stage('Deploy') {
      when {
        branch 'production'
      }
      steps {
        sshagent(credentials: ['fetchium-deploy-ssh']) {
          sh '''
            rsync -az --delete \
              --exclude '.git' \
              --exclude 'target' \
              ./ "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}"
          '''

          sh '''
            ssh "${DEPLOY_USER}@${DEPLOY_HOST}" \
              "cd '${DEPLOY_PATH}' && chmod +x scripts/deploy.sh && FETCHIUM_ENV_FILE='${DEPLOY_ENV_FILE}' ./scripts/deploy.sh"
          '''
        }
      }
    }

    stage('Smoke') {
      when {
        branch 'production'
      }
      steps {
        sh '''
          curl --fail --silent --show-error --max-time 15 https://api.fetchium.com/v1/health
          curl --fail --silent --show-error --max-time 15 \
            -H 'Content-Type: application/json' \
            -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
            https://api.fetchium.com/mcp
        '''
      }
    }
  }
}
