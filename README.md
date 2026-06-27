# Scola

> *From /ˈskʰɔ.ɫa/ - Latin for "school"*

---

## Motivation

My school uses an internal system simply called "intranet" for managing timetables and grades. It lacks certain features and has poor uptime. I wanted to explore what it would take to build such a system from scratch - and for an extra challenge, I chose to do it in Rust.

---

## Tech Stack

### Backend

| Crate | Purpose |
|---|---|
| [Rust](https://www.rust-lang.org/) | Programming language |
| [axum](https://github.com/tokio-rs/axum) | HTTP server framework |
| [tokio](https://tokio.rs/) | Async runtime and task management |
| [sqlx](https://github.com/launchbadge/sqlx) | Async database access |
| [serde](https://serde.rs/) | JSON serialization / deserialization |
| [bcrypt](https://crates.io/crates/bcrypt) | Password hashing |
| [cookie](https://crates.io/crates/cookie) & [tower-cookies](https://crates.io/crates/tower-cookies) | Server-managed session cookies |

### Database

- **PostgreSQL**