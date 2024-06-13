use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    aud: String,
    sub: String,
    exp: u64,
}

#[test]
fn rsa_decode() {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    let encoding_key = EncodingKey::from_rsa_der(priv_key.to_pkcs1_der().unwrap().as_bytes());
    let decoding_key = DecodingKey::from_rsa_der(pub_key.to_pkcs1_der().unwrap().as_bytes());

    let aud = "users";

    let claims = Claims {
        aud: aud.to_owned(),
        sub: "user@example.com".to_owned(),
        exp: (SystemTime::now() + Duration::from_secs(30))
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let header = Header::new(jsonwebtoken::Algorithm::RS256);

    let jwt = encode(&header, &claims, &encoding_key).expect("Failed to encode JWT");

    let validation = {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[aud.to_owned()]);
        validation.set_required_spec_claims(&["exp", "sub", "aud"]);
        validation
    };

    let token_data: TokenData<Claims> =
        decode(&jwt, &decoding_key, &validation).expect("decoded jwt");

    assert_eq!(token_data.claims.sub, "user@example.com");
}
