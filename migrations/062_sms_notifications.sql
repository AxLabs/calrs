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
-- collected and shown when the event type asks for one.
-- Stored normalized to E.164 (e.g. +33612345678).
ALTER TABLE bookings ADD COLUMN guest_phone TEXT;

-- Per-event-type phone policy, the SMS opt-in:
--   'off'      no field, no SMS. The default, so existing event types are
--              untouched.
--   'optional' field shown, guest may skip it and simply gets no SMS.
--   'required' field shown and enforced, for event types where the text
--              message is the point (a phone call, an on-site visit).
ALTER TABLE event_types ADD COLUMN sms_phone_mode TEXT NOT NULL DEFAULT 'off';

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
    -- Instance-wide messages/day ceiling. 0 means no limit. The booking form
    -- is public and the recipient is guest-controlled, so a runaway event
    -- type (or an SMS pumping attempt) would otherwise bill the operator
    -- without bound. When the cap trips, email carries on alone.
    daily_cap            INTEGER NOT NULL DEFAULT 0,
    enabled              INTEGER NOT NULL DEFAULT 1,
    created_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

-- One row per accepted message: what the cap counts, and what the admin panel
-- reports spend from. Deliberately carries no recipient number: this is a
-- usage ledger, not a message log, and guest phone numbers already live on
-- the booking that needs them.
CREATE TABLE IF NOT EXISTS sms_usage (
    id         TEXT PRIMARY KEY,
    sent_at    TEXT NOT NULL DEFAULT (datetime('now')),
    -- SmsEvent: confirmed / cancelled / rescheduled / reminder / test.
    event      TEXT NOT NULL,
    provider   TEXT NOT NULL,
    -- Billed segments and cost as reported by the gateway, when it says.
    segments   INTEGER,
    cost       REAL,
    currency   TEXT
);

CREATE INDEX IF NOT EXISTS idx_sms_usage_sent_at ON sms_usage(sent_at);

-- Who may put an event type into an SMS-sending mode. SMS spends instance-wide
-- credit an admin paid for, so the default matches shared resources: admins
-- only, with an explicit opt-out for instances where everyone is trusted.
ALTER TABLE auth_config ADD COLUMN sms_allow_all_users INTEGER NOT NULL DEFAULT 0;
