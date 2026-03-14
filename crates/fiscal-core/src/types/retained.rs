use crate::newtypes::{Cents, Rate4};
use serde::{Deserialize, Serialize};

/// Retained federal taxes (`<retTrib>`) withheld at source.
///
/// All fields are optional; include only those applicable to the transaction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct RetTribData {
    /// Retained PIS value (`vRetPIS`).
    pub v_ret_pis: Option<Cents>,
    /// Retained COFINS value (`vRetCOFINS`).
    pub v_ret_cofins: Option<Cents>,
    /// Retained CSLL value (`vRetCSLL`).
    pub v_ret_csll: Option<Cents>,
    /// IRRF calculation base (`vBCIRRF`).
    pub v_bc_irrf: Option<Cents>,
    /// Retained IRRF value (`vIRRF`).
    pub v_irrf: Option<Cents>,
    /// Social security (INSS) calculation base (`vBCRetPrev`).
    pub v_bc_ret_prev: Option<Cents>,
    /// Retained social security contribution (`vRetPrev`).
    pub v_ret_prev: Option<Cents>,
}

impl RetTribData {
    /// Create a new empty `RetTribData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the retained PIS value.
    pub fn v_ret_pis(mut self, v: Cents) -> Self {
        self.v_ret_pis = Some(v);
        self
    }
    /// Set the retained COFINS value.
    pub fn v_ret_cofins(mut self, v: Cents) -> Self {
        self.v_ret_cofins = Some(v);
        self
    }
    /// Set the retained CSLL value.
    pub fn v_ret_csll(mut self, v: Cents) -> Self {
        self.v_ret_csll = Some(v);
        self
    }
    /// Set the IRRF base calculation value.
    pub fn v_bc_irrf(mut self, v: Cents) -> Self {
        self.v_bc_irrf = Some(v);
        self
    }
    /// Set the IRRF value.
    pub fn v_irrf(mut self, v: Cents) -> Self {
        self.v_irrf = Some(v);
        self
    }
    /// Set the social security base calculation value.
    pub fn v_bc_ret_prev(mut self, v: Cents) -> Self {
        self.v_bc_ret_prev = Some(v);
        self
    }
    /// Set the retained social security value.
    pub fn v_ret_prev(mut self, v: Cents) -> Self {
        self.v_ret_prev = Some(v);
        self
    }
}

/// Crédito presumido ICMS data (`<gCred>`) — up to 4 per item inside `<prod>`.
///
/// Maps to the PHP `taggCred()` method in sped-nfe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct GCredData {
    /// Código de Benefício Fiscal de Crédito Presumido (`cCredPresumido`).
    pub c_cred_presumido: String,
    /// Percentual do Crédito Presumido (`pCredPresumido`) — 4 decimal places.
    pub p_cred_presumido: Rate4,
    /// Valor do Crédito Presumido (`vCredPresumido`) — 2 decimal places. Optional.
    pub v_cred_presumido: Option<Cents>,
}

impl GCredData {
    /// Create a new `GCredData` with required fields.
    pub fn new(c_cred_presumido: impl Into<String>, p_cred_presumido: Rate4) -> Self {
        Self {
            c_cred_presumido: c_cred_presumido.into(),
            p_cred_presumido,
            v_cred_presumido: None,
        }
    }

    /// Set the crédito presumido value (`vCredPresumido`).
    pub fn v_cred_presumido(mut self, v: Cents) -> Self {
        self.v_cred_presumido = Some(v);
        self
    }
}
