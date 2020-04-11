#!/usr/bin/env bash

# The user must be logged in to docker in order to
# deploy to a registry
grep -q "docker login" <<< docker pull mitsukom/test:latest > /dev/null 2>&1 || docker login || exit 1

# We must be in a directory with a dockerfile, or have
# moved to a directory where we should find one
[ -f Dockerfile ] || cd ..

# Build the notedly server image
docker build -t notedly-server .

VERSION=$(git rev-parse HEAD)

# Publish the image under the ID of the last commit, as well as the "latest" tag
docker tag notedly-server "mitsukom/notedly-server:$VERSION"
docker tag notedly-server mitsukom/notedly-server:latest

# Publish to gcloud
if command -v gcloud; then
    echo "Looks like you have gcloud installed! Publishing to gcloud now..."

    docker tag notedly-server "us.grc.io/notedly-627656760499/notedly-server:$VERSION"
    docker tag notedly-server us.grc.io/notedly-627656760499/notedly-server:latest

    gcloud docker -- push "us.grc.io/notedly-627656760499/notedly-server:$VERSION"
    gcloud docker -- push us.grc.io/notedly-627656760499/notedly-server:latest
fi


docker push "mitsukom/notedly-server:$VERSION"
docker push mitsukom/noteldy-server:latest
