#! /usr/bin/env sh

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}"  )" >/dev/null 2>&1 && pwd  )"

sudo $DIR/deploy_docker.sh && $DIR/deploy_gcloud.sh
