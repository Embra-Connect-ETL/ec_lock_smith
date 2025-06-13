# Locksmith - Embra Connect's Secrets Management Platform

![Locksmith by Embra Connect](https://github.com/Embra-Connect-ETL/ec_secrets_management/blob/master/previews/ecs_console.png?raw=true)

## Overview

Locksmith is an open-source **secrets management platform** built by **Embra Connect**. Designed with security and scalability in mind, Locksmith enables developers to securely store, retrieve, and manage sensitive data such as API keys, credentials, and encryption secrets.

### **Key Features**

-   🔐 **Secure Vault** – Store secrets with strong encryption.
-   ⚡ **Fast Retrieval** – Optimized for high-speed access.
-   🛠 **API-First Design** – Easily integrate with existing applications.
-   🏗 **Built with Rust** – Performance and memory safety at its core.
-   🏛 **Role-Based Access Control (RBAC)** – Fine-grained permissions for users and services.
-   🔄 **Audit Logging** – Keep track of all access and modifications.
-   🌍 **Open Source** – Community-driven development.

## Getting Started

### **Prerequisites**

-   Rust (latest stable version)
-   Cargo package manager
-   MongoDB (for storage)
-   Docker (optional, for containerized deployment)

### **Installation**

Clone the repository and navigate to the project directory:

```sh
 git clone https://github.com/Embra-Connect-ETL/ec_secrets_management.git
 cd ec_secrets_management
```

### **Building the Project**

```sh
 cargo build --release
```

### **Running the Server**

```sh
 cargo run
```

### **Environment Variables**

Configure the `.env` file with necessary values:

```env
# Operation
# Working with host machine
ECS_DATABASE_URL=mongodb://ec_root:ec_root@localhost:27017/embra_connect_dev?authSource=admin

# Working with docker compose
# ECS_DATABASE_URL=mongodb://ec_root:ec_root@ec_secrets_management_storage:27017/embra_connect_dev?authSource=admin
ECS_DATABASE_NAME=embra_connect_dev

# An encryption key can be genrated via the following command: openssl rand -base64 32
# Expected output -> IRwTgHBtmblSfAXpYOuvf4ZIhSY32JoP8TLIxeLuCrg=
ECS_ENCRYPTION_KEY=
# An authentication key can be genrated via the following command: openssl rand -base64 32
# Expected output -> HEJpH886G0gArUNIYK7CLXfvOSKHBAnlJM3rVw/Tfdg=
ECS_AUTHENTICATION_KEY=
ECS_SIGNING_KEY=

# Storage
MONGO_INITDB_ROOT_USERNAME=ec_root # Do NOT use in production
MONGO_INITDB_ROOT_PASSWORD=ec_root # Do NOT use in production
MONGO_INITDB_DATABASE=embra_connect_dev # Do NOT use in production
```

## API Usage

### **Authentication**

All API requests must be authenticated using JWT tokens.

#### **Login**

```http
POST /login
```

**Request Body:**

```json
{
  "email": "user@domain.com",
  "password": "yourpassword"
}
```

**Response:**

```json
{
  "status": 200,
  "token": "your_auth_token"
}
```

### **Retrieve Secrets**

```http
GET /retrieve/vault/entries
```

**Response:**

```json
[
  {
    "id": "1",
    "name": "API_KEY",
    "value": "sk-123456",
    "created_at": "2025-03-22T12:34:56Z"
  }
]
```

## License

Locksmith is licensed under the **MIT License**. See [LICENSE](https://chatgpt.com/c/LICENSE) for more details.

## Contributing

We welcome contributions! To get started:

1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature-name`).
3.  Commit your changes (`git commit -m "Add new feature"`).
4.  Push to your branch (`git push origin feature-name`).
5.  Create a Pull Request.

## Contact & Community

For discussions, issues, and support:

-   **GitHub Issues**: [Report Issues](https://github.com/Embra-Connect-ETL/ec_secrets_management/issues)
-   **Embra Connect Website**: [www.embraconnect.com](https://www.embraconnect.com/)

----------

🔑 **Locksmith** – Your secure vault for managing secrets, built with Rust.
