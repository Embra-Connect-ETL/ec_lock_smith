use rocket::catch;

#[catch(400)]
pub async fn bad_request() -> &'static str {
    "Bad Request."
}

#[catch(401)]
pub async fn unauthorized() -> &'static str {
    "Unauthorized access."
}

#[catch(403)]
pub async fn forbidden() -> &'static str {
    "Forbidden: You don't have permission to access this resource."
}

#[catch(404)]
pub async fn not_found() -> &'static str {
    "Resource not found."
}

#[catch(405)]
pub async fn method_not_allowed() -> &'static str {
    "Method Not Allowed."
}

#[catch(408)]
pub async fn request_timeout() -> &'static str {
    "Request Timeout."
}

#[catch(409)]
pub async fn conflict() -> &'static str {
    "The request could not be completed due to a conflict."
}

#[catch(413)]
pub async fn payload_too_large() -> &'static str {
    "Payload Too Large."
}

#[catch(415)]
pub async fn unsupported_media_type() -> &'static str {
    "Unsupported Media Type."
}

#[catch(418)]
pub async fn teapot() -> &'static str {
    "I'm a teapot. ☕"
}

#[catch(429)]
pub async fn too_many_requests() -> &'static str {
    "Too Many Requests. Slow down!"
}

#[catch(500)]
pub async fn internal_error() -> &'static str {
    "Internal Server Error."
}

#[catch(502)]
pub async fn bad_gateway() -> &'static str {
    "Bad Gateway."
}

#[catch(503)]
pub async fn service_unavailable() -> &'static str {
    "Service Unavailable."
}

#[catch(504)]
pub async fn gateway_timeout() -> &'static str {
    "Gateway Timeout."
}
