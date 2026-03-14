use crate::newtypes::IbgeCode;
use serde::{Deserialize, Serialize};

/// Address data for pickup (`retirada`) or delivery (`entrega`) locations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct LocationData {
    /// CNPJ or CPF of the location owner.
    pub tax_id: String,
    /// Name of the location (`xNome`). Optional.
    pub name: Option<String>,
    /// Street name (`xLgr`).
    pub street: String,
    /// Street / building number (`nro`).
    pub number: String,
    /// Address complement (`xCpl`). Optional.
    pub complement: Option<String>,
    /// Neighbourhood / district (`xBairro`).
    pub district: String,
    /// IBGE city code, e.g. `"4106852"` for Curitiba.
    pub city_code: IbgeCode,
    /// City name (`xMun`).
    pub city_name: String,
    /// Two-letter state abbreviation (UF), e.g. `"PR"`.
    pub state_code: String,
    /// Postal code (`CEP`). Optional.
    pub zip_code: Option<String>,
    /// Código do país (`cPais`). Optional.
    pub c_pais: Option<String>,
    /// Nome do país (`xPais`). Optional.
    pub x_pais: Option<String>,
    /// Telefone (`fone`). Optional.
    pub fone: Option<String>,
    /// E-mail (`email`). Optional.
    pub email: Option<String>,
    /// Inscrição Estadual (`IE`). Optional.
    pub ie: Option<String>,
}

impl LocationData {
    /// Create a new `LocationData` with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tax_id: impl Into<String>,
        street: impl Into<String>,
        number: impl Into<String>,
        district: impl Into<String>,
        city_code: IbgeCode,
        city_name: impl Into<String>,
        state_code: impl Into<String>,
    ) -> Self {
        Self {
            tax_id: tax_id.into(),
            name: None,
            street: street.into(),
            number: number.into(),
            complement: None,
            district: district.into(),
            city_code,
            city_name: city_name.into(),
            state_code: state_code.into(),
            zip_code: None,
            c_pais: None,
            x_pais: None,
            fone: None,
            email: None,
            ie: None,
        }
    }

    /// Set the name.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.name = Some(v.into());
        self
    }
    /// Set the complement.
    pub fn complement(mut self, v: impl Into<String>) -> Self {
        self.complement = Some(v.into());
        self
    }
    /// Set the zip code.
    pub fn zip_code(mut self, v: impl Into<String>) -> Self {
        self.zip_code = Some(v.into());
        self
    }
    /// Set o código do país (`cPais`).
    pub fn c_pais(mut self, v: impl Into<String>) -> Self {
        self.c_pais = Some(v.into());
        self
    }
    /// Set o nome do país (`xPais`).
    pub fn x_pais(mut self, v: impl Into<String>) -> Self {
        self.x_pais = Some(v.into());
        self
    }
    /// Set o telefone (`fone`).
    pub fn fone(mut self, v: impl Into<String>) -> Self {
        self.fone = Some(v.into());
        self
    }
    /// Set o e-mail (`email`).
    pub fn email(mut self, v: impl Into<String>) -> Self {
        self.email = Some(v.into());
        self
    }
    /// Set a inscrição estadual (`IE`).
    pub fn ie(mut self, v: impl Into<String>) -> Self {
        self.ie = Some(v.into());
        self
    }
}

/// Additional information section (`<infAdic>`) for freeform notes and observations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct AdditionalInfo {
    /// Free-text note for the taxpayer (`infCpl`), printed on the DANFE.
    pub taxpayer_note: Option<String>,
    /// Note for the tax authority (`infAdFisco`), not printed on the DANFE.
    pub tax_authority_note: Option<String>,
    /// Contributor observations (`obsCont`).
    pub contributor_obs: Option<Vec<FieldText>>,
    /// Fiscal observations (`obsFisco`).
    pub fiscal_obs: Option<Vec<FieldText>>,
    /// References to administrative or judicial processes (`procRef`).
    pub process_refs: Option<Vec<ProcessRef>>,
}

impl AdditionalInfo {
    /// Create a new empty `AdditionalInfo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the taxpayer note.
    pub fn taxpayer_note(mut self, v: impl Into<String>) -> Self {
        self.taxpayer_note = Some(v.into());
        self
    }
    /// Set the tax authority note.
    pub fn tax_authority_note(mut self, v: impl Into<String>) -> Self {
        self.tax_authority_note = Some(v.into());
        self
    }
    /// Set the contributor observations.
    pub fn contributor_obs(mut self, v: Vec<FieldText>) -> Self {
        self.contributor_obs = Some(v);
        self
    }
    /// Set the fiscal observations.
    pub fn fiscal_obs(mut self, v: Vec<FieldText>) -> Self {
        self.fiscal_obs = Some(v);
        self
    }
    /// Set the process references.
    pub fn process_refs(mut self, v: Vec<ProcessRef>) -> Self {
        self.process_refs = Some(v);
        self
    }
}

/// A field-name / text-value pair used in contributor and fiscal observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct FieldText {
    /// Field identifier (`xCampo`), max 20 characters.
    pub field: String,
    /// Text value (`xTexto`), max 60 characters.
    pub text: String,
}

impl FieldText {
    /// Create a new `FieldText`.
    pub fn new(field: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            text: text.into(),
        }
    }
}

/// Reference to an administrative or judicial process (`<procRef>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct ProcessRef {
    /// Process number (`nProc`).
    pub number: String,
    /// Process origin code (`indProc`): `"0"` (SEFAZ) through `"9"` (others).
    pub origin: String,
    /// Type of act (`tpAto`). Optional.
    pub tp_ato: Option<String>,
}

impl ProcessRef {
    /// Create a new `ProcessRef`.
    pub fn new(number: impl Into<String>, origin: impl Into<String>) -> Self {
        Self {
            number: number.into(),
            origin: origin.into(),
            tp_ato: None,
        }
    }

    /// Create a new `ProcessRef` with a type of act (`tpAto`).
    pub fn with_tp_ato(
        number: impl Into<String>,
        origin: impl Into<String>,
        tp_ato: impl Into<String>,
    ) -> Self {
        Self {
            number: number.into(),
            origin: origin.into(),
            tp_ato: Some(tp_ato.into()),
        }
    }
}

/// Intermediary entity data (`<infIntermed>`) for marketplace transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct IntermediaryData {
    /// CNPJ of the intermediary platform.
    pub tax_id: String,
    /// Platform's internal identifier for the transaction (`idCadIntTran`). Optional.
    pub id_cad_int_tran: Option<String>,
}

impl IntermediaryData {
    /// Create a new `IntermediaryData`.
    pub fn new(tax_id: impl Into<String>) -> Self {
        Self {
            tax_id: tax_id.into(),
            id_cad_int_tran: None,
        }
    }

    /// Set the intermediary transaction registration ID.
    pub fn id_cad_int_tran(mut self, v: impl Into<String>) -> Self {
        self.id_cad_int_tran = Some(v.into());
        self
    }
}

/// Technical responsible entity (`<infRespTec>`) — the company that developed the
/// software used to issue the NF-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct TechResponsibleData {
    /// CNPJ of the responsible software company (`CNPJ`).
    pub tax_id: String,
    /// Name of the technical contact person (`xContato`).
    pub contact: String,
    /// Contact email address (`email`).
    pub email: String,
    /// Contact phone number (`fone`). Optional.
    pub phone: Option<String>,
    /// CSRT token provided by the fiscal authority. Optional.
    /// When present along with `csrt_id`, generates `<idCSRT>` and `<hashCSRT>` tags.
    pub csrt: Option<String>,
    /// CSRT identifier (typically `"01"`). Optional.
    pub csrt_id: Option<String>,
}

impl TechResponsibleData {
    /// Create a new `TechResponsibleData` with required fields.
    pub fn new(
        tax_id: impl Into<String>,
        contact: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            tax_id: tax_id.into(),
            contact: contact.into(),
            email: email.into(),
            phone: None,
            csrt: None,
            csrt_id: None,
        }
    }

    /// Set the phone number.
    pub fn phone(mut self, v: impl Into<String>) -> Self {
        self.phone = Some(v.into());
        self
    }

    /// Set the CSRT token and identifier.
    ///
    /// The CSRT (Código de Segurança do Responsável Técnico) is a token
    /// provided by the fiscal authority. When set, `<idCSRT>` and `<hashCSRT>`
    /// tags are generated in the XML. The hash is `base64(sha1(CSRT + chNFe))`.
    pub fn csrt(mut self, token: impl Into<String>, id: impl Into<String>) -> Self {
        self.csrt = Some(token.into());
        self.csrt_id = Some(id.into());
        self
    }
}

/// Purchase references (`<compra>`) linking the NF-e to a purchase order or contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct PurchaseData {
    /// Purchase order number (`xPed`). Optional.
    pub order_number: Option<String>,
    /// Contract number (`xCont`). Optional.
    pub contract_number: Option<String>,
    /// Purchase note / tender number (`xNEmp`). Optional.
    pub purchase_note: Option<String>,
}

impl PurchaseData {
    /// Create a new empty `PurchaseData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the order number.
    pub fn order_number(mut self, v: impl Into<String>) -> Self {
        self.order_number = Some(v.into());
        self
    }
    /// Set the contract number.
    pub fn contract_number(mut self, v: impl Into<String>) -> Self {
        self.contract_number = Some(v.into());
        self
    }
    /// Set the purchase note.
    pub fn purchase_note(mut self, v: impl Into<String>) -> Self {
        self.purchase_note = Some(v.into());
        self
    }
}

/// Export information (`<exporta>`) for NF-e documents covering international exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct ExportData {
    /// UF of the exit point from Brazil (`UFSaidaPais`).
    pub exit_state: String,
    /// Name of the export location / port (`xLocExporta`).
    pub export_location: String,
    /// Name of the dispatch/customs location (`xLocDespacho`). Optional.
    pub dispatch_location: Option<String>,
}

impl ExportData {
    /// Create a new `ExportData` with required fields.
    pub fn new(exit_state: impl Into<String>, export_location: impl Into<String>) -> Self {
        Self {
            exit_state: exit_state.into(),
            export_location: export_location.into(),
            dispatch_location: None,
        }
    }

    /// Set the dispatch location.
    pub fn dispatch_location(mut self, v: impl Into<String>) -> Self {
        self.dispatch_location = Some(v.into());
        self
    }
}
