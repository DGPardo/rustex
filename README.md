# Basic Exchange System

A scalable exchange system built with Rust using **Actix Web**, **Tokio**, **Diesel**, and **Tarpc**. This project provides a **REST API** for order matching and database interaction, with a **WebSocket API** under development.

## Features

- **Microservices Architecture**: The system is designed to be modular, with independent services handling order matching and database interactions.
- **Horizontal Scalability**: Supports currency pair-based scaling (e.g., `BTC_USD` and `BTC_GBP` can be hosted on separate servers and databases).
- **REST API with JWT Authentication**: Secure API endpoints protected with **JSON Web Tokens (JWT)**.
- **PostgreSQL with Diesel ORM**: Efficient database access and migrations.
- **Async Runtime with Tokio**: Ensures high performance and concurrency.
- **Work-In-Progress WebSocket API**: Future support for real-time trading updates.

## Tech Stack

- **Rust**
- **Actix Web** (REST API framework)
- **Tokio** (Async runtime)
- **Diesel** (PostgreSQL ORM)
- **JSON Web Tokens (JWT)** (Authentication)
- **PostgreSQL** (Database)
- **Tarpc** (RPC Framework)

## Getting Started

The application allows for plenty customization dividing the services between
different exchange markets, databasesm API servers and microservices. For a basic setup follow the
instructions below.

### Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install)
- Install [Docker](https://www.docker.com/)
- Install PostgreSQL and create the necessary databases
- Set up environment variables

### Installation

Clone the repository:

```sh
git clone https://github.com/your-username/basic-exchange.git
cd basic-exchange
```

Set up the database:

```sh
cargo install diesel_cli --no-default-features --features postgres
diesel setup
```

Run the application with docker (basic setup):

```sh
docker compose --profile debug up
```

## Configuration

Environment variables are defined in `.env_template`. Please duplicate the file into `.env` and edit as required.
In order to serve the public REST API over HTTPS, you will need to provide TLS Certificates.
See the [mkcert](https://github.com/FiloSottile/mkcert) project if you need to generate your own.

## API Endpoints

### Authentication

- `POST /auth/login` → Authenticate user and return JWT (WIP)
The plan is to depend on a third party provider in the long term.
Currently it just generates a JWT Token.

### Orders

- `POST /orders/sell` → Create a new order
- `GET /orders/{id}` → Get order details (WIP)
- `DELETE /orders/{id}` → Cancel an order (WIP)

## WebSocket API (WIP)

WebSockets will be used for real-time market updates and order book changes.

## Scalability & Deployment

- **Microservices Deployment**: Each currency pair (e.g., `BTC_USD`, `BTC_GBP`) can be deployed independently.
- **Database Sharding**: The PostgreSQL database can be split per currency pair for scalability.
- **Containerization**: Docker and Kubernetes can be used for orchestration.

## License

MIT License. See `LICENSE` for details.

## Contact

For any questions or issues, please open an issue in this repository or reach out via email.

