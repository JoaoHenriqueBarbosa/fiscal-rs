use crate::newtypes::{Cents, IbgeCode};

/// Payment method and amount for a single payment entry (`<detPag>`).
///
/// Use the payment type codes from [`crate::constants::payment_types`] for
/// the `method` field (e.g. `"01"` for cash, `"17"` for Pix).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PaymentData {
    /// Payment type code (`tPag`), e.g. `"01"` (cash) or `"03"` (credit card).
    pub method: String,
    /// Amount paid in this payment entry.
    pub amount: Cents,
    /// Payment indicator (`indPag`). Optional — "0" à vista, "1" a prazo.
    pub ind_pag: Option<String>,
    /// Payment description (`xPag`). Optional.
    pub x_pag: Option<String>,
    /// Payment date (`dPag`). Optional — format YYYY-MM-DD.
    pub d_pag: Option<String>,
    /// CNPJ of the payer (`CNPJPag`). Optional — NT 2023.004.
    pub cnpj_pag: Option<String>,
    /// UF of the payer (`UFPag`). Optional — NT 2023.004.
    pub uf_pag: Option<String>,
}

impl PaymentData {
    /// Create a new `PaymentData`.
    pub fn new(method: impl Into<String>, amount: Cents) -> Self {
        Self {
            method: method.into(),
            amount,
            ind_pag: None,
            x_pag: None,
            d_pag: None,
            cnpj_pag: None,
            uf_pag: None,
        }
    }

    /// Set the payment indicator.
    pub fn ind_pag(mut self, v: impl Into<String>) -> Self {
        self.ind_pag = Some(v.into());
        self
    }

    /// Set the payment description.
    pub fn x_pag(mut self, v: impl Into<String>) -> Self {
        self.x_pag = Some(v.into());
        self
    }

    /// Set the payment date.
    pub fn d_pag(mut self, v: impl Into<String>) -> Self {
        self.d_pag = Some(v.into());
        self
    }

    /// Set the payer CNPJ.
    pub fn cnpj_pag(mut self, v: impl Into<String>) -> Self {
        self.cnpj_pag = Some(v.into());
        self
    }

    /// Set the payer UF.
    pub fn uf_pag(mut self, v: impl Into<String>) -> Self {
        self.uf_pag = Some(v.into());
        self
    }
}

/// Optional credit/debit card details attached to a payment entry (`<card>`).
///
/// All fields are optional; set only the ones available from the payment terminal.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PaymentCardDetail {
    /// Integration type code (`tpIntegra`): `"1"` (integrated) or `"2"` (non-integrated).
    pub integ_type: Option<String>,
    /// CNPJ of the card acquirer (`CNPJ`).
    pub card_tax_id: Option<String>,
    /// Card brand code (`tBand`), e.g. `"01"` (Visa), `"02"` (Mastercard).
    pub card_brand: Option<String>,
    /// Authorization code from the acquirer (`cAut`).
    pub auth_code: Option<String>,
    /// CNPJ of the payment beneficiary (`CNPJReceb`). Optional — NT 2023.004.
    pub cnpj_receb: Option<String>,
    /// Payment terminal identifier (`idTermPag`). Optional — NT 2023.004.
    pub id_term_pag: Option<String>,
}

impl PaymentCardDetail {
    /// Create a new `PaymentCardDetail` with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the integration type.
    pub fn integ_type(mut self, v: impl Into<String>) -> Self {
        self.integ_type = Some(v.into());
        self
    }

    /// Set the card tax ID (CNPJ).
    pub fn card_tax_id(mut self, v: impl Into<String>) -> Self {
        self.card_tax_id = Some(v.into());
        self
    }

    /// Set the card brand.
    pub fn card_brand(mut self, v: impl Into<String>) -> Self {
        self.card_brand = Some(v.into());
        self
    }

    /// Set the authorization code.
    pub fn auth_code(mut self, v: impl Into<String>) -> Self {
        self.auth_code = Some(v.into());
        self
    }

    /// Set the CNPJ of the payment beneficiary (NT 2023.004).
    pub fn cnpj_receb(mut self, v: impl Into<String>) -> Self {
        self.cnpj_receb = Some(v.into());
        self
    }

    /// Set the payment terminal identifier (NT 2023.004).
    pub fn id_term_pag(mut self, v: impl Into<String>) -> Self {
        self.id_term_pag = Some(v.into());
        self
    }
}

/// Referenced fiscal document types that may appear in the `<NFref>` section.
///
/// Each variant represents a different class of referenced document.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ReferenceDoc {
    /// Reference to another NF-e by its 44-digit access key.
    Nfe {
        /// 44-digit access key of the referenced NF-e.
        access_key: String,
    },
    /// Reference to another NF-e by its signed access key (PL_010).
    ///
    /// Emits `<refNFeSig>` instead of `<refNFe>`. Mutually exclusive with `Nfe`.
    NfeSig {
        /// Signed access key of the referenced NF-e.
        access_key: String,
    },
    /// Reference to a paper NF (model 1 or 1A).
    Nf {
        /// IBGE numeric state code (e.g. `"41"` for PR).
        state_code: IbgeCode,
        /// Year and month in `YYMM` format.
        year_month: String,
        /// CNPJ of the issuer.
        tax_id: String,
        /// Document model (e.g. `"01"`).
        model: String,
        /// Series number.
        series: String,
        /// Document number.
        number: String,
    },
    /// Reference to a paper NF from a rural producer (NFP).
    Nfp {
        /// IBGE numeric state code (e.g. `"41"` for PR).
        state_code: IbgeCode,
        /// Year and month in `YYMM` format.
        year_month: String,
        /// CPF or CNPJ of the issuer.
        tax_id: String,
        /// Inscrição Estadual do produtor rural (ou `"ISENTO"`).
        ie: String,
        /// Document model.
        model: String,
        /// Series number.
        series: String,
        /// Document number.
        number: String,
    },
    /// Reference to a CT-e by its 44-digit access key.
    Cte {
        /// 44-digit access key of the referenced CT-e.
        access_key: String,
    },
    /// Reference to an ECF fiscal receipt.
    Ecf {
        /// ECF model code.
        model: String,
        /// ECF sequential number.
        ecf_number: String,
        /// COO (Contador de Ordem de Operação) number.
        coo_number: String,
    },
}
