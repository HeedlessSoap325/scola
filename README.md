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

- **[PostgreSQL](https://www.postgresql.org/)**: For storing users, grades, lessons, etc.
- **[Valkey](https://www.valkey.io/)**: For session and user caching

---

## Getting Started

### Option A - Docker (Recommended)

#### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and able to create containers, images, and volumes
- *(Optional)* A database management tool such as [DBeaver](https://dbeaver.io/) or [pgAdmin](https://www.pgadmin.org/)
- *(Optional)*: A management tool such as [Redis Insight](https://redis.io/insight/) for the Valkey chaching database

#### Steps

**1. Clone the repository**

```shell
git clone https://github.com/HeedlessSoap325/scola.git && cd scola
```

**2. Configure environment variables**

Copy the example file and fill in your values:

```shell
cp .env.example .env
```

| Variable | Description | Required | Default | Notes |
|---|---|---|---|---|
| `POSTGRES_PORT` | Port of the PostgreSQL container | Yes | - | |
| `POSTGRES_USER` | Root PostgreSQL username | Yes | - | Only needed for Docker Compose |
| `POSTGRES_PASSWORD` | Root PostgreSQL password | Yes | - | Only needed for Docker Compose |
| `VALKEY_PORT` | Port of the Valkey container | Yes | - | |
| `BACKEND_PORT` | Port the backend listens on | No | `3000` | Also forwarded to host via Docker Compose |
| `BACKEND_DATABASE_NAME` | Name of the backend database | Yes | - | Created automatically by Docker Compose |
| `BACKEND_DATABASE_USER` | Database user for the backend | Yes | - | Created automatically; cannot be `POSTGRES_USER` |
| `BACKEND_DATABASE_PASSWORD` | Password for `BACKEND_DATABASE_USER` | Yes | - | Set automatically by Docker Compose |
| `BACKEND_DATABASE_HOST` | Database host | No | - | Isn't used in Docker Compose |
| `BACKEND_VALKEY_USER` | Valkey user for the backend | Yes | - | Created automatically |
| `BACKEND_VALKEY_PASSWORD` | Password for `BACKEND_VALKEY_USER` | Yes | - | Set automatically by Docker Compose |
| `BACKEND_VALKEY_HOST` | Valkey host | No | - | Isn't used in Docker Compose |
| `BACKEND_VALKEY_DATABASE` | Number of the Valkey database  | Yes | - | Permissions are automatically granted to `BACKEND_VALKEY_USER` by Docker Compose; Valid indicies range from 0 to 15 |
| `DATABASE_URL` | Full database connection URL | No | - | Overridden by Docker Compose |
| `REDIS_URL` | Full redis / valkey connection URL | No | - | Overridden by Docker Compose |
| `COOKIE_SECRET` | Secret used to sign session cookies | Yes | - | Must be **at least 64 bytes** long |

**3. Start the application**

```shell
docker compose up
```

---

### Option B - Local / Manual

#### Prerequisites

- A running PostgreSQL instance (local or remote)
- A running Valkey / Redis instance (local or remote)
- [Cargo](https://doc.rust-lang.org/cargo/) (Rust's package manager and build tool)
- *(Optional)* A database management tool such as [DBeaver](https://dbeaver.io/) or [pgAdmin](https://www.pgadmin.org/)
- *(Optional)*: A management tool such as [Redis Insight](https://redis.io/insight/) for the Valkey / Redis chaching database
- *(Optional)* The [rest-client VS Code Extension](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) is highly recommended to send requests from *.http* files, and the project is already set up for it.

#### Steps

**1. Clone the repository**

```shell
git clone https://github.com/HeedlessSoap325/scola.git && cd scola
```

**2. Configure environment variables**

```shell
cp .env.example .env
```

| Variable | Description | Required | Default | Notes |
|---|---|---|---|---|
| `POSTGRES_PORT` | Port of the PostgreSQL instance | Yes | - | |
| `POSTGRES_USER` | Root PostgreSQL username | No | - | Not needed for local setup |
| `POSTGRES_PASSWORD` | Root PostgreSQL password | No | - | Not needed for local setup |
| `VALKEY_PORT` | Port of the Valkey instance | Yes | - | |
| `BACKEND_PORT` | Port the backend listens on | No | `3000` | |
| `BACKEND_DATABASE_NAME` | Name of the backend database | Yes | - | Will be created by `sqlx` if it doesn't exist |
| `BACKEND_DATABASE_USER` | Database user for the backend | Yes | - | Must have login privileges; optionally `CREATEDATABASE` for `sqlx database` commands |
| `BACKEND_DATABASE_PASSWORD` | Password for `BACKEND_DATABASE_USER` | Yes | - | |
| `BACKEND_DATABASE_HOST` | Database host | Yes | - | |
| `BACKEND_VALKEY_USER` | Valkey user for the backend | Yes | - | Must be active, have access to all keys and have get, set and del commands enabled on the `BACKEND_VALKEY_DATABASE` database. |
| `BACKEND_VALKEY_PASSWORD` | Password for `BACKEND_VALKEY_USER` | Yes | - | |
| `BACKEND_VALKEY_HOST` | Valkey host | Yes | |
| `BACKEND_VALKEY_DATABASE` | Number of the Valkey database  | Yes | - | Valid indicies range from 0 to 15 |
| `DATABASE_URL` | Full database connection URL | Yes | - | Format: `postgres://${BACKEND_DATABASE_USER}:${BACKEND_DATABASE_PASSWORD}@${BACKEND_DATABASE_HOST}:${POSTGRES_PORT}/${BACKEND_DATABASE_NAME}` |
| `REDIS_URL` | Full valkey / redis connection URL | Yes | - | Format: `(valkey \| redis)://${BACKEND_VALKEY_USER}:${BACKEND_VALKEY_PASSWORD}@${BACKEND_VALKEY_HOST}:${VALKEY_PORT}/${BACKEND_VALKEY_DATABASE}` |
| `COOKIE_SECRET` | Secret used to sign session cookies | Yes | - | Must be **at least 64 bytes** long |

**3. Build the project**

```shell
cargo build
```

**4. Set up the database**

To create and migrate the database from scratch (no existing database):

This will create the database from `DATABSE_URL`

```shell
cargo sqlx database setup
```

To run migrations only on an existing database:

This will use the database from `DATABSE_URL`

```shell
cargo sqlx migrate run
```

**5. Run the server**

```shell
cargo run
```