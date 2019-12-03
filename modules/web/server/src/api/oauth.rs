use actix_web::web::Form;

/// A request to authenticate a user in the database.
struct AuthenticationRequest {
    
}

#[get("/api/oauth/")]
pub async fn authenticate(
