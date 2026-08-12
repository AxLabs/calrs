//! Guest phone numbers: normalisation to E.164 and the country calling codes
//! the booking form and the admin panel offer.
//!
//! Gateways all want E.164 (`+33612345678`), while guests type what they type:
//! `06 12 34 56 78`, `0033612345678`, `+33 6 12 34 56 78`. Normalisation
//! happens once, server-side, at booking time; everything downstream assumes
//! `bookings.guest_phone` is already E.164.
//!
//! This is deliberately not libphonenumber. It handles the trunk-prefix and
//! international-prefix cases that cover ordinary input, and leaves genuine
//! validity to the gateway, which is the only real authority on whether a
//! number can receive a message.

/// Used when nothing is configured yet, so the booking form can render before
/// an admin has been through the panel.
pub const DEFAULT_COUNTRY_CODE: &str = "+33";

/// Country calling codes offered in the admin panel and used to normalise
/// locally formatted guest numbers.
pub const COUNTRY_CODES: &[(&str, &str)] = &[
    ("+1", "United States / Canada (+1)"),
    ("+20", "Egypt (+20)"),
    ("+27", "South Africa (+27)"),
    ("+30", "Greece (+30)"),
    ("+31", "Netherlands (+31)"),
    ("+32", "Belgium (+32)"),
    ("+33", "France (+33)"),
    ("+34", "Spain (+34)"),
    ("+36", "Hungary (+36)"),
    ("+39", "Italy (+39)"),
    ("+40", "Romania (+40)"),
    ("+41", "Switzerland (+41)"),
    ("+43", "Austria (+43)"),
    ("+44", "United Kingdom (+44)"),
    ("+45", "Denmark (+45)"),
    ("+46", "Sweden (+46)"),
    ("+47", "Norway (+47)"),
    ("+48", "Poland (+48)"),
    ("+49", "Germany (+49)"),
    ("+52", "Mexico (+52)"),
    ("+53", "Cuba (+53)"),
    ("+54", "Argentina (+54)"),
    ("+55", "Brazil (+55)"),
    ("+56", "Chile (+56)"),
    ("+57", "Colombia (+57)"),
    ("+58", "Venezuela (+58)"),
    ("+60", "Malaysia (+60)"),
    ("+61", "Australia (+61)"),
    ("+62", "Indonesia (+62)"),
    ("+63", "Philippines (+63)"),
    ("+64", "New Zealand (+64)"),
    ("+65", "Singapore (+65)"),
    ("+66", "Thailand (+66)"),
    ("+7", "Russia / Kazakhstan (+7)"),
    ("+81", "Japan (+81)"),
    ("+82", "South Korea (+82)"),
    ("+84", "Vietnam (+84)"),
    ("+86", "China (+86)"),
    ("+90", "Turkey (+90)"),
    ("+91", "India (+91)"),
    ("+92", "Pakistan (+92)"),
    ("+93", "Afghanistan (+93)"),
    ("+94", "Sri Lanka (+94)"),
    ("+95", "Myanmar (+95)"),
    ("+98", "Iran (+98)"),
    ("+212", "Morocco (+212)"),
    ("+213", "Algeria (+213)"),
    ("+216", "Tunisia (+216)"),
    ("+218", "Libya (+218)"),
    ("+220", "Gambia (+220)"),
    ("+221", "Senegal (+221)"),
    ("+222", "Mauritania (+222)"),
    ("+223", "Mali (+223)"),
    ("+224", "Guinea (+224)"),
    ("+225", "Côte d'Ivoire (+225)"),
    ("+226", "Burkina Faso (+226)"),
    ("+227", "Niger (+227)"),
    ("+228", "Togo (+228)"),
    ("+229", "Benin (+229)"),
    ("+230", "Mauritius (+230)"),
    ("+231", "Liberia (+231)"),
    ("+232", "Sierra Leone (+232)"),
    ("+233", "Ghana (+233)"),
    ("+234", "Nigeria (+234)"),
    ("+235", "Chad (+235)"),
    ("+236", "Central African Republic (+236)"),
    ("+237", "Cameroon (+237)"),
    ("+238", "Cape Verde (+238)"),
    ("+239", "São Tomé and Príncipe (+239)"),
    ("+240", "Equatorial Guinea (+240)"),
    ("+241", "Gabon (+241)"),
    ("+242", "Republic of the Congo (+242)"),
    ("+243", "DR Congo (+243)"),
    ("+244", "Angola (+244)"),
    ("+245", "Guinea-Bissau (+245)"),
    ("+248", "Seychelles (+248)"),
    ("+249", "Sudan (+249)"),
    ("+250", "Rwanda (+250)"),
    ("+251", "Ethiopia (+251)"),
    ("+252", "Somalia (+252)"),
    ("+253", "Djibouti (+253)"),
    ("+254", "Kenya (+254)"),
    ("+255", "Tanzania (+255)"),
    ("+256", "Uganda (+256)"),
    ("+257", "Burundi (+257)"),
    ("+258", "Mozambique (+258)"),
    ("+260", "Zambia (+260)"),
    ("+261", "Madagascar (+261)"),
    ("+262", "Réunion / Mayotte (+262)"),
    ("+263", "Zimbabwe (+263)"),
    ("+264", "Namibia (+264)"),
    ("+265", "Malawi (+265)"),
    ("+266", "Lesotho (+266)"),
    ("+267", "Botswana (+267)"),
    ("+268", "Eswatini (+268)"),
    ("+269", "Comoros (+269)"),
    ("+290", "Saint Helena (+290)"),
    ("+291", "Eritrea (+291)"),
    ("+297", "Aruba (+297)"),
    ("+298", "Faroe Islands (+298)"),
    ("+299", "Greenland (+299)"),
    ("+350", "Gibraltar (+350)"),
    ("+351", "Portugal (+351)"),
    ("+352", "Luxembourg (+352)"),
    ("+353", "Ireland (+353)"),
    ("+354", "Iceland (+354)"),
    ("+355", "Albania (+355)"),
    ("+356", "Malta (+356)"),
    ("+357", "Cyprus (+357)"),
    ("+358", "Finland (+358)"),
    ("+359", "Bulgaria (+359)"),
    ("+370", "Lithuania (+370)"),
    ("+371", "Latvia (+371)"),
    ("+372", "Estonia (+372)"),
    ("+373", "Moldova (+373)"),
    ("+374", "Armenia (+374)"),
    ("+375", "Belarus (+375)"),
    ("+376", "Andorra (+376)"),
    ("+377", "Monaco (+377)"),
    ("+378", "San Marino (+378)"),
    ("+380", "Ukraine (+380)"),
    ("+381", "Serbia (+381)"),
    ("+382", "Montenegro (+382)"),
    ("+383", "Kosovo (+383)"),
    ("+385", "Croatia (+385)"),
    ("+386", "Slovenia (+386)"),
    ("+387", "Bosnia and Herzegovina (+387)"),
    ("+389", "North Macedonia (+389)"),
    ("+420", "Czech Republic (+420)"),
    ("+421", "Slovakia (+421)"),
    ("+423", "Liechtenstein (+423)"),
    ("+500", "Falkland Islands (+500)"),
    ("+501", "Belize (+501)"),
    ("+502", "Guatemala (+502)"),
    ("+503", "El Salvador (+503)"),
    ("+504", "Honduras (+504)"),
    ("+505", "Nicaragua (+505)"),
    ("+506", "Costa Rica (+506)"),
    ("+507", "Panama (+507)"),
    ("+508", "Saint Pierre and Miquelon (+508)"),
    ("+509", "Haiti (+509)"),
    ("+590", "Guadeloupe / Saint Martin (+590)"),
    ("+591", "Bolivia (+591)"),
    ("+592", "Guyana (+592)"),
    ("+593", "Ecuador (+593)"),
    ("+594", "French Guiana (+594)"),
    ("+595", "Paraguay (+595)"),
    ("+596", "Martinique (+596)"),
    ("+597", "Suriname (+597)"),
    ("+598", "Uruguay (+598)"),
    ("+599", "Caribbean Netherlands / Curaçao (+599)"),
    ("+670", "Timor-Leste (+670)"),
    ("+672", "Australian External Territories (+672)"),
    ("+673", "Brunei (+673)"),
    ("+674", "Nauru (+674)"),
    ("+675", "Papua New Guinea (+675)"),
    ("+676", "Tonga (+676)"),
    ("+677", "Solomon Islands (+677)"),
    ("+678", "Vanuatu (+678)"),
    ("+679", "Fiji (+679)"),
    ("+680", "Palau (+680)"),
    ("+681", "Wallis and Futuna (+681)"),
    ("+682", "Cook Islands (+682)"),
    ("+683", "Niue (+683)"),
    ("+685", "Samoa (+685)"),
    ("+686", "Kiribati (+686)"),
    ("+687", "New Caledonia (+687)"),
    ("+688", "Tuvalu (+688)"),
    ("+689", "French Polynesia (+689)"),
    ("+690", "Tokelau (+690)"),
    ("+691", "Micronesia (+691)"),
    ("+692", "Marshall Islands (+692)"),
    ("+850", "North Korea (+850)"),
    ("+852", "Hong Kong (+852)"),
    ("+853", "Macau (+853)"),
    ("+855", "Cambodia (+855)"),
    ("+856", "Laos (+856)"),
    ("+880", "Bangladesh (+880)"),
    ("+886", "Taiwan (+886)"),
    ("+960", "Maldives (+960)"),
    ("+961", "Lebanon (+961)"),
    ("+962", "Jordan (+962)"),
    ("+963", "Syria (+963)"),
    ("+964", "Iraq (+964)"),
    ("+965", "Kuwait (+965)"),
    ("+966", "Saudi Arabia (+966)"),
    ("+967", "Yemen (+967)"),
    ("+968", "Oman (+968)"),
    ("+970", "Palestine (+970)"),
    ("+971", "United Arab Emirates (+971)"),
    ("+972", "Israel (+972)"),
    ("+973", "Bahrain (+973)"),
    ("+974", "Qatar (+974)"),
    ("+975", "Bhutan (+975)"),
    ("+976", "Mongolia (+976)"),
    ("+977", "Nepal (+977)"),
    ("+992", "Tajikistan (+992)"),
    ("+993", "Turkmenistan (+993)"),
    ("+994", "Azerbaijan (+994)"),
    ("+995", "Georgia (+995)"),
    ("+996", "Kyrgyzstan (+996)"),
    ("+998", "Uzbekistan (+998)"),
];

pub fn is_valid_country_code(code: &str) -> bool {
    COUNTRY_CODES.iter().any(|(value, _)| *value == code.trim())
}

/// Normalise a guest-entered number to E.164, using `default_country_code` for
/// numbers written in national form. Returns `None` when the result could not
/// possibly be a phone number, in which case the booking is rejected with a
/// validation error rather than silently losing the number.
///
/// * `+33612345678`, `+33 6 12 34 56 78` stay as they are.
/// * `0033612345678` becomes `+33612345678` (international prefix).
/// * `0612345678` becomes `+33612345678` (one trunk `0` dropped).
/// * `612345678` becomes `+33612345678` (countries with no trunk prefix).
pub fn normalize(raw: &str, default_country_code: &str) -> Option<String> {
    let cleaned: String = raw
        .trim()
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '(' | ')' | '.' | '/' | '\u{a0}'))
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    let country = default_country_code.trim();
    let normalized = if let Some(rest) = cleaned.strip_prefix("00") {
        format!("+{}", rest)
    } else if cleaned.starts_with('+') {
        cleaned
    } else {
        if !is_valid_country_code(country) {
            return None;
        }
        // A single leading zero is the national trunk prefix and is dropped;
        // countries without one (the US, say) are unaffected.
        let national = cleaned.strip_prefix('0').unwrap_or(&cleaned);
        format!("{}{}", country, national)
    };

    is_e164(&normalized).then_some(normalized)
}

/// Loose E.164 check: a leading `+` then 8 to 15 digits.
///
/// E.164 caps the total at 15 digits; the lower bound only rejects input that
/// is obviously not a number. The gateway remains the authority on validity.
pub fn is_e164(raw: &str) -> bool {
    let raw = raw.trim();
    let Some(digits) = raw.strip_prefix('+') else {
        return false;
    };
    digits.len() >= 8 && digits.len() <= 15 && digits.chars().all(|c| c.is_ascii_digit())
}

/// E.164 without the leading `+`, the wire format GatewayAPI and seven.io
/// document for recipients.
pub fn without_plus(e164: &str) -> String {
    e164.trim().trim_start_matches('+').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plausible_e164() {
        assert!(is_e164("+15551234567"));
        assert!(is_e164(" +447911123456 "));
    }

    #[test]
    fn rejects_missing_plus_or_bad_length_or_non_digits() {
        assert!(!is_e164("07911123456"));
        assert!(!is_e164("+1234"));
        assert!(!is_e164("+1555abc4567"));
        assert!(!is_e164(""));
        // 16 digits is one past the E.164 ceiling.
        assert!(!is_e164("+1234567890123456"));
    }

    #[test]
    fn normalizes_national_numbers_with_the_default_country() {
        assert_eq!(
            normalize("06 12 34 56 78", "+33").as_deref(),
            Some("+33612345678")
        );
        // No trunk prefix: nothing is dropped.
        assert_eq!(
            normalize("5551234567", "+1").as_deref(),
            Some("+15551234567")
        );
    }

    #[test]
    fn preserves_international_forms() {
        assert_eq!(
            normalize("+33 6 12 34 56 78", "+1").as_deref(),
            Some("+33612345678")
        );
        assert_eq!(
            normalize("0033612345678", "+1").as_deref(),
            Some("+33612345678")
        );
    }

    #[test]
    fn strips_punctuation_guests_actually_type() {
        assert_eq!(
            normalize("(06) 12-34.56/78", "+33").as_deref(),
            Some("+33612345678")
        );
    }

    #[test]
    fn rejects_junk() {
        assert_eq!(normalize("", "+33"), None);
        assert_eq!(normalize("not a phone", "+33"), None);
        assert_eq!(normalize("0612345678", "+999"), None);
        // Only one trunk zero is dropped, so this stays too long to be E.164.
        assert_eq!(normalize("+00000000000000000", "+33"), None);
    }

    #[test]
    fn country_code_table_is_usable() {
        assert!(is_valid_country_code("+33"));
        assert!(is_valid_country_code(" +1 "));
        assert!(!is_valid_country_code("33"));
        assert!(is_valid_country_code(DEFAULT_COUNTRY_CODE));
    }

    #[test]
    fn without_plus_matches_gateway_wire_format() {
        assert_eq!(without_plus("+33612345678"), "33612345678");
    }
}
