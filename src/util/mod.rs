// use std::time::SystemTime;
//
// use jsonwebtoken::{
//     decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
// };
// use pwhash::bcrypt;
// use serde::{Deserialize, Serialize};
// use uuid::Uuid;

pub mod controllers;

// pub type Hash = String;
// pub fn password_hash(s: String) -> Option<Hash> {
//     bcrypt::hash(&s).ok()
// }
// pub fn password_verify(s: String, hashed_s: Hash) -> bool {
//     bcrypt::verify(&s, &hashed_s)
// }
//
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct UserJWT {
//     pub username: String,
//     pub id: Uuid,
// }
//
// #[derive(Clone, Debug, Serialize, Deserialize)]
// struct MetaUserJWT {
//     pub user: UserJWT,
//     pub exp: u64,
// }
//
// pub fn encode_jwt_user(user_jwt: UserJWT) -> Result<String, String> {
//     let secret_key = std::env::var("JWT_SECRET_KEY")
//         .map_err(|err| format!("Cant access \"JWT_SECRET_KEY\" var: {err}"))?;
//
//     let expiration = std::env::var("JWT_EXPIRATION")
//         .map_err(|err| format!("Cant access \"JWT_EXPIRATION\" var: {err}"))?
//         .parse::<u64>()
//         .map_err(|err| format!("Cant parse \"JWT_EXPIRATION\" var: {err}"))?;
//
//     let meta_user_jwt = MetaUserJWT {
//         user: user_jwt,
//         exp: SystemTime::now()
//             .duration_since(SystemTime::UNIX_EPOCH)
//             .unwrap()
//             .as_secs()
//             + expiration,
//     };
//
//     encode(
//         &Header::default(),
//         &meta_user_jwt,
//         &EncodingKey::from_secret(secret_key.as_ref()),
//     )
//     .map_err(|err| format!("Can encode user structure: {err}"))
// }
//
// pub fn decode_jwt_user(token: String) -> Result<UserJWT, String> {
//     let secret_key = std::env::var("JWT_SECRET_KEY")
//         .map_err(|err| format!("Cant access \"JWT_SECRET_KEY\" var: {err}"))?;
//
//     let user_jwt: TokenData<MetaUserJWT> = decode(
//         &token,
//         &DecodingKey::from_secret(secret_key.as_ref()),
//         &Validation::new(Algorithm::HS256),
//     )
//     .map_err(|err| format!("Can decode user structure: {err}"))?;
//
//     let now = SystemTime::now()
//         .duration_since(SystemTime::UNIX_EPOCH)
//         .unwrap()
//         .as_secs();
//
//     if now > user_jwt.claims.exp {
//         return Err("Expired".to_string());
//     }
//
//     Ok(user_jwt.claims.user)
// }
