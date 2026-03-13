use super::TaxRegime;
use crate::newtypes::IbgeCode;

/// Issuer (emitente) identification and address data.
///
/// Required for every NF-e / NFC-e document. Built via [`IssuerData::new`];
/// optional fields (`trade_name`, `address_complement`) are set with chainable
/// methods.
///
/// # Examples
///
/// ```
/// use fiscal_core::types::{IssuerData, TaxRegime};
/// use fiscal_core::newtypes::IbgeCode;
///
/// let issuer = IssuerData::new(
///     "12345678000199",   // CNPJ
///     "123456789",        // state tax ID
///     "Minha Empresa Ltda",
///     TaxRegime::SimplesNacional,
///     "PR",
///     IbgeCode("4106852".into()),
///     "Curitiba",
///     "Rua das Flores",
///     "100",
///     "Centro",
///     "80010-010",
/// );
/// assert_eq!(issuer.state_code, "PR");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IssuerData {
    /// CNPJ or CPF of the issuer (digits only).
    pub tax_id: String,
    /// State tax registration (Inscrição Estadual).
    pub state_tax_id: String,
    /// Legal company name (`xNome`).
    pub company_name: String,
    /// Trading / fantasy name (`xFant`). Optional.
    pub trade_name: Option<String>,
    /// Tax regime code (`CRT`).
    pub tax_regime: TaxRegime,
    /// Two-letter state abbreviation (UF), e.g. `"PR"`.
    pub state_code: String,
    /// IBGE city code, e.g. `"4106852"` for Curitiba.
    pub city_code: IbgeCode,
    /// City name (`xMun`).
    pub city_name: String,
    /// Street name (`xLgr`).
    pub street: String,
    /// Street / building number (`nro`).
    pub street_number: String,
    /// Neighbourhood / district (`xBairro`).
    pub district: String,
    /// Brazilian postal code — 8 digits, no hyphen (`CEP`).
    pub zip_code: String,
    /// Address complement such as suite or floor (`xCpl`). Optional.
    pub address_complement: Option<String>,
    /// Phone number (`fone`). Optional.
    pub phone: Option<String>,
    /// Substitute ST state tax registration (`IEST`). Optional.
    pub iest: Option<String>,
    /// Municipal registration (`IM`). Optional — required for service providers.
    pub im: Option<String>,
    /// CNAE fiscal code (`CNAE`). Optional — required when `im` is present.
    pub cnae: Option<String>,
}

impl IssuerData {
    /// Create a new `IssuerData` with all required fields.
    /// Optional fields default to `None`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tax_id: impl Into<String>,
        state_tax_id: impl Into<String>,
        company_name: impl Into<String>,
        tax_regime: TaxRegime,
        state_code: impl Into<String>,
        city_code: IbgeCode,
        city_name: impl Into<String>,
        street: impl Into<String>,
        street_number: impl Into<String>,
        district: impl Into<String>,
        zip_code: impl Into<String>,
    ) -> Self {
        Self {
            tax_id: tax_id.into(),
            state_tax_id: state_tax_id.into(),
            company_name: company_name.into(),
            trade_name: None,
            tax_regime,
            state_code: state_code.into(),
            city_code,
            city_name: city_name.into(),
            street: street.into(),
            street_number: street_number.into(),
            district: district.into(),
            zip_code: zip_code.into(),
            address_complement: None,
            phone: None,
            iest: None,
            im: None,
            cnae: None,
        }
    }

    /// Set the trade name.
    pub fn trade_name(mut self, name: impl Into<String>) -> Self {
        self.trade_name = Some(name.into());
        self
    }

    /// Set the address complement.
    pub fn address_complement(mut self, complement: impl Into<String>) -> Self {
        self.address_complement = Some(complement.into());
        self
    }

    /// Set the phone number.
    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// Set the substitute ST state tax registration (IEST).
    pub fn iest(mut self, iest: impl Into<String>) -> Self {
        self.iest = Some(iest.into());
        self
    }

    /// Set the municipal registration (IM).
    pub fn im(mut self, im: impl Into<String>) -> Self {
        self.im = Some(im.into());
        self
    }

    /// Set the CNAE fiscal code.
    pub fn cnae(mut self, cnae: impl Into<String>) -> Self {
        self.cnae = Some(cnae.into());
        self
    }
}
