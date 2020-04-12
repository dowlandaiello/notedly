#! /usr/bin/env sh

# We must be in a directory with a dockerfile, or have
# moved to a directory where we should find one
[ -f Dockerfile ] || cd ..

# Remove an existing cluster
kubectl delete -f ../../../deployments/api.yaml

# Spawn a cluster
kubectl create -f ../../../deployments/api.yaml
