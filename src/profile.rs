//! Payment profile request/response types for the Worldline NAM Profiles API.
//!
//! Public types mirror Worldline JSON field names. Use [`BillingAddress::validate_for_worldline`]
//! and [`BankAccount::validate_for_worldline`] before POST/PUT payloads.

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use std::{error, fmt};

fn deserialize_flex_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>
{
    struct FlexU64Visitor;
    impl<'de> Visitor<'de> for FlexU64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or numeric string")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            u64::try_from(value).map_err(|_| E::invalid_value(Unexpected::Signed(value), &self))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            value.trim().parse::<u64>().map_err(|_| E::invalid_value(Unexpected::Str(value), &self))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(FlexU64Visitor)
}

fn deserialize_optional_flex_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>
{
    struct OptFlexU64Visitor;
    impl<'de> Visitor<'de> for OptFlexU64Visitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, an integer, or a numeric string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(None)
        }

        fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
        where
            D2: Deserializer<'de>
        {
            deserialize_flex_u64(deserializer).map(Some)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(Some(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            deserialize_flex_u64(de::value::I64Deserializer::new(value)).map(Some)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            deserialize_flex_u64(de::value::StrDeserializer::new(value)).map(Some)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_option(OptFlexU64Visitor)
}

fn deserialize_card_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>
{
    struct CardIdVisitor;
    impl<'de> Visitor<'de> for CardIdVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, a string, or an integer card_id")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(None)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(Some(value.to_string()))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(Some(value.to_string()))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            let trimmed = value.trim();
            if trimmed.is_empty() { Ok(None) } else { Ok(Some(trimmed.to_owned())) }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(CardIdVisitor)
}

/// Outcome of a profile create/update API call.
pub type ProfileResult = Result<ProfileSuccessResult, ProfileErrorResult>;

/// Request body for creating or updating a Worldline payment profile.
#[derive(Deserialize, Debug, Serialize)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<TokenField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account: Option<BankAccount>,
    pub billing: BillingAddress,
    pub custom: CustomFields,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<bool>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct TokenField {
    pub name: String,
    pub code: String
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct BillingAddress {
    name: String,
    address_line1: String,
    address_line2: Option<String>,
    city: String,
    province: Province,
    country: Country,
    postal_code: String,
    phone_number: String,
    phone_country_code: String,
    phone_type: PhoneType,
    email_address: String
}

impl BillingAddress {
    pub fn validate_for_worldline(&self) -> Result<(), String> {
        let postal = self.postal_code.trim().to_uppercase().replace(' ', "");
        if postal.len() != 6
            || !postal
                .chars()
                .enumerate()
                .all(|(i, c)| if i % 2 == 0 { c.is_ascii_alphabetic() } else { c.is_ascii_digit() })
        {
            return Err("Postal code must be a valid Canadian format (e.g. V8T4M3).".to_string());
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Province {
    #[serde(alias = "bc", alias = "BC", alias = "British Columbia")]
    BC,
    #[serde(alias = "on", alias = "ON", alias = "Ontario")]
    ON
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Country {
    CA
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PhoneType {
    #[serde(rename = "m")]
    MOBILE,
    #[serde(rename = "h")]
    HOME,
    #[serde(rename = "w")]
    WORK
}

#[derive(Deserialize, Debug, Clone)]
pub struct BankAccount {
    #[serde(deserialize_with = "deserialize_flex_u64")]
    pub account_number: u64,
    pub bank_account_holder: String,
    pub bank_account_type: BankAccountType,
    #[serde(deserialize_with = "deserialize_flex_u64")]
    pub branch_number: u64,
    #[serde(deserialize_with = "deserialize_flex_u64")]
    pub institution_number: u64,
    #[serde(default, deserialize_with = "deserialize_optional_flex_u64")]
    pub routing_number: Option<u64>
}

impl Serialize for BankAccount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("BankAccount", 6)?;
        state.serialize_field("account_number", &self.account_number)?;
        state.serialize_field("bank_account_holder", &self.bank_account_holder)?;
        state.serialize_field("bank_account_type", &self.bank_account_type)?;
        state.serialize_field("branch_number", &format!("{:05}", self.branch_number))?;
        state.serialize_field("institution_number", &format!("{:03}", self.institution_number))?;
        state.serialize_field("routing_number", &self.routing_number)?;
        state.end()
    }
}

impl BankAccount {
    pub fn validate_for_worldline(&self) -> Result<(), String> {
        if self.bank_account_holder.trim().is_empty() {
            return Err("Bank account holder name is required.".to_string());
        }
        if self.institution_number == 0 || self.institution_number > 999 {
            return Err(
                "Institution number must be a 3-digit Canadian bank institution number (e.g. 001, 010, 127)."
                    .to_string(),
            );
        }
        if self.branch_number < 10_000 || self.branch_number > 99_999 {
            return Err(
                "Branch / transit number must be a 5-digit Canadian transit number.".to_string()
            );
        }
        if self.account_number == 0 {
            return Err("Bank account number is required.".to_string());
        }
        Ok(())
    }
}

/// Worldline Profiles API expects `"Canadian"` (not `"CA"`).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum BankAccountType {
    #[serde(rename = "Canadian", alias = "CA")]
    Canadian
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CustomFields {
    ref1: String,
    ref2: String,
    ref3: String,
    ref4: Option<String>,
    ref5: Option<String>
}

impl CustomFields {
    pub fn new(
        ref1: String,
        ref2: String,
        ref3: String,
        ref4: Option<String>,
        ref5: Option<String>
    ) -> Self {
        Self { ref1, ref2, ref3, ref4, ref5 }
    }
}
#[derive(thiserror::Error, Debug)]
pub enum ProfileErrorResult {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    WorldlineError(#[from] ProfileError)
}

impl ProfileErrorResult {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            ProfileErrorResult::UnexpectedError(v) => json!({"error": v.to_string()}),
            ProfileErrorResult::WorldlineError(v) => json!({"error": v})
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ProfileSuccessResult {
    pub code: u64,
    pub customer_code: String,
    pub message: String
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ProfileError {
    pub code: u64,
    pub category: u64,
    pub message: String
}

/// Lenient bank account parse for Worldline GET profile (masked account numbers, string transit fields).
#[derive(Debug, Clone)]
pub struct BankAccountRead {
    pub bank_account_holder: String,
    pub account_number: u64,
    pub account_number_raw: String,
    pub bank_account_type: BankAccountType,
    pub institution_number: u64,
    pub branch_number: u64,
    pub routing_number: Option<u64>
}

fn parse_account_number_field(raw: &serde_json::Value) -> (u64, String) {
    let text = match raw {
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::String(value) => value.trim().to_owned(),
        _ => String::new()
    };
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    let number = digits.parse::<u64>().unwrap_or(0);
    (number, text)
}

fn parse_bank_account_value(value: serde_json::Value) -> Result<BankAccountRead, String> {
    let holder = value
        .get("bank_account_holder")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "bank_account_holder is required".to_string())?;

    let (account_number, account_number_raw) =
        value.get("account_number").map(parse_account_number_field).unwrap_or((0, String::new()));

    let institution_number = value
        .get("institution_number")
        .map(|v| {
            if let Some(raw) = v.as_str() {
                raw.trim().parse::<u64>().unwrap_or(0)
            } else {
                v.as_u64().unwrap_or(0)
            }
        })
        .unwrap_or(0);

    let branch_number = value
        .get("branch_number")
        .map(|v| {
            if let Some(raw) = v.as_str() {
                raw.trim().parse::<u64>().unwrap_or(0)
            } else {
                v.as_u64().unwrap_or(0)
            }
        })
        .unwrap_or(0);

    let routing_number = value.get("routing_number").and_then(|v| {
        if v.is_null() {
            return None;
        }
        if let Some(raw) = v.as_str() { raw.trim().parse::<u64>().ok() } else { v.as_u64() }
    });

    let bank_account_type = value
        .get("bank_account_type")
        .and_then(|v| serde_json::from_value::<BankAccountType>(v.clone()).ok())
        .unwrap_or(BankAccountType::Canadian);

    Ok(BankAccountRead {
        bank_account_holder: holder.to_owned(),
        account_number,
        account_number_raw,
        bank_account_type,
        institution_number,
        branch_number,
        routing_number
    })
}

fn deserialize_optional_bank_account_read<'de, D>(
    deserializer: D
) -> Result<Option<BankAccountRead>, D::Error>
where
    D: Deserializer<'de>
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match value {
        None | Some(serde_json::Value::Null) => None,
        Some(raw) => parse_bank_account_value(raw).ok()
    })
}

/// Worldline GET /v1/profiles/{id} — fields used for Statbook profile details.
#[derive(Deserialize, Debug, Clone)]
pub struct ProfileDetails {
    #[serde(default)]
    pub customer_code: String,
    pub billing: Option<BillingAddress>,
    #[serde(default, deserialize_with = "deserialize_optional_bank_account_read")]
    pub bank_account: Option<BankAccountRead>,
    pub card: Option<ProfileCard>
}

pub fn redact_account_number(number: u64, raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains('*') || trimmed.contains('X') || trimmed.contains('x') {
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            let last = &digits[digits.len() - 4..];
            return format!("***{last}");
        }
        return "***".to_string();
    }

    if number == 0 {
        return "***".to_string();
    }

    let digits = number.to_string();
    if digits.len() <= 4 {
        return format!("***{digits}");
    }
    format!("***{}", &digits[digits.len() - 4..])
}

pub fn format_bank_label(
    holder: &str,
    institution_number: u64,
    branch_number: u64,
    account_display: &str
) -> String {
    format!(
        "EFT {account_display} ({}) — institution {:03} / transit {:05}",
        holder.trim(),
        institution_number,
        branch_number
    )
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProfileCard {
    #[serde(default, deserialize_with = "deserialize_card_id")]
    pub card_id: Option<String>,
    pub name: Option<String>,
    pub number: Option<String>,
    #[serde(alias = "card_type")]
    pub card_type: Option<String>,
    pub expiry_month: Option<String>,
    pub expiry_year: Option<String>
}

/// Worldline GET `/v1/profiles/{id}/cards` — `card` may be a single object or an array.
#[derive(Debug, Clone)]
pub struct ProfileCardsResponse {
    pub card: Option<ProfileCard>,
    pub cards: Option<Vec<ProfileCard>>
}

fn cards_same_identity(a: &ProfileCard, b: &ProfileCard) -> bool {
    match (a.card_id.as_deref(), b.card_id.as_deref()) {
        (Some(left), Some(right)) => left == right,
        _ => a.number == b.number && a.name == b.name
    }
}

fn push_parsed_card(collected: &mut Vec<ProfileCard>, raw: &serde_json::Value) {
    if let Ok(card) = serde_json::from_value::<ProfileCard>(raw.clone())
        && !collected.iter().any(|existing| cards_same_identity(existing, &card))
    {
        collected.push(card);
    }
}

fn parse_profile_cards_value(value: serde_json::Value) -> ProfileCardsResponse {
    let mut collected = Vec::new();

    if let Some(card_field) = value.get("card") {
        match card_field {
            serde_json::Value::Array(items) => {
                for item in items {
                    push_parsed_card(&mut collected, item);
                }
            }
            obj @ serde_json::Value::Object(_) => push_parsed_card(&mut collected, obj),
            _ => {}
        }
    }

    if let Some(serde_json::Value::Array(items)) = value.get("cards") {
        for item in items {
            push_parsed_card(&mut collected, item);
        }
    }

    let card = if collected.len() == 1 { collected.first().cloned() } else { None };

    ProfileCardsResponse { card, cards: if collected.is_empty() { None } else { Some(collected) } }
}

impl<'de> Deserialize<'de> for ProfileCardsResponse {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(parse_profile_cards_value(value))
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AddCardRequest {
    pub token: TokenField
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UpdateCardRequest {
    pub card: UpdateCardFields
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UpdateCardFields {
    pub expiry_month: String,
    pub expiry_year: String
}

/// Redact a card number for display: first 4 + *** + last 4 (or ***last4 when already masked).
pub fn redact_card_number(number: &str) -> String {
    let trimmed = number.trim();
    if trimmed.is_empty() {
        return "***".to_string();
    }

    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if trimmed.contains('*') {
        if digits.len() >= 4 {
            let last = &digits[digits.len() - 4..];
            return format!("***{last}");
        }
        return "***".to_string();
    }

    if digits.len() < 8 {
        return "***".to_string();
    }
    let first = &digits[..4];
    let last = &digits[digits.len() - 4..];
    format!("{first}***{last}")
}

pub(crate) fn card_type_label(card_type: &str) -> String {
    match card_type.trim().to_ascii_uppercase().as_str() {
        "VI" | "VISA" => "Visa".to_string(),
        "MC" | "MASTER" | "MASTERCARD" => "Mastercard".to_string(),
        "AM" | "AMEX" | "AX" => "Amex".to_string(),
        "DI" | "DISCOVER" | "DC" => "Discover".to_string(),
        "" => "Card".to_string(),
        other => other.to_string()
    }
}

pub fn format_card_label(card_type: Option<&str>, number: &str, name: &str) -> String {
    let brand = card_type.map(card_type_label).unwrap_or_else(|| "Card".to_string());
    let display = redact_card_number(number);
    if name.trim().is_empty() {
        format!("{brand} {display} card on file")
    } else {
        format!("{brand} {display} ({})", name.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account_number_is_masked(raw: &str) -> bool {
        let trimmed = raw.trim();
        trimmed.contains('*') || trimmed.contains('X') || trimmed.contains('x')
    }

    #[test]
    fn profile_update_json_omits_null_token_and_bank_account() {
        let billing: BillingAddress = serde_json::from_value(json!({
            "name": "Jane",
            "address_line1": "1 Main",
            "address_line2": null,
            "city": "Victoria",
            "province": "BC",
            "country": "CA",
            "postal_code": "V8T4M3",
            "phone_number": "2505551234",
            "phone_country_code": "1",
            "phone_type": "m",
            "email_address": "jane@example.com"
        }))
        .expect("billing json");

        let profile = Profile {
            token: None,
            bank_account: None,
            billing,
            custom: CustomFields {
                ref1: "party_type::client".to_string(),
                ref2: "1".to_string(),
                ref3: String::new(),
                ref4: None,
                ref5: None
            },
            validate: Some(true)
        };

        let json = serde_json::to_value(&profile).expect("profile json");
        let obj = json.as_object().expect("object");
        assert!(!obj.contains_key("token"));
        assert!(!obj.contains_key("bank_account"));
    }

    #[test]
    fn deserializes_profile_card_with_numeric_card_id() {
        let card: ProfileCard = serde_json::from_value(json!({
            "card_id": 42,
            "card_type": "VI",
            "name": "Jane Doe",
            "number": "403000******1234",
            "expiry_month": "12",
            "expiry_year": "28"
        }))
        .expect("card json");

        assert_eq!(card.card_id.as_deref(), Some("42"));
        assert_eq!(card.card_type.as_deref(), Some("VI"));
    }

    #[test]
    fn deserializes_bank_account_with_string_transit_fields() {
        let bank: BankAccount = serde_json::from_value(json!({
            "account_number": "1234567",
            "bank_account_holder": "Jane Doe",
            "bank_account_type": "Canadian",
            "branch_number": "12773",
            "institution_number": "010"
        }))
        .expect("bank json");

        assert_eq!(bank.institution_number, 10);
        assert_eq!(bank.branch_number, 12773);
        assert_eq!(bank.account_number, 1234567);
    }

    #[test]
    fn formats_masked_card_label() {
        let label = format_card_label(Some("VI"), "403000******1234", "Jane Doe");
        assert_eq!(label, "Visa ***1234 (Jane Doe)");
    }

    #[test]
    fn deserializes_profile_with_masked_bank_account() {
        let profile: ProfileDetails = serde_json::from_value(json!({
            "customer_code": "abc123",
            "bank_account": {
                "bank_account_holder": "Jane Doe",
                "account_number": "XXXXXX7890",
                "bank_account_type": "Canadian",
                "institution_number": "010",
                "branch_number": "12773"
            }
        }))
        .expect("profile json");

        let bank = profile.bank_account.expect("bank account");
        assert_eq!(bank.bank_account_holder, "Jane Doe");
        assert_eq!(bank.institution_number, 10);
        assert_eq!(bank.branch_number, 12773);
        assert_eq!(bank.account_number, 7890);
        assert!(account_number_is_masked(&bank.account_number_raw));
    }

    #[test]
    fn deserializes_worldline_cards_array_under_card_key() {
        let resp: ProfileCardsResponse = serde_json::from_value(json!({
            "code": 1,
            "message": "Operation Successful",
            "customer_code": "f34Abf626684474eA7EbA253a1193DBe",
            "card": [{
                "card_id": "1",
                "function": "DEF",
                "name": "vendor-test",
                "number": "510000XXXXXX1004",
                "expiry_month": "01",
                "expiry_year": "30",
                "card_type": "MC"
            }]
        }))
        .expect("cards response");

        let cards = resp.cards.expect("cards list");
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].card_id.as_deref(), Some("1"));
        assert_eq!(cards[0].card_type.as_deref(), Some("MC"));
    }

    fn sample_billing() -> BillingAddress {
        serde_json::from_value(json!({
            "name": "Jane",
            "address_line1": "1 Main",
            "address_line2": null,
            "city": "Victoria",
            "province": "BC",
            "country": "CA",
            "postal_code": "V8T4M3",
            "phone_number": "2505551234",
            "phone_country_code": "1",
            "phone_type": "m",
            "email_address": "jane@example.com"
        }))
        .expect("billing json")
    }

    fn sample_bank_account() -> BankAccount {
        serde_json::from_value(json!({
            "account_number": "1234567",
            "bank_account_holder": "Jane Doe",
            "bank_account_type": "Canadian",
            "branch_number": "12773",
            "institution_number": "010",
            "routing_number": null
        }))
        .expect("bank json")
    }

    #[test]
    fn bank_account_serializes_padded_transit_fields() {
        let bank = sample_bank_account();
        let value = serde_json::to_value(&bank).expect("bank json");
        assert_eq!(value["branch_number"], "12773");
        assert_eq!(value["institution_number"], "010");
    }

    #[test]
    fn profile_roundtrips_with_bank_account_and_token() {
        let profile = Profile {
            token: Some(TokenField { name: "Jane Doe".to_string(), code: "tok-abc".to_string() }),
            bank_account: Some(sample_bank_account()),
            billing: sample_billing(),
            custom: CustomFields::new(
                "party_type::client".to_string(),
                "1".to_string(),
                String::new(),
                None,
                None
            ),
            validate: Some(true)
        };

        let value = serde_json::to_value(&profile).expect("profile json");
        let reparsed: Profile = serde_json::from_value(value).expect("profile roundtrip");
        assert!(reparsed.token.is_some());
        assert!(reparsed.bank_account.is_some());
        assert_eq!(reparsed.validate, Some(true));
    }

    #[test]
    fn billing_address_validate_accepts_canadian_postal_code() {
        let billing = sample_billing();
        assert!(billing.validate_for_worldline().is_ok());
    }

    #[test]
    fn billing_address_validate_rejects_invalid_postal_code() {
        let billing: BillingAddress = serde_json::from_value(json!({
            "name": "Jane",
            "address_line1": "1 Main",
            "address_line2": null,
            "city": "Victoria",
            "province": "BC",
            "country": "CA",
            "postal_code": "12345",
            "phone_number": "2505551234",
            "phone_country_code": "1",
            "phone_type": "m",
            "email_address": "jane@example.com"
        }))
        .expect("billing json");
        assert!(billing.validate_for_worldline().is_err());
    }

    #[test]
    fn bank_account_validate_accepts_valid_canadian_account() {
        let bank = sample_bank_account();
        assert!(bank.validate_for_worldline().is_ok());
    }

    #[test]
    fn bank_account_validate_rejects_invalid_transit() {
        let mut bank = sample_bank_account();
        bank.branch_number = 12;
        assert!(bank.validate_for_worldline().is_err());
    }

    #[test]
    fn redact_and_format_bank_helpers() {
        assert_eq!(redact_account_number(1234567890, "1234567890"), "***7890");
        assert_eq!(redact_account_number(0, "XXXXXX7890"), "***7890");
        let label = format_bank_label("Jane Doe", 10, 12773, "***7890");
        assert!(label.contains("EFT ***7890"));
        assert!(label.contains("institution 010"));
    }

    #[test]
    fn redact_card_number_and_type_label() {
        assert_eq!(redact_card_number("4030001234567890"), "4030***7890");
        assert_eq!(card_type_label("VI"), "Visa");
        assert_eq!(card_type_label("MC"), "Mastercard");
        assert_eq!(
            format_card_label(Some("VI"), "4030001234567890", ""),
            "Visa 4030***7890 card on file"
        );
    }

    #[test]
    fn profile_error_result_to_json_variants() {
        let worldline = ProfileErrorResult::WorldlineError(ProfileError {
            code: 1,
            category: 2,
            message: "failed".to_string()
        });
        assert!(worldline.to_json().get("error").is_some());

        let unexpected = ProfileErrorResult::UnexpectedError(anyhow::anyhow!("boom"));
        assert_eq!(unexpected.to_json()["error"], "boom");
    }

    #[test]
    fn deserializes_profile_success_result() {
        let result: ProfileSuccessResult = serde_json::from_value(json!({
            "code": 1,
            "customer_code": "cust-1",
            "message": "ok"
        }))
        .expect("success result");
        assert_eq!(result.customer_code, "cust-1");
    }

    #[test]
    fn deserializes_profile_cards_single_object() {
        let resp: ProfileCardsResponse = serde_json::from_value(json!({
            "card": {
                "card_id": "9",
                "name": "Jane",
                "number": "510000XXXXXX1004",
                "card_type": "MC"
            }
        }))
        .expect("single card");
        assert!(resp.card.is_some());
        assert_eq!(resp.cards.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn add_and_update_card_requests_roundtrip() {
        let add = AddCardRequest {
            token: TokenField { name: "Jane".to_string(), code: "tok".to_string() }
        };
        let add_json = serde_json::to_value(&add).expect("add card");
        let _: AddCardRequest = serde_json::from_value(add_json).expect("add card roundtrip");

        let update = UpdateCardRequest {
            card: UpdateCardFields {
                expiry_month: "12".to_string(),
                expiry_year: "30".to_string()
            }
        };
        let update_json = serde_json::to_value(&update).expect("update card");
        let _: UpdateCardRequest =
            serde_json::from_value(update_json).expect("update card roundtrip");
    }
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "code: {}, category: {}, message: {}", self.code, self.category, self.message)
    }
}

impl error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
