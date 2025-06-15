# 🔐 **Locksmith API Documentation**

**Base URL**  
`http://localhost:8000`

## 🔐 Authentication

Most endpoints (except `/setup` and `/login`) require authentication using a **JWT**. You can authenticate in **two ways**:

### Option 1: Bearer Token (Authorization Header)

Include the JWT in the `Authorization` header like so:

```http
Authorization: Bearer <your-jwt-token>
```


### Option 2: Cookie-Based Authentication

After a successful login, the server returns a `Set-Cookie` header with the token:

```http
Set-Cookie: auth_token=<TOKEN>; HttpOnly; Path=/; SameSite=Strict;
```

You can then include it in subsequent requests like:

```http
Cookie: auth_token=<TOKEN>
```

This works automatically in most browser-based clients and tools like Postman (if `Send cookies` is enabled) or when using `curl`:

```bash
curl -X GET http://localhost:8000/retrieve/vault/entries \
  --cookie "auth_token=<TOKEN>"

```

> 🔐 **Note**:  
> The cookie is `HttpOnly`, meaning it's inaccessible to JavaScript in a browser — enhancing security against XSS attacks.

----------

### Important CORS Note

To support cookies across domains (e.g., frontend at `localhost:3000`, backend at `localhost:8000`):

-   Set `Access-Control-Allow-Credentials: true`
-   Set `Access-Control-Allow-Origin: <frontend-origin>`
-   Ensure client requests use `withCredentials: true`

* Both **Client** & **API** are currently hosted on the **same domain**.

## 👤 User Endpoints

### ➕ Create User

**POST** `/setup`  
Create a new user account.

```json
{
  "email": "user@example.com",
  "password": "ThisShouldBeKeptSecret"
}
```

**Response:**

```json
{
  "id": "684d5b36cd2cc6f9df1ae31d",
  "email": "user@example.com",
  "created_at": "2025-06-14T12:00:00Z"
}
```


### 🔐 Login

**POST** `/login`  
Authenticate user and receive JWT token.

```json
{
  "email": "user@example.com",
  "password": "ThisShouldBeKeptSecret"
}
```

**Response:**

```json
{
  "token": "<TOKEN>"
}
```

----------

### 🔓 Logout

**POST** `/logout`  
Invalidates user session (JWT blacklist, if implemented).  
Requires `Authorization` header.



### 👁️‍🗨️ Get User by Email

**GET** `/users/{email}`  
Returns a user document (no password).

**Response:**

```json
{
  "id": "684d5b36cd2cc6f9df1ae31d",
  "email": "user@example.com",
  "created_at": "2025-06-14T12:00:00Z"
}
```

### ✏️ Update User

**PUT** `/update/{user_id}`  
Update email and/or password.

```json
{
  "email": "new@example.com",
  "password": "UpdatedPassword123"
}
```

**Response:**

```json
{
  "id": "684d5b36cd2cc6f9df1ae31d",
  "email": "new@example.com",
  "updated_at": "2025-06-14T14:01:00Z"
}
```


### ❌ Delete User

**DELETE** `/delete/user/{user_id}`  
Deletes a user and associated vault entries.

**Response:**

```json
{
  "status": 200,
  "message": "User deleted successfully. 3 vault entries removed."
}
```



## 🔐 Vault Endpoints

> ⚠️ All Vault endpoints require authentication and use the **user's email (sub)** claim internally.

----------

### ➕ Create Vault Entry

**POST** `/create/vault/entry`

```json
{
  "key": "test",
  "value": "ThisShouldBeKeptSecret"
}
```

✅ Uses the token's `sub` claim to identify creator — no need to pass `created_by`.

**Response:**

```json
{
  "status": 200,
  "message": "Vault entry created successfully"
}
```

### 📋 List All Vault Entries

**GET** `/retrieve/vault/entries`

Returns metadata (no values) for entries created by the authenticated user.

**Response:**

```json
[
  {
    "id": "684ea535b5abcb414f2afdc5",
    "key": "test",
    "created_by": "user@example.com",
    "created_at": "2025-06-14T12:30:00Z"
  }
]
```

### 🔍 Get Entry by Key

**GET** `/retrieve/vault/entry/key/{key}`

Returns decrypted secret value for matching key (if owned by user).

**Response:**

```json
"ThisShouldBeKeptSecret"
```

### 🔍 Get Entry by ID

**GET** `/retrieve/vault/entries/{id}`

Returns full vault entry by ID (if owned by user).

**Response:**

```json
{
  "key": "test",
  "value": "ThisShouldBeKeptSecret"
}
```

### 🔍 Get Entries by Author (Email)

**GET** `/retrieve/vault/entry/{email}`

Returns vault metadata for a given user **(used internally or by admins)**.  
Regular users will only see their own entries.

**Response:**

```json
[
  {
    "id": "684ea535b5abcb414f2afdc5",
    "key": "test",
    "created_by": "user@example.com",
    "created_at": "2025-06-14T12:30:00Z"
  }
]
```

### ❌ Delete Vault Entry

**DELETE** `/delete/{vault_entry_id}`

Deletes the entry if the authenticated user is the creator.

**Response:**

```json
{
  "status": 200,
  "message": "Vault entry deleted successfully"
}
```

## 🧪 Test Tokens Easily

Use Postman or CLI:

```bash
curl -X POST http://localhost:8000/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "ThisShouldBeKeptSecret"}'
```

Then copy the token and add it to `Authorization: Bearer <token>` for all protected routes.

## 📝 Notes

-   All IDs (users and vault entries) are MongoDB `ObjectId` strings.
-   All timestamps are UTC ISO 8601 format.
-   Sensitive data like secrets are encrypted at rest and decrypted at request time.
 