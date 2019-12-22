# Notedly

A general-purpose note-taking app written in Rust + TypeScript.

## Why should I care?

This project simply serves as a demonstration of Rust's full-stack capabilities. Each of the following technologies, for example, are used in the notedly source code:

* Postgres - backend database
* R2D2 - Rust Postgres connection pool manager / connector
* Diesel - Rust Postgres query builder, compatability layer
* Actix-web - Rust web server
* Tokio, Actix - futures executors
* Bazel - monorepo submodules build manager
* Oauth2-rs - GitHub & Google oauth client
