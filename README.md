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

## API

To get started with the Notedly API, grab an API token from the [Notedly GitHub authentication endpoint](https://notedly.app/api/oauth/login/github)

## Building from source

In order to build the Notedly source code, you'll need the PostgreSQL development library. For example:

```zsh
sudo pacman -S postgresql-libs
```

or, on a debian-based distribution:

```zsh
sudo apt-get install postgresql-devel
```
