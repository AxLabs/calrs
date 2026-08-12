//! Twilio adapter.
//!
//! `POST /2010-04-01/Accounts/{AccountSid}/Messages.json`, form-encoded, HTTP
//! Basic with the Account SID as the username. Twilio wants recipients in full
//! E.164 with the leading `+`.
//!
//! Errors arrive as JSON with a numeric `code`; the ones worth distinguishing
//! are mapped onto [`SmsError`], the rest keep Twilio's own message.
//! Reference: <https://www.twilio.com/docs/api/errors>.

use async_trait::async_trait;

use super::{factory, SendReceipt, SmsConfig, SmsError, SmsProvider};

pub struct TwilioProvider {
    account_sid: String,
    auth_token: String,
    from: String,
    base_url: String,
}

impl TwilioProvider {
    pub fn new(config: &SmsConfig) -> Self {
        Self {
            account_sid: config.api_key.trim().to_string(),
            auth_token: config.api_secret.clone(),
            from: config.sender.trim().to_string(),
            base_url: factory::base_url(config),
        }
    }
}

#[derive(serde::Deserialize)]
struct MessageResponse {
    sid: Option<String>,
    num_segments: Option<String>,
    price: Option<String>,
    price_unit: Option<String>,
}

#[derive(serde::Deserialize)]
struct ErrorResponse {
    code: Option<i64>,
    message: Option<String>,
}

/// Map an HTTP status plus Twilio's JSON error body onto a normalised error.
///
/// Split out from the request so it can be tested against the payloads in
/// Twilio's documentation without a network call or a mock server.
fn parse_error(status: u16, body: &str) -> SmsError {
    let parsed: Option<ErrorResponse> = serde_json::from_str(body).ok();
    let code = parsed.as_ref().and_then(|e| e.code);
    let message = parsed
        .as_ref()
        .and_then(|e| e.message.clone())
        .unwrap_or_else(|| body.trim().chars().take(200).collect());

    match (status, code) {
        (_, Some(21211)) | (_, Some(21614)) | (_, Some(21610)) => {
            SmsError::InvalidRecipient(message)
        }
        (_, Some(21212)) | (_, Some(21606)) | (_, Some(21659)) | (_, Some(21660)) => {
            SmsError::InvalidSender(message)
        }
        (_, Some(20003)) | (401, _) | (403, _) => SmsError::Auth(message),
        (429, _) | (_, Some(20429)) => SmsError::RateLimited(message),
        (_, Some(21608)) => SmsError::Other(format!(
            "{} (trial accounts can only message verified numbers)",
            message
        )),
        (s, _) if (500..600).contains(&s) => SmsError::Transport(message),
        _ => SmsError::Other(message),
    }
}

#[async_trait]
impl SmsProvider for TwilioProvider {
    fn kind(&self) -> &'static str {
        factory::kinds::TWILIO
    }

    async fn send(&self, to: &str, body: &str) -> Result<SendReceipt, SmsError> {
        let url = format!(
            "{}/2010-04-01/Accounts/{}/Messages.json",
            self.base_url, self.account_sid
        );
        let params = [("To", to), ("From", self.from.as_str()), ("Body", body)];

        let response = super::http_client()
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| SmsError::Transport(e.to_string()))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(parse_error(status.as_u16(), &text));
        }

        let parsed: MessageResponse = serde_json::from_str(&text).unwrap_or(MessageResponse {
            sid: None,
            num_segments: None,
            price: None,
            price_unit: None,
        });

        Ok(SendReceipt {
            message_id: parsed.sid,
            segments: parsed.num_segments.and_then(|s| s.parse().ok()),
            // Twilio reports the price as a negative string ("-0.0750") and
            // often only after the message has left the queue.
            cost: parsed
                .price
                .and_then(|p| p.parse::<f64>().ok())
                .map(f64::abs),
            currency: parsed.price_unit,
        })
    }

    async fn check(&self) -> Result<(), SmsError> {
        let url = format!(
            "{}/2010-04-01/Accounts/{}.json",
            self.base_url, self.account_sid
        );
        let response = super::http_client()
            .get(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .send()
            .await
            .map_err(|e| SmsError::Transport(e.to_string()))?;

        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let text = response.text().await.unwrap_or_default();
        Err(parse_error(status.as_u16(), &text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_invalid_recipient() {
        let body = r#"{"code": 21211, "message": "The 'To' number is not a valid phone number.", "status": 400}"#;
        assert!(matches!(
            parse_error(400, body),
            SmsError::InvalidRecipient(_)
        ));
    }

    #[test]
    fn maps_bad_credentials() {
        let body = r#"{"code": 20003, "message": "Authenticate", "status": 401}"#;
        assert!(matches!(parse_error(401, body), SmsError::Auth(_)));
        // Even without a parseable body, a 401 is an auth failure.
        assert!(matches!(parse_error(401, "nope"), SmsError::Auth(_)));
    }

    #[test]
    fn maps_sender_and_rate_limit_and_server_errors() {
        let sender =
            r#"{"code": 21606, "message": "The From number is not a valid, SMS-capable number"}"#;
        assert!(matches!(
            parse_error(400, sender),
            SmsError::InvalidSender(_)
        ));
        assert!(matches!(parse_error(429, "{}"), SmsError::RateLimited(_)));
        assert!(matches!(parse_error(503, "{}"), SmsError::Transport(_)));
    }

    #[test]
    fn keeps_the_gateway_message_for_unmapped_codes() {
        let body = r#"{"code": 30001, "message": "Queue overflow"}"#;
        match parse_error(400, body) {
            SmsError::Other(m) => assert_eq!(m, "Queue overflow"),
            other => panic!("unexpected mapping: {:?}", other),
        }
    }
}
