#! /usr/bin/env bash

DEPLOY_PRIVATE_KEY=$(cat << EOF
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBWqZVmzYAq6iAYNmDVr2ieXT+waSMlSJleIADKOYOQDAAAAJg/b7eqP2+3
qgAAAAtzc2gtZWQyNTUxOQAAACBWqZVmzYAq6iAYNmDVr2ieXT+waSMlSJleIADKOYOQDA
AAAEBpih5n3um+B+4JY2lJ3Z9Top/rfGxbx/Cp46Y4H4rtjlaplWbNgCrqIBg2YNWvaJ5d
P7BpIyVImV4gAMo5g5AMAAAAFW1heGltdW1zdG9ja0BhbmFjb25kYQ==
-----END OPENSSH PRIVATE KEY-----
EOF
)

DEPLOY_HOST="154.53.57.10"
DEPLOY_USER="poe"


ssh-add <(echo "$DEPLOY_PRIVATE_KEY")
ssh -q -o StrictHostKeyChecking=no $DEPLOY_USER@$DEPLOY_HOST << EOF
  docker pull maximumstock2/trade:latest
  export ENV="production"
  export AGE_KEY="AGE-SECRET-KEY-1PH43ETR6TM5J8KKN28APWDRTU3XV3A3HJ5NECUENDZZN77AZ0G4S6VAEAH"
  echo "Environment: \$ENV"

  set -ex
  # rm -rf deployment
  # git clone https://github.com/maximumstock/poe-stash-indexer.git deployment && cd deployment
  cd deployment
  git pull
  make config
  docker-compose -f docker-compose.yaml -f docker-compose.production.yaml build --force-rm reverse-proxy prometheus rabbitmq db
  docker-compose -f docker-compose.yaml -f docker-compose.production.yaml build grafana
  docker-compose -f docker-compose.yaml -f docker-compose.production.yaml stop --timeout 60 trade # give it time to save
  make up-prod
EOF
