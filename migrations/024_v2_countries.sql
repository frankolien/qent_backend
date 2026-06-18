-- V2 §9.1 — countries reference table.
--
-- Drives every per-jurisdiction rule in the app: currency display,
-- Sumsub flow id, on/off-ramp partners, plate-format hint, phone code,
-- default off-ramp destination type, and the supported/waitlist gate
-- from §6.3.
--
-- Seeded below with the §11.3 day-1 launch list plus a handful of
-- waitlist-only countries so the country picker has something to show
-- non-launch markets.

CREATE TABLE IF NOT EXISTS countries (
    iso2                              TEXT PRIMARY KEY,
    name                              TEXT NOT NULL,
    currency_code                     TEXT NOT NULL,
    phone_code                        TEXT NOT NULL,
    sumsub_level                      TEXT NOT NULL,
    onramp_partners                   TEXT[] NOT NULL DEFAULT '{}',
    offramp_partners                  TEXT[] NOT NULL DEFAULT '{}',
    supported                         BOOLEAN NOT NULL DEFAULT false,
    plate_format_hint                 TEXT,
    default_offramp_destination_type  TEXT NOT NULL DEFAULT 'bank_account'
);

CREATE INDEX IF NOT EXISTS countries_supported_idx ON countries(supported);

-- §11.3 day-1 launch markets.
INSERT INTO countries
    (iso2, name,           currency_code, phone_code, sumsub_level,    onramp_partners,                offramp_partners,               supported, plate_format_hint, default_offramp_destination_type)
VALUES
    ('NG', 'Nigeria',      'NGN',         '+234',     'qent-ng-v1',    '{yellow_card,moonpay}',        '{yellow_card,moonpay}',        true,      'ABC-123-XY',      'bank_account'),
    ('GH', 'Ghana',        'GHS',         '+233',     'qent-gh-v1',    '{moonpay}',                    '{moonpay}',                    true,      'GR-1234-20',      'bank_account'),
    ('KE', 'Kenya',        'KES',         '+254',     'qent-ke-v1',    '{moonpay}',                    '{moonpay}',                    true,      'KAA 123A',        'bank_account'),
    ('ZA', 'South Africa', 'ZAR',         '+27',      'qent-za-v1',    '{moonpay}',                    '{moonpay}',                    true,      'CA 123 456',      'bank_account'),
    ('AE', 'UAE',          'AED',         '+971',     'qent-ae-v1',    '{moonpay}',                    '{moonpay}',                    true,      'A 12345',         'iban')
ON CONFLICT (iso2) DO NOTHING;

-- Waitlist-only markets so the country picker shows them as
-- "coming soon" rather than absent.
INSERT INTO countries
    (iso2, name,           currency_code, phone_code, sumsub_level,    onramp_partners,                offramp_partners,               supported, plate_format_hint, default_offramp_destination_type)
VALUES
    ('DE', 'Germany',      'EUR',         '+49',      'qent-eu-v1',    '{moonpay}',                    '{moonpay}',                    false,     'M-AB 1234',       'iban'),
    ('FR', 'France',       'EUR',         '+33',      'qent-eu-v1',    '{moonpay}',                    '{moonpay}',                    false,     'AB-123-CD',       'iban'),
    ('ES', 'Spain',        'EUR',         '+34',      'qent-eu-v1',    '{moonpay}',                    '{moonpay}',                    false,     '1234 ABC',        'iban'),
    ('IT', 'Italy',        'EUR',         '+39',      'qent-eu-v1',    '{moonpay}',                    '{moonpay}',                    false,     'AB 123 CD',       'iban'),
    ('NL', 'Netherlands',  'EUR',         '+31',      'qent-eu-v1',    '{moonpay}',                    '{moonpay}',                    false,     'AB-12-CD',        'iban'),
    ('PT', 'Portugal',     'EUR',         '+351',     'qent-eu-v1',    '{moonpay}',                    '{moonpay}',                    false,     '12-AB-34',        'iban'),
    ('GB', 'United Kingdom','GBP',        '+44',      'qent-uk-v1',    '{moonpay}',                    '{moonpay}',                    false,     'AB12 CDE',        'bank_account'),
    ('US', 'United States','USD',         '+1',       'qent-us-v1',    '{moonpay}',                    '{moonpay}',                    false,     'ABC 1234',        'ach'),
    ('CA', 'Canada',       'CAD',         '+1',       'qent-ca-v1',    '{moonpay}',                    '{moonpay}',                    false,     'ABCD 123',        'ach'),
    ('BR', 'Brazil',       'BRL',         '+55',      'qent-br-v1',    '{moonpay}',                    '{moonpay}',                    false,     'ABC-1234',        'bank_account'),
    ('MX', 'Mexico',       'MXN',         '+52',      'qent-mx-v1',    '{moonpay}',                    '{moonpay}',                    false,     'ABC-12-34',       'bank_account')
ON CONFLICT (iso2) DO NOTHING;
