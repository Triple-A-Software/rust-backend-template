# Rust backend template

This is an example implementation for our frontend template, written in rust.

## Features

- :lock: Authentication
- :family: User-management
- :key: Access-token-management
- :electric_plug: Live user-status over websockets
- :adult: Avatar storage for users
- :closed_lock_with_key: Password-reset flow
- :sparkles: Activity-tracking

## Tech-stack

This implementation is written in Rust with the following crates used for major parts:
- [SQLx](https://crates.io/crates/sqlx) for database access and migration management
- [Axum](https://crates.io/crates/axum) as our server framework
- [lettre](https://crates.io/crates/lettre) for sending emails over SMTP
- [serde](https://crates.io/crates/serde) for serialization
