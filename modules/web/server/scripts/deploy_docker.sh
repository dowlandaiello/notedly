#!/usr/bin/env bash

# The user must be logged in to docker in order to
# deploy to a registry
grep -q "docker login" <<< $(docker pull mitsukom/test:latest > /dev/null 2>&1) || docker login || exit 1
# We must be in a directory with a dockerfile, or have
# moved to a directory where we should find one
[ -f Dockerfile ] || cd ..

# Build the notedly server image
docker build -t notedly-server .

# Publish the image under the ID of the last commit, as well as the "latest" tag
docker tag notedly-server mitsukom/notedly-server:$(git rev-parse HEAD)
docker tag notedly-server mitsukom/notedly-server:latest

docker push mitsukom/notedly-server:$(git rev-parse HEAD)
docker push mitsukom/noteldy-server:latest
