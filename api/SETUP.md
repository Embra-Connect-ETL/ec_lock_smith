# Local development setup

## Prerequisites
- Make sure to install `musl-tools`.

```bash
sudo apt update && sudo apt install musl-tools
```

### **Steps to Set Up, Build the Image, and Run the Container Locally**

Follow these steps to set up and run the `ec_secrets_management` service using Docker with the provided configuration:

----------

### **1. Build the Rust Binary Locally**

You'll need to ensure the `x86_64-unknown-linux-musl` target is installed and then build the release version.

1.  Add the `musl` target:
    
    ```bash
    rustup target add x86_64-unknown-linux-musl
    ```
    
2.  Build the binary for the `musl` target:
    
    ```bash
    cargo build --release --target x86_64-unknown-linux-musl
    ```
    
    This will produce the binary at:
    
    ```
    ./target/x86_64-unknown-linux-musl/release/ec_secrets_management
    ```
----------

### **2. Create the Dockerfile**

Use the following Dockerfile configuration:

```dockerfile
FROM rust:1.73-slim-bullseye

WORKDIR /app

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

COPY ./target/x86_64-unknown-linux-musl/release/api /app/
COPY ./api/public /app/public/

CMD ["/app/api"]

EXPOSE 8000
```

----------

### **3. Build the Docker Image**

Run the following command to build the Docker image:

```bash
docker build -t ec_secrets_management .
```

----------

### **4. Run the Docker Container**

To run the container locally, map the exposed port (`8000`) to your local machine:

```bash
docker run -p 8000:8000 ec_secrets_management
```

If your application uses environment variables from a `.env` file, pass them using the `--env-file` flag:

```bash
docker run -p 8000:8000 --env-file .env ec_secrets_management
```

----------

### **5. Access the Application**

-   The service will be accessible at: **[http://localhost:8000](http://localhost:8000/)**

----------

### **6. Debugging Tips**

-   Verify the binary is **statically linked** by running:
    
    ```bash
    file ./target/x86_64-unknown-linux-musl/release/ec_secrets_management
    ```
    
    You should see `statically linked` in the output.
    
-   If the container fails to run, check the logs:
    
    ```bash
    docker logs <container-id>
    ```
----------

### **Optional: Testing Locally Without Docker**
To test the binary locally (without Docker), you can run it directly:

```bash
ROCKET_ADDRESS=0.0.0.0 ROCKET_PORT=8000 ./target/x86_64-unknown-linux-musl/release/ec_secrets_management
```

* Alternatively, you can use a similar `Rocket.toml` configuration:

```toml
[default]
# Network settings
address = "0.0.0.0"  # Listen on all network interfaces
port = 8000  # Port number
workers = 16  # Number of threads for request handling (adjust to number of CPU cores)
keep_alive = 5  # Keep-alive timeout in seconds
max_blocking = 512  # Maximum number of blocking operations allowed simultaneously
temp_dir = "/tmp"  # Directory for temporary files
ident = "Rocket"  # Server identifier in responses

# Logging and debugging
log_level = "critical"  # Logging level: "critical", "normal", "debug"
cli_colors = true  # Enable CLI colors for local logs

# Security
ip_header = "X-Real-IP"  # Use reverse proxy header for client IP detection (set to "false" if unused)
 
# Resource limits
[default.limits]
json = 52428800  # Max size for JSON payloads (10 MB)
form = 2097152  # Max size for form submissions (2 MB)
file = 52428800  # Max size for uploaded files (50 MB)

# TLS configuration (uncomment and configure for HTTPS)
[default.tls]
certs = "/path/to/certificate.pem" # Path to TLS certificate
key = "/path/to/private.key" # Path to private key

[global]
# Global overrides for all environments
address = "0.0.0.0"
port = 8000

[global.limits]
json = 52428800
form = 2097152
file = 52428800
```
