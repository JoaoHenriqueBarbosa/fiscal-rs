use serde::{Deserialize, Serialize};

use crate::newtypes::{Cents, Rate};

/// Batch/lot tracking data (`<rastro>`) for traceability of perishable or regulated goods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct RastroData {
    /// Batch / lot number (`nLote`).
    pub n_lote: String,
    /// Quantity in this batch (`qLote`).
    pub q_lote: f64,
    /// Manufacturing / production date (`dFab`) in `YYYY-MM-DD` format.
    pub d_fab: String,
    /// Expiry / validation date (`dVal`) in `YYYY-MM-DD` format.
    pub d_val: String,
    /// Aggregate code (`cAgreg`). Optional.
    pub c_agreg: Option<String>,
}

impl RastroData {
    /// Create a new `RastroData` with required fields.
    pub fn new(
        n_lote: impl Into<String>,
        q_lote: f64,
        d_fab: impl Into<String>,
        d_val: impl Into<String>,
    ) -> Self {
        Self {
            n_lote: n_lote.into(),
            q_lote,
            d_fab: d_fab.into(),
            d_val: d_val.into(),
            c_agreg: None,
        }
    }

    /// Set the aggregate code.
    pub fn c_agreg(mut self, v: impl Into<String>) -> Self {
        self.c_agreg = Some(v.into());
        self
    }
}

/// Vehicle product details (`<veicProd>`) for NF-e documents covering automotive sales.
///
/// All fields are required as mandated by DENATRAN / SEFAZ vehicle invoicing rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct VeicProdData {
    /// Type of operation (`tpOp`): `"1"` (sale to end consumer), `"2"` (sell to reseller), `"3"` (other).
    pub tp_op: String,
    /// Chassis number (`chassi`), 17 characters.
    pub chassi: String,
    /// DENATRAN colour code (`cCor`).
    pub c_cor: String,
    /// Colour description (`xCor`).
    pub x_cor: String,
    /// Engine power in CV (`pot`).
    pub pot: String,
    /// Engine displacement in cm³ (`cilin`).
    pub cilin: String,
    /// Net weight in kg (`pesoL`).
    pub peso_l: String,
    /// Gross weight in kg (`pesoB`).
    pub peso_b: String,
    /// Vehicle serial number (`nSerie`).
    pub n_serie: String,
    /// Fuel type code (`tpComb`).
    pub tp_comb: String,
    /// Engine number (`nMotor`).
    pub n_motor: String,
    /// Maximum towing capacity in kg (`CMT`).
    pub cmt: String,
    /// Wheelbase in mm (`dist`).
    pub dist: String,
    /// Model year (`anoMod`).
    pub ano_mod: String,
    /// Manufacturing year (`anoFab`).
    pub ano_fab: String,
    /// Paint type code (`tpPint`).
    pub tp_pint: String,
    /// Vehicle type code (`tpVeic`).
    pub tp_veic: String,
    /// Vehicle species code (`espVeic`).
    pub esp_veic: String,
    /// VIN condition (`VIN`): `"R"` (regular) or `"N"` (non-regular).
    pub vin: String,
    /// Vehicle condition (`condVeic`): `"1"` (new) or `"2"` (used).
    pub cond_veic: String,
    /// DENATRAN vehicle model code (`cMod`).
    pub c_mod: String,
    /// DENATRAN colour code (extended) (`cCorDENATRAN`).
    pub c_cor_denatran: String,
    /// Passenger capacity (`lota`).
    pub lota: String,
    /// Vehicle restriction code (`tpRest`).
    pub tp_rest: String,
}

impl VeicProdData {
    /// Create a new `VeicProdData` with all required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tp_op: impl Into<String>,
        chassi: impl Into<String>,
        c_cor: impl Into<String>,
        x_cor: impl Into<String>,
        pot: impl Into<String>,
        cilin: impl Into<String>,
        peso_l: impl Into<String>,
        peso_b: impl Into<String>,
        n_serie: impl Into<String>,
        tp_comb: impl Into<String>,
        n_motor: impl Into<String>,
        cmt: impl Into<String>,
        dist: impl Into<String>,
        ano_mod: impl Into<String>,
        ano_fab: impl Into<String>,
        tp_pint: impl Into<String>,
        tp_veic: impl Into<String>,
        esp_veic: impl Into<String>,
        vin: impl Into<String>,
        cond_veic: impl Into<String>,
        c_mod: impl Into<String>,
        c_cor_denatran: impl Into<String>,
        lota: impl Into<String>,
        tp_rest: impl Into<String>,
    ) -> Self {
        Self {
            tp_op: tp_op.into(),
            chassi: chassi.into(),
            c_cor: c_cor.into(),
            x_cor: x_cor.into(),
            pot: pot.into(),
            cilin: cilin.into(),
            peso_l: peso_l.into(),
            peso_b: peso_b.into(),
            n_serie: n_serie.into(),
            tp_comb: tp_comb.into(),
            n_motor: n_motor.into(),
            cmt: cmt.into(),
            dist: dist.into(),
            ano_mod: ano_mod.into(),
            ano_fab: ano_fab.into(),
            tp_pint: tp_pint.into(),
            tp_veic: tp_veic.into(),
            esp_veic: esp_veic.into(),
            vin: vin.into(),
            cond_veic: cond_veic.into(),
            c_mod: c_mod.into(),
            c_cor_denatran: c_cor_denatran.into(),
            lota: lota.into(),
            tp_rest: tp_rest.into(),
        }
    }
}

/// Medicine / pharmaceutical product details (`<med>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct MedData {
    /// ANVISA product registry code (`cProdANVISA`). Optional (use `"isento"` when exempt).
    pub c_prod_anvisa: Option<String>,
    /// Exemption reason when `cProdANVISA` is absent (`xMotivoIsencao`). Optional.
    pub x_motivo_isencao: Option<String>,
    /// Maximum consumer price (`vPMC`) in the applicable state.
    pub v_pmc: Cents,
}

impl MedData {
    /// Create a new `MedData` with the required PMC value.
    pub fn new(v_pmc: Cents) -> Self {
        Self {
            c_prod_anvisa: None,
            x_motivo_isencao: None,
            v_pmc,
        }
    }

    /// Set the ANVISA product code.
    pub fn c_prod_anvisa(mut self, v: impl Into<String>) -> Self {
        self.c_prod_anvisa = Some(v.into());
        self
    }
    /// Set the exemption reason.
    pub fn x_motivo_isencao(mut self, v: impl Into<String>) -> Self {
        self.x_motivo_isencao = Some(v.into());
        self
    }
}

/// Firearm / weapon product details (`<arma>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct ArmaData {
    /// Weapon type code (`tpArma`): `"0"` (allowed use) or `"1"` (restricted use).
    pub tp_arma: String,
    /// Weapon serial number (`nSerie`).
    pub n_serie: String,
    /// Barrel number (`nCano`).
    pub n_cano: String,
    /// Weapon description (`descr`).
    pub descr: String,
}

impl ArmaData {
    /// Create a new `ArmaData` with all required fields.
    pub fn new(
        tp_arma: impl Into<String>,
        n_serie: impl Into<String>,
        n_cano: impl Into<String>,
        descr: impl Into<String>,
    ) -> Self {
        Self {
            tp_arma: tp_arma.into(),
            n_serie: n_serie.into(),
            n_cano: n_cano.into(),
            descr: descr.into(),
        }
    }
}

/// Per-item observation fields (`<obsItem>`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct ObsItemData {
    /// Contributor observation (`obsCont`). Optional.
    pub obs_cont: Option<ObsField>,
    /// Fiscal observation (`obsFisco`). Optional.
    pub obs_fisco: Option<ObsField>,
}

impl ObsItemData {
    /// Create a new empty `ObsItemData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the contributor observation.
    pub fn obs_cont(mut self, v: ObsField) -> Self {
        self.obs_cont = Some(v);
        self
    }
    /// Set the fiscal observation.
    pub fn obs_fisco(mut self, v: ObsField) -> Self {
        self.obs_fisco = Some(v);
        self
    }
}

/// A single per-item observation field (`obsCont` or `obsFisco`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct ObsField {
    /// Field identifier (`xCampo`), max 20 characters.
    pub x_campo: String,
    /// Text content (`xTexto`), max 60 characters.
    pub x_texto: String,
}

impl ObsField {
    /// Create a new `ObsField`.
    pub fn new(x_campo: impl Into<String>, x_texto: impl Into<String>) -> Self {
        Self {
            x_campo: x_campo.into(),
            x_texto: x_texto.into(),
        }
    }
}

/// Import declaration data (`<DI>` inside `<prod>`).
///
/// Represents a Declaração de Importação (DI, DSI, DIRE) attached to an invoice
/// item. Each DI may contain one or more additions ([`AdiData`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct DiData {
    /// Document number (`nDI`) — DI, DSI, DIRE, etc.
    pub n_di: String,
    /// Document registration date (`dDI`) in `YYYY-MM-DD` format.
    pub d_di: String,
    /// Customs clearance location (`xLocDesemb`).
    pub x_loc_desemb: String,
    /// State (UF) where customs clearance occurred (`UFDesemb`).
    pub uf_desemb: String,
    /// Customs clearance date (`dDesemb`) in `YYYY-MM-DD` format.
    pub d_desemb: String,
    /// International transport route code (`tpViaTransp`).
    pub tp_via_transp: String,
    /// AFRMM value — Adicional ao Frete para Renovação da Marinha Mercante (`vAFRMM`). Optional.
    pub v_afrmm: Option<Cents>,
    /// Import intermediation type code (`tpIntermedio`).
    pub tp_intermedio: String,
    /// CNPJ of the acquirer or ordering party (`CNPJ`). Optional.
    pub cnpj: Option<String>,
    /// CPF of the acquirer or ordering party (`CPF`). Optional.
    pub cpf: Option<String>,
    /// State (UF) of the third-party acquirer (`UFTerceiro`). Optional.
    pub uf_terceiro: Option<String>,
    /// Exporter code (`cExportador`).
    pub c_exportador: String,
    /// List of additions (`adi`) within this DI.
    pub adi: Vec<AdiData>,
}

impl DiData {
    /// Create a new `DiData` with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        n_di: impl Into<String>,
        d_di: impl Into<String>,
        x_loc_desemb: impl Into<String>,
        uf_desemb: impl Into<String>,
        d_desemb: impl Into<String>,
        tp_via_transp: impl Into<String>,
        tp_intermedio: impl Into<String>,
        c_exportador: impl Into<String>,
        adi: Vec<AdiData>,
    ) -> Self {
        Self {
            n_di: n_di.into(),
            d_di: d_di.into(),
            x_loc_desemb: x_loc_desemb.into(),
            uf_desemb: uf_desemb.into(),
            d_desemb: d_desemb.into(),
            tp_via_transp: tp_via_transp.into(),
            v_afrmm: None,
            tp_intermedio: tp_intermedio.into(),
            cnpj: None,
            cpf: None,
            uf_terceiro: None,
            c_exportador: c_exportador.into(),
            adi,
        }
    }

    /// Set the AFRMM value.
    pub fn v_afrmm(mut self, v: Cents) -> Self {
        self.v_afrmm = Some(v);
        self
    }

    /// Set the CNPJ of the acquirer or ordering party.
    pub fn cnpj(mut self, v: impl Into<String>) -> Self {
        self.cnpj = Some(v.into());
        self
    }

    /// Set the CPF of the acquirer or ordering party.
    pub fn cpf(mut self, v: impl Into<String>) -> Self {
        self.cpf = Some(v.into());
        self
    }

    /// Set the UF of the third-party acquirer.
    pub fn uf_terceiro(mut self, v: impl Into<String>) -> Self {
        self.uf_terceiro = Some(v.into());
        self
    }
}

/// Addition data (`<adi>` inside `<DI>`) for import declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct AdiData {
    /// Addition number (`nAdicao`). Optional.
    pub n_adicao: Option<String>,
    /// Sequential number within the addition (`nSeqAdic`).
    pub n_seq_adic: String,
    /// Foreign manufacturer code (`cFabricante`).
    pub c_fabricante: String,
    /// Discount value for this DI addition (`vDescDI`). Optional.
    pub v_desc_di: Option<Cents>,
    /// Drawback concession act number (`nDraw`). Optional.
    pub n_draw: Option<String>,
}

impl AdiData {
    /// Create a new `AdiData` with required fields.
    pub fn new(n_seq_adic: impl Into<String>, c_fabricante: impl Into<String>) -> Self {
        Self {
            n_adicao: None,
            n_seq_adic: n_seq_adic.into(),
            c_fabricante: c_fabricante.into(),
            v_desc_di: None,
            n_draw: None,
        }
    }

    /// Set the addition number.
    pub fn n_adicao(mut self, v: impl Into<String>) -> Self {
        self.n_adicao = Some(v.into());
        self
    }

    /// Set the DI discount value.
    pub fn v_desc_di(mut self, v: Cents) -> Self {
        self.v_desc_di = Some(v);
        self
    }

    /// Set the Drawback act number.
    pub fn n_draw(mut self, v: impl Into<String>) -> Self {
        self.n_draw = Some(v.into());
        self
    }
}

/// Export detail data (`<detExport>` inside `<prod>`).
///
/// Contains export information for an invoice item, including the optional
/// indirect export (`<exportInd>`) sub-group.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct DetExportData {
    /// Drawback concession act number (`nDraw`). Optional.
    pub n_draw: Option<String>,
    /// Export registration number (`nRE`). Optional — triggers `<exportInd>` when present.
    pub n_re: Option<String>,
    /// Access key of the NF-e received for export (`chNFe`). Optional.
    pub ch_nfe: Option<String>,
    /// Quantity actually exported (`qExport`). Optional.
    pub q_export: Option<f64>,
}

impl DetExportData {
    /// Create a new empty `DetExportData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the Drawback act number.
    pub fn n_draw(mut self, v: impl Into<String>) -> Self {
        self.n_draw = Some(v.into());
        self
    }

    /// Set the export registration number.
    pub fn n_re(mut self, v: impl Into<String>) -> Self {
        self.n_re = Some(v.into());
        self
    }

    /// Set the NF-e access key for the export.
    pub fn ch_nfe(mut self, v: impl Into<String>) -> Self {
        self.ch_nfe = Some(v.into());
        self
    }

    /// Set the exported quantity.
    pub fn q_export(mut self, v: f64) -> Self {
        self.q_export = Some(v);
        self
    }
}

/// Imposto devolvido data (`<impostoDevol>` inside `<det>`).
///
/// Applicable to return/devolution invoices. Contains the devolution percentage
/// and the IPI value being returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct ImpostoDevolData {
    /// Percentage of goods returned (`pDevol`) — 2 decimal places.
    pub p_devol: Rate,
    /// IPI value being returned (`vIPIDevol`).
    pub v_ipi_devol: Cents,
}

impl ImpostoDevolData {
    /// Create a new `ImpostoDevolData`.
    pub fn new(p_devol: Rate, v_ipi_devol: Cents) -> Self {
        Self {
            p_devol,
            v_ipi_devol,
        }
    }
}

/// A referenced digital fiscal document (DFe) linked to an invoice item (`<DFeRef>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct DFeReferenciadoData {
    /// 44-digit access key of the referenced DFe.
    pub chave_acesso: String,
    /// Item number within the referenced DFe (`nItemRef`). Optional.
    pub n_item: Option<String>,
}

impl DFeReferenciadoData {
    /// Create a new `DFeReferenciadoData`.
    pub fn new(chave_acesso: impl Into<String>) -> Self {
        Self {
            chave_acesso: chave_acesso.into(),
            n_item: None,
        }
    }

    /// Set the item number.
    pub fn n_item(mut self, v: impl Into<String>) -> Self {
        self.n_item = Some(v.into());
        self
    }
}

// ── Combustíveis (comb) ──────────────────────────────────────────────────────

/// CIDE data for fuel products (`<CIDE>` inside `<comb>`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct CideData {
    /// BC da CIDE (`qBCProd`) — quantity base, formatted with 4 decimal places.
    pub q_bc_prod: String,
    /// Alíquota da CIDE (`vAliqProd`) — formatted with 4 decimal places.
    pub v_aliq_prod: String,
    /// Valor da CIDE (`vCIDE`) — formatted with 2 decimal places.
    pub v_cide: String,
}

impl CideData {
    /// Create a new `CideData` with all required fields.
    pub fn new(
        q_bc_prod: impl Into<String>,
        v_aliq_prod: impl Into<String>,
        v_cide: impl Into<String>,
    ) -> Self {
        Self {
            q_bc_prod: q_bc_prod.into(),
            v_aliq_prod: v_aliq_prod.into(),
            v_cide: v_cide.into(),
        }
    }
}

/// Encerrante (meter reading) data for fuel pump operations (`<encerrante>` inside `<comb>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct EncerranteData {
    /// Número do bico (`nBico`).
    pub n_bico: String,
    /// Número da bomba (`nBomba`). Optional.
    pub n_bomba: Option<String>,
    /// Número do tanque (`nTanque`).
    pub n_tanque: String,
    /// Valor do encerrante no início do abastecimento (`vEncIni`) — 3 decimal places.
    pub v_enc_ini: String,
    /// Valor do encerrante no final do abastecimento (`vEncFin`) — 3 decimal places.
    pub v_enc_fin: String,
}

impl EncerranteData {
    /// Create a new `EncerranteData` with required fields.
    pub fn new(
        n_bico: impl Into<String>,
        n_tanque: impl Into<String>,
        v_enc_ini: impl Into<String>,
        v_enc_fin: impl Into<String>,
    ) -> Self {
        Self {
            n_bico: n_bico.into(),
            n_bomba: None,
            n_tanque: n_tanque.into(),
            v_enc_ini: v_enc_ini.into(),
            v_enc_fin: v_enc_fin.into(),
        }
    }

    /// Set the pump number (`nBomba`).
    pub fn n_bomba(mut self, v: impl Into<String>) -> Self {
        self.n_bomba = Some(v.into());
        self
    }
}

/// Origin of fuel indicator (`<origComb>` inside `<comb>`).
///
/// NT2023_0001_v1.10: may appear multiple times per `<comb>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct OrigCombData {
    /// Indicador de importação (`indImport`): `"0"` nacional, `"1"` importado.
    pub ind_import: String,
    /// Código da UF de origem (`cUFOrig`).
    pub c_uf_orig: String,
    /// Percentual originário para a UF (`pOrig`) — 4 decimal places.
    pub p_orig: String,
}

impl OrigCombData {
    /// Create a new `OrigCombData` with all required fields.
    pub fn new(
        ind_import: impl Into<String>,
        c_uf_orig: impl Into<String>,
        p_orig: impl Into<String>,
    ) -> Self {
        Self {
            ind_import: ind_import.into(),
            c_uf_orig: c_uf_orig.into(),
            p_orig: p_orig.into(),
        }
    }
}

/// Fuel product data (`<comb>` inside `<prod>`).
///
/// Represents the complete fuel detail group per NF-e layout 4.00 and
/// NT2016_002_v1.30 / NT2023_0001_v1.10.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct CombData {
    /// Código de produto da ANP (`cProdANP`) — 9 digits.
    pub c_prod_anp: String,
    /// Descrição do produto conforme ANP (`descANP`).
    pub desc_anp: String,
    /// Percentual do GLP derivado do petróleo (`pGLP`) — 4 decimal places. Optional.
    pub p_glp: Option<String>,
    /// Percentual de Gás Natural Nacional (`pGNn`) — 4 decimal places. Optional.
    pub p_gn_n: Option<String>,
    /// Percentual de Gás Natural Importado (`pGNi`) — 4 decimal places. Optional.
    pub p_gn_i: Option<String>,
    /// Valor de partida (`vPart`) — 2 decimal places. Optional.
    pub v_part: Option<String>,
    /// Código de autorização CODIF (`CODIF`). Optional.
    pub codif: Option<String>,
    /// Quantidade de combustível faturada à temperatura ambiente (`qTemp`) — 4 decimal places. Optional.
    pub q_temp: Option<String>,
    /// Sigla da UF de consumo (`UFCons`).
    pub uf_cons: String,
    /// Dados da CIDE (`CIDE`). Optional — present when `qBCProd` is non-empty.
    pub cide: Option<CideData>,
    /// Dados do encerrante (`encerrante`). Optional.
    pub encerrante: Option<EncerranteData>,
    /// Percentual do índice de mistura do Biodiesel (`pBio`) — 4 decimal places. Optional.
    pub p_bio: Option<String>,
    /// Origens do combustível (`origComb`). Optional — may contain multiple entries.
    pub orig_comb: Option<Vec<OrigCombData>>,
}

impl CombData {
    /// Create a new `CombData` with the required fields.
    pub fn new(
        c_prod_anp: impl Into<String>,
        desc_anp: impl Into<String>,
        uf_cons: impl Into<String>,
    ) -> Self {
        Self {
            c_prod_anp: c_prod_anp.into(),
            desc_anp: desc_anp.into(),
            p_glp: None,
            p_gn_n: None,
            p_gn_i: None,
            v_part: None,
            codif: None,
            q_temp: None,
            uf_cons: uf_cons.into(),
            cide: None,
            encerrante: None,
            p_bio: None,
            orig_comb: None,
        }
    }

    /// Set the GLP percentage (`pGLP`).
    pub fn p_glp(mut self, v: impl Into<String>) -> Self {
        self.p_glp = Some(v.into());
        self
    }
    /// Set the national natural gas percentage (`pGNn`).
    pub fn p_gn_n(mut self, v: impl Into<String>) -> Self {
        self.p_gn_n = Some(v.into());
        self
    }
    /// Set the imported natural gas percentage (`pGNi`).
    pub fn p_gn_i(mut self, v: impl Into<String>) -> Self {
        self.p_gn_i = Some(v.into());
        self
    }
    /// Set the partida value (`vPart`).
    pub fn v_part(mut self, v: impl Into<String>) -> Self {
        self.v_part = Some(v.into());
        self
    }
    /// Set the CODIF code.
    pub fn codif(mut self, v: impl Into<String>) -> Self {
        self.codif = Some(v.into());
        self
    }
    /// Set the temperature-adjusted quantity (`qTemp`).
    pub fn q_temp(mut self, v: impl Into<String>) -> Self {
        self.q_temp = Some(v.into());
        self
    }
    /// Set the CIDE data.
    pub fn cide(mut self, v: CideData) -> Self {
        self.cide = Some(v);
        self
    }
    /// Set the encerrante data.
    pub fn encerrante(mut self, v: EncerranteData) -> Self {
        self.encerrante = Some(v);
        self
    }
    /// Set the biodiesel percentage (`pBio`).
    pub fn p_bio(mut self, v: impl Into<String>) -> Self {
        self.p_bio = Some(v.into());
        self
    }
    /// Set the fuel origin list (`origComb`).
    pub fn orig_comb(mut self, v: Vec<OrigCombData>) -> Self {
        self.orig_comb = Some(v);
        self
    }
}
