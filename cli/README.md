This guide will explain how to authenticate, manage users, and manage secrets with clear examples.


## 🔐 `ec_lock_smith` CLI Usage Guide

### 📦 Tool Info

-   **Name:** `ec_lock_smith`    
-   **Version:** 1.0
-   **Description:** CLI tool for interacting with the _Embra Connect Lock Smith_ secrets manager service.

## 🔑 Authentication
Before using any commands that require access to your vault or user account, **login** first:

```bash
ec_lock_smith login -e user@example.com -p password123
```

If successful, your session will be stored for further authenticated commands.


## 👤 User Management

### ▶ Create a new user

```bash
ec_lock_smith users create -e newuser@example.com -p strongpassword
```

### ▶ List all users

```bash
ec_lock_smith users list
```

### ▶ Get a specific user by ID

```bash
ec_lock_smith users list -i <user_id>
```

### ▶ Delete a user by ID

```bash
ec_lock_smith users delete -i <user_id>
```

## 🔐 Secret Management

### ▶ Create a new secret

```bash
ec_lock_smith secret create -k API_KEY -v super-secret-value
```

### ▶ List all your secrets

```bash
ec_lock_smith secret list
```

### ▶ Retrieve a specific secret by ID

```bash
ec_lock_smith secret list -i <secret_id>
```

### ▶ Delete a secret by ID

```bash
ec_lock_smith secret delete -i <secret_id>
```

## 🧭 Notes

-   You **must be logged in** to run any `users` or `secret` subcommands.
-   All user-specific and secret-specific data are scoped to your authenticated session.
-   Secrets are encrypted and bound to your user ID in the backend.
    
## 🛠 Example Workflow

```bash
# Login
ec_lock_smith login -e alice@example.com -p hunter2

# Create a secret
ec_lock_smith secret create -k DB_PASSWORD -v s3cr3t

# List secrets
ec_lock_smith secret list

# Fetch one by ID
ec_lock_smith secret list -i 665f01bcabb3c7c347a4c9c5

# Delete it
ec_lock_smith secret delete -i 665f01bcabb3c7c347a4c9c5

# Create a user
ec_lock_smith users create -e bob@example.com -p pass123
```