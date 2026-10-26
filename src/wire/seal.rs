use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::pkcs8::DecodePublicKey;
use rsa::{Oaep, RsaPublicKey};
use serde_json::{json, Value};
use sha2::Sha256;

use crate::support::consts::net::{ENVELOPE_TYPE, KEX_KID, KEX_PUBLIC_KEY};
use crate::support::error::{AppError, AppResult};

const TAG_LEN: usize = 16;
const IV_LEN: usize = 12;

pub struct Sealed {
    pub text: String,
    pub reply: Vec<u8>,
    pub request_id: String,
}

pub fn seal(method: &str, path: &str, query: &str, body: Option<&Value>) -> AppResult<Sealed> {
    let key = RsaPublicKey::from_public_key_pem(KEX_PUBLIC_KEY)
        .map_err(|_| AppError::network("the envelope key could not be read"))?;

    let mut content = [0u8; 32];
    let mut reply = [0u8; 32];
    let mut iv = [0u8; IV_LEN];
    let mut stamp = [0u8; 24];
    OsRng.fill_bytes(&mut content);
    OsRng.fill_bytes(&mut reply);
    OsRng.fill_bytes(&mut iv);
    OsRng.fill_bytes(&mut stamp);

    let request_id = URL_SAFE_NO_PAD.encode(stamp);
    let header = json!({
        "alg": "RSA-OAEP-256",
        "enc": "A256GCM",
        "kid": KEX_KID,
        "typ": ENVELOPE_TYPE,
    });
    let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());

    let payload = json!({
        "v": 1,
        "requestId": request_id,
        "method": method.to_ascii_uppercase(),
        "path": path,
        "query": query,
        "replyKey": URL_SAFE_NO_PAD.encode(reply),
        "body": body.cloned().unwrap_or(Value::Null),
    });

    let wrapped = key
        .encrypt(&mut OsRng, Oaep::new::<Sha256>(), &content)
        .map_err(|_| AppError::network("the envelope key refused the secret"))?;

    let cipher = Aes256Gcm::new_from_slice(&content)
        .map_err(|_| AppError::network("the envelope secret is not usable"))?;
    let sealed = cipher
        .encrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: payload.to_string().as_bytes(),
                aad: header_b64.as_bytes(),
            },
        )
        .map_err(|_| AppError::network("the request could not be sealed"))?;

    if sealed.len() <= TAG_LEN {
        return Err(AppError::network("the request could not be sealed"));
    }
    let (data, tag) = sealed.split_at(sealed.len() - TAG_LEN);

    let text = [
        header_b64,
        URL_SAFE_NO_PAD.encode(wrapped),
        URL_SAFE_NO_PAD.encode(iv),
        URL_SAFE_NO_PAD.encode(data),
        URL_SAFE_NO_PAD.encode(tag),
    ]
    .join(".");

    Ok(Sealed {
        text,
        reply: reply.to_vec(),
        request_id,
    })
}

pub fn unseal(text: &str, reply: &[u8]) -> AppResult<Value> {
    let parts: Vec<&str> = text.trim().split('.').collect();
    if parts.len() != 5 {
        return Err(AppError::network("the reply was not sealed"));
    }
    let iv = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| AppError::network("the reply was not sealed"))?;
    if iv.len() != IV_LEN {
        return Err(AppError::network("the reply was not sealed"));
    }
    let mut body = URL_SAFE_NO_PAD
        .decode(parts[3])
        .map_err(|_| AppError::network("the reply was not sealed"))?;
    let tag = URL_SAFE_NO_PAD
        .decode(parts[4])
        .map_err(|_| AppError::network("the reply was not sealed"))?;
    body.extend_from_slice(&tag);

    let cipher = Aes256Gcm::new_from_slice(reply)
        .map_err(|_| AppError::network("the reply secret is not usable"))?;
    let plain = cipher
        .decrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: &body,
                aad: parts[0].as_bytes(),
            },
        )
        .map_err(|_| AppError::network("the reply could not be opened"))?;

    serde_json::from_slice(&plain).map_err(|_| AppError::network("the reply could not be read"))
}

pub fn seal_reply(value: &Value, reply: &[u8]) -> AppResult<String> {
    let mut iv = [0u8; IV_LEN];
    OsRng.fill_bytes(&mut iv);
    let header = json!({"alg": "dir", "enc": "A256GCM", "typ": "newera-response+jwe"});
    let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
    let cipher = Aes256Gcm::new_from_slice(reply)
        .map_err(|_| AppError::network("the reply secret is not usable"))?;
    let sealed = cipher
        .encrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: value.to_string().as_bytes(),
                aad: header_b64.as_bytes(),
            },
        )
        .map_err(|_| AppError::network("the reply could not be sealed"))?;
    let (data, tag) = sealed.split_at(sealed.len() - TAG_LEN);
    Ok([
        header_b64,
        String::new(),
        URL_SAFE_NO_PAD.encode(iv),
        URL_SAFE_NO_PAD.encode(data),
        URL_SAFE_NO_PAD.encode(tag),
    ]
    .join("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sealed_request_has_five_parts() {
        let sealed = seal("GET", "/users/me", "", None).unwrap();
        assert_eq!(sealed.text.split('.').count(), 5);
        assert_eq!(sealed.reply.len(), 32);
        assert!(!sealed.request_id.is_empty());
        assert!(!sealed.text.contains('='));
    }

    #[test]
    fn two_requests_never_share_a_request_id() {
        let first = seal("GET", "/tasks", "", None).unwrap();
        let second = seal("GET", "/tasks", "", None).unwrap();
        assert_ne!(first.request_id, second.request_id);
        assert_ne!(first.reply, second.reply);
    }

    #[test]
    fn a_body_survives_the_seal() {
        let body = json!({"identityId": "abc", "platform": "ANDROID"});
        let sealed = seal("POST", "/auth/login", "", Some(&body)).unwrap();
        assert!(sealed.text.len() > 200);
    }

    #[test]
    fn a_sealed_reply_opens_again() {
        let reply = [7u8; 32];
        let value = json!({"v": 1, "status": 200, "body": {"code": 200}});
        let text = seal_reply(&value, &reply).unwrap();
        let opened = unseal(&text, &reply).unwrap();
        assert_eq!(opened["status"], 200);
    }

    #[test]
    fn another_secret_cannot_open_the_reply() {
        let text = seal_reply(&json!({"v": 1}), &[1u8; 32]).unwrap();
        assert!(unseal(&text, &[2u8; 32]).is_err());
    }

    #[test]
    fn plain_text_is_not_a_sealed_reply() {
        assert!(unseal("{\"code\":200}", &[0u8; 32]).is_err());
    }
}
