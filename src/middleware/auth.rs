use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::models::Claims;

pub fn extract_claims(req: &ServiceRequest, jwt_secret: &str) -> Result<Claims, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ErrorUnauthorized("Missing authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ErrorUnauthorized("Invalid authorization format"))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ErrorUnauthorized("Invalid token"))?;

    Ok(token_data.claims)
}

pub fn validate_token(req: &ServiceRequest, jwt_secret: &str) -> Result<(), Error> {
    let claims = extract_claims(req, jwt_secret)?;
    req.extensions_mut().insert(claims);
    Ok(())
}
