-- SMS notifications (optional, opt-in per event type).
--
-- Provider-agnostic by design: one singleton config row names the provider
-- ('twilio', 'gatewayapi', 'sevenio', 'webhook') and carries the small set of
-- fields every SMS gateway needs. See src/sms/ for the SmsProvider trait and
-- the per-provider adapters.
--
-- Same lifecycle as smtp_config: a system-wide singleton, editable from the
-- admin panel or overridden wholesale by the CALRS_SMS_* environment block,
-- with the secret encrypted at rest via AES-256-GCM (see src/crypto.rs).

-- Guests can optionally leave a phone number on the booking form. It is only
-- collected and shown when the event type has SMS notifications enabled.
-- Stored normalized to E.164 (e.g. +33612345678).
ALTER TABLE bookings ADD COLUMN guest_phone TEXT;

-- Per-event-type opt-in switch (default off, so existing event types keep
-- working exactly as before with no SMS involved).
ALTER TABLE event_types ADD COLUMN sms_notifications_enabled INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS sms_config (
    id                   TEXT PRIMARY KEY,
    -- Provider kind: see sms::factory::kinds.
    provider             TEXT NOT NULL,
    -- Non-secret account identifier. Twilio stores its Account SID here;
    -- GatewayAPI and seven.io have no such field and leave it NULL.
    api_key              TEXT,
    -- The actual secret (Twilio auth token, GatewayAPI/seven.io API key,
    -- webhook HMAC secret), encrypted at rest.
    api_secret_enc       TEXT,
    -- From-number (Twilio) or alphanumeric sender ID (GatewayAPI, seven.io).
    sender               TEXT NOT NULL DEFAULT '',
    -- Region or self-hosted endpoint override, e.g. https://gatewayapi.eu.
    -- NULL uses the provider's documented default. For the 'webhook' provider
    -- this is the target URL and is required.
    base_url             TEXT,
    -- Used to normalize locally formatted guest numbers into E.164.
    default_country_code TEXT NOT NULL DEFAULT '+33',
    enabled              INTEGER NOT NULL DEFAULT 1,
    created_at           TEXT NOT NULL DEFAULT (datetime('now'))
);
