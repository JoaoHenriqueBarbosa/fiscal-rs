use super::ContingencyType;
use crate::newtypes::IbgeCode;
use chrono::{DateTime, FixedOffset};

/// Recipient (destinatário) identification and optional address data.
///
/// For NFC-e issued to anonymous consumers under R$200 the recipient may be
/// omitted entirely. For other documents, at minimum `tax_id` and `name` are
/// required; the full address is optional.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct RecipientData {
    /// CNPJ, CPF, or foreign ID of the recipient (digits only).
    pub tax_id: String,
    /// Recipient legal or individual name (`xNome`).
    pub name: String,
    /// Two-letter state abbreviation (UF), e.g. `"PR"`.
    /// `None` when the recipient's state is unknown or absent.
    pub state_code: Option<String>,
    /// State tax registration (IE) of the recipient.
    pub state_tax_id: Option<String>,
    /// Street name (`xLgr`).
    pub street: Option<String>,
    /// Street / building number (`nro`).
    pub street_number: Option<String>,
    /// Neighbourhood / district (`xBairro`).
    pub district: Option<String>,
    /// IBGE city code, e.g. `"4106852"` for Curitiba.
    pub city_code: Option<IbgeCode>,
    /// City name (`xMun`).
    pub city_name: Option<String>,
    /// Brazilian postal code — 8 digits, no hyphen (`CEP`).
    pub zip_code: Option<String>,
    /// Address complement (`xCpl`).
    pub complement: Option<String>,
    /// Phone number (`fone`). Optional.
    pub phone: Option<String>,
    /// Email address (`email`). Optional.
    pub email: Option<String>,
    /// SUFRAMA registration (`ISUF`). Optional — for Zona Franca de Manaus.
    pub isuf: Option<String>,
    /// Municipal registration (`IM`). Optional.
    pub im: Option<String>,
    /// IE indicator (`indIEDest`). Optional override — "1" contribuinte, "2" isento, "9" não contribuinte.
    pub ind_ie_dest: Option<String>,
    /// Country code (`cPais`). Optional — defaults to "1058" (Brazil).
    pub country_code: Option<String>,
    /// Country name (`xPais`). Optional — defaults to "Brasil".
    pub country_name: Option<String>,
}

impl RecipientData {
    /// Create a new `RecipientData` with the two required fields.
    /// All optional fields default to `None`.
    pub fn new(tax_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            tax_id: tax_id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the state code (UF).
    pub fn state_code(mut self, code: impl Into<String>) -> Self {
        self.state_code = Some(code.into());
        self
    }

    /// Set the state tax ID (IE).
    pub fn state_tax_id(mut self, id: impl Into<String>) -> Self {
        self.state_tax_id = Some(id.into());
        self
    }

    /// Set the street.
    pub fn street(mut self, street: impl Into<String>) -> Self {
        self.street = Some(street.into());
        self
    }

    /// Set the street number.
    pub fn street_number(mut self, number: impl Into<String>) -> Self {
        self.street_number = Some(number.into());
        self
    }

    /// Set the district.
    pub fn district(mut self, district: impl Into<String>) -> Self {
        self.district = Some(district.into());
        self
    }

    /// Set the city code (IBGE).
    pub fn city_code(mut self, code: IbgeCode) -> Self {
        self.city_code = Some(code);
        self
    }

    /// Set the city name.
    pub fn city_name(mut self, name: impl Into<String>) -> Self {
        self.city_name = Some(name.into());
        self
    }

    /// Set the zip code.
    pub fn zip_code(mut self, zip: impl Into<String>) -> Self {
        self.zip_code = Some(zip.into());
        self
    }

    /// Set the address complement.
    pub fn complement(mut self, complement: impl Into<String>) -> Self {
        self.complement = Some(complement.into());
        self
    }

    /// Set the phone number.
    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// Set the email address.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the SUFRAMA registration (ISUF).
    pub fn isuf(mut self, isuf: impl Into<String>) -> Self {
        self.isuf = Some(isuf.into());
        self
    }

    /// Set the municipal registration (IM).
    pub fn im(mut self, im: impl Into<String>) -> Self {
        self.im = Some(im.into());
        self
    }

    /// Override the IE indicator (indIEDest).
    pub fn ind_ie_dest(mut self, ind: impl Into<String>) -> Self {
        self.ind_ie_dest = Some(ind.into());
        self
    }

    /// Set the country code (cPais) for foreign recipients.
    pub fn country_code(mut self, code: impl Into<String>) -> Self {
        self.country_code = Some(code.into());
        self
    }

    /// Set the country name (xPais) for foreign recipients.
    pub fn country_name(mut self, name: impl Into<String>) -> Self {
        self.country_name = Some(name.into());
        self
    }
}

/// Contingency activation data embedded in an NF-e when the primary SEFAZ
/// authorizer is unavailable.
///
/// When present, the XML builder inserts `<dhCont>` and `<xJust>` into `<ide>`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ContingencyData {
    /// Which contingency mode is active.
    pub contingency_type: ContingencyType,
    /// Human-readable justification for entering contingency (15–256 chars).
    pub reason: String,
    /// Timestamp when contingency mode was activated.
    pub at: DateTime<FixedOffset>,
}

impl ContingencyData {
    /// Create a new `ContingencyData` with all required fields.
    pub fn new(
        contingency_type: ContingencyType,
        reason: impl Into<String>,
        at: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            contingency_type,
            reason: reason.into(),
            at,
        }
    }
}
