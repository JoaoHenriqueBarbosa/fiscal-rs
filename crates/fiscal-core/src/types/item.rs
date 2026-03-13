use super::{
    ArmaData, CombData, DFeReferenciadoData, DetExportData, DiData, GCredData, ImpostoDevolData,
    MedData, ObsItemData, RastroData, VeicProdData,
};
use crate::newtypes::{Cents, Rate, Rate4};

/// Complete data for a single invoice line item (`<det>`), including product
/// identification, pricing, and all applicable taxes.
///
/// Required fields are supplied via [`InvoiceItemData::new`]; optional fields
/// (shipping, discounts, extended tax fields, specialised product data) are set
/// via chainable setter methods.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InvoiceItemData {
    /// Sequential item number (`nItem`, 1-based).
    pub item_number: u32,
    /// Issuer's internal product code (`cProd`).
    pub product_code: String,
    /// Product or service description (`xProd`).
    pub description: String,
    /// NCM (Nomenclatura Comum do MERCOSUL) classification code.
    pub ncm: String,
    /// NVE (Nomenclatura de Valor Aduaneiro e Estatística) codes.
    /// Up to 8 NVE codes per item (I05a).
    pub nve: Vec<String>,
    /// CFOP operation code (4 digits).
    pub cfop: String,
    /// Commercial unit of measure (`uCom`), e.g. `"UN"`, `"KG"`.
    pub unit_of_measure: String,
    /// Quantity in commercial units (`qCom`).
    pub quantity: f64,
    /// Commercial unit price (`vUnCom`).
    pub unit_price: Cents,
    /// Total item value (`vProd = qCom × vUnCom`).
    pub total_price: Cents,
    /// GTIN / EAN barcode for commercial units (`cEAN`). `None` = no barcode.
    pub c_ean: Option<String>,
    /// Non-GTIN barcode for commercial units (`cBarra`). Optional.
    pub c_barra: Option<String>,
    /// GTIN / EAN barcode for the taxation unit (`cEANTrib`). `None` = no barcode.
    pub c_ean_trib: Option<String>,
    /// Non-GTIN barcode for the taxation unit (`cBarraTrib`). Optional.
    pub c_barra_trib: Option<String>,
    /// Taxable unit of measure (`uTrib`). When `None`, defaults to `unit_of_measure`.
    pub taxable_unit: Option<String>,
    /// Taxable quantity (`qTrib`). When `None`, defaults to `quantity`.
    pub taxable_quantity: Option<f64>,
    /// Taxable unit price (`vUnTrib`). When `None`, defaults to `unit_price`.
    pub taxable_unit_price: Option<Cents>,
    /// CEST code for ST-subject products (`CEST`). Optional.
    pub cest: Option<String>,
    /// CEST scale indicator (`indEscala`). Optional — "S" or "N".
    pub cest_ind_escala: Option<String>,
    /// CEST manufacturer CNPJ (`CNPJFab`). Optional.
    pub cest_cnpj_fab: Option<String>,
    /// Tax benefit code (`cBenef`). Optional.
    pub c_benef: Option<String>,
    /// Classificação para subapuração do IBS na ZFM (`tpCredPresIBSZFM`).
    /// PL_010 only — emitted inside `<prod>` after `<cBenef>`.
    pub tp_cred_pres_ibs_zfm: Option<String>,
    /// Crédito presumido ICMS entries (`gCred`). Optional — up to 4 per item.
    pub g_cred: Vec<GCredData>,
    /// TIPI exception code (`EXTIPI`). Optional.
    pub extipi: Option<String>,
    /// Purchase order number (`xPed`). Optional.
    pub x_ped: Option<String>,
    /// Purchase order item number (`nItemPed`). Optional.
    pub n_item_ped: Option<String>,
    /// FCI number — Ficha de Conteúdo de Importação (`nFCI`). Optional.
    pub n_fci: Option<String>,
    /// Freight value allocated to this item (`vFrete`). Optional.
    pub v_frete: Option<Cents>,
    /// Insurance value allocated to this item (`vSeg`). Optional.
    pub v_seg: Option<Cents>,
    /// Discount value for this item (`vDesc`). Optional.
    pub v_desc: Option<Cents>,
    /// Other costs allocated to this item (`vOutro`). Optional.
    pub v_outro: Option<Cents>,
    /// Product origin code (`orig`), e.g. `"0"` (domestic). Optional.
    pub orig: Option<String>,
    // ── ICMS ────────────────────────────────────────────────────────────────
    /// ICMS CST or CSOSN code (2–3 digits).
    pub icms_cst: String,
    /// ICMS rate applied to this item (`pICMS`).
    pub icms_rate: Rate,
    /// ICMS value for this item (`vICMS`).
    pub icms_amount: Cents,
    /// ICMS base calculation modality (`modBC`). Optional.
    pub icms_mod_bc: Option<i64>,
    /// ICMS base reduction rate (`pRedBC`). Optional.
    pub icms_red_bc: Option<Rate>,
    /// ICMS ST base calculation modality (`modBCST`). Optional.
    pub icms_mod_bc_st: Option<i64>,
    /// ICMS ST added value margin (`pMVAST`). Optional.
    pub icms_p_mva_st: Option<Rate>,
    /// ICMS ST base reduction rate (`pRedBCST`). Optional.
    pub icms_red_bc_st: Option<Rate>,
    /// ICMS ST calculation base value (`vBCST`). Optional.
    pub icms_v_bc_st: Option<Cents>,
    /// ICMS ST rate (`pICMSST`). Optional.
    pub icms_p_icms_st: Option<Rate>,
    /// ICMS ST value (`vICMSST`). Optional.
    pub icms_v_icms_st: Option<Cents>,
    /// Desonerated ICMS value (`vICMSDeson`). Optional.
    pub icms_v_icms_deson: Option<Cents>,
    /// Reason code for ICMS desoneration (`motDesICMS`). Optional.
    pub icms_mot_des_icms: Option<i64>,
    /// FCP rate (`pFCP`). Optional.
    pub icms_p_fcp: Option<Rate>,
    /// FCP value (`vFCP`). Optional.
    pub icms_v_fcp: Option<Cents>,
    /// FCP calculation base (`vBCFCP`). Optional.
    pub icms_v_bc_fcp: Option<Cents>,
    /// FCP-ST rate (`pFCPST`). Optional.
    pub icms_p_fcp_st: Option<Rate>,
    /// FCP-ST value (`vFCPST`). Optional.
    pub icms_v_fcp_st: Option<Cents>,
    /// FCP-ST calculation base (`vBCFCPST`). Optional.
    pub icms_v_bc_fcp_st: Option<Cents>,
    /// Simples Nacional ICMS credit rate (`pCredSN`). Optional.
    pub icms_p_cred_sn: Option<Rate>,
    /// Simples Nacional ICMS credit value (`vCredICMSSN`). Optional.
    pub icms_v_cred_icms_sn: Option<Cents>,
    /// ICMS substitute value (`vICMSSubstituto`). Optional.
    pub icms_v_icms_substituto: Option<Cents>,
    /// ICMS desoneration deduction indicator (`indDeduzDeson`). Optional.
    /// When `"1"`, the desonerated value is deducted from vNF.
    pub icms_ind_deduz_deson: Option<String>,
    // ── PIS ─────────────────────────────────────────────────────────────────
    /// PIS CST code (2 digits).
    pub pis_cst: String,
    /// PIS calculation base value (`vBCPIS`). Optional.
    pub pis_v_bc: Option<Cents>,
    /// PIS rate (`pPIS`). Optional.
    pub pis_p_pis: Option<Rate4>,
    /// PIS value (`vPIS`). Optional.
    pub pis_v_pis: Option<Cents>,
    /// PIS quantity base (`qBCProd`). Optional.
    pub pis_q_bc_prod: Option<i64>,
    /// PIS unit value (`vAliqProd`) for quantity-based calculation. Optional.
    pub pis_v_aliq_prod: Option<i64>,
    // ── COFINS ──────────────────────────────────────────────────────────────
    /// COFINS CST code (2 digits).
    pub cofins_cst: String,
    /// COFINS calculation base value (`vBCCOFINS`). Optional.
    pub cofins_v_bc: Option<Cents>,
    /// COFINS rate (`pCOFINS`). Optional.
    pub cofins_p_cofins: Option<Rate4>,
    /// COFINS value (`vCOFINS`). Optional.
    pub cofins_v_cofins: Option<Cents>,
    /// COFINS quantity base (`qBCProd`). Optional.
    pub cofins_q_bc_prod: Option<i64>,
    /// COFINS unit value (`vAliqProd`) for quantity-based calculation. Optional.
    pub cofins_v_aliq_prod: Option<i64>,
    // ── IPI ─────────────────────────────────────────────────────────────────
    /// IPI CST code. Optional (only for industrialised products).
    pub ipi_cst: Option<String>,
    /// IPI enquadramento (classification) code (`cEnq`). Optional.
    pub ipi_c_enq: Option<String>,
    /// IPI calculation base (`vBCIPI`). Optional.
    pub ipi_v_bc: Option<Cents>,
    /// IPI rate (`pIPI`). Optional.
    pub ipi_p_ipi: Option<Rate>,
    /// IPI value (`vIPI`). Optional.
    pub ipi_v_ipi: Option<Cents>,
    /// IPI quantity base (`qUnid`). Optional.
    pub ipi_q_unid: Option<i64>,
    /// IPI unit value (`vUnid`). Optional.
    pub ipi_v_unid: Option<i64>,
    // ── II (Import Duty) ─────────────────────────────────────────────────────
    /// Import duty (II) calculation base (`vBCII`). Optional.
    pub ii_v_bc: Option<Cents>,
    /// Customs clearance expenses (`vDespAdu`). Optional.
    pub ii_v_desp_adu: Option<Cents>,
    /// Import duty value (`vII`). Optional.
    pub ii_v_ii: Option<Cents>,
    /// IOF (financial operations tax) for imports (`vIOF`). Optional.
    pub ii_v_iof: Option<Cents>,
    // ── Specialised product data ─────────────────────────────────────────────
    /// Batch / lot traceability records (`rastro`). Optional.
    pub rastro: Option<Vec<RastroData>>,
    /// Vehicle product details (`veicProd`). Optional.
    pub veic_prod: Option<VeicProdData>,
    /// Medicine / pharmaceutical product details (`med`). Optional.
    pub med: Option<MedData>,
    /// Firearm / weapon details (`arma`). Optional.
    pub arma: Option<Vec<ArmaData>>,
    /// Fuel product data (`comb`). Optional.
    pub comb: Option<CombData>,
    /// RECOPI number for paper / printing sector products. Optional.
    pub n_recopi: Option<String>,
    /// ISSQN data for service items. Optional.
    /// When present, the `<ISSQN>` element is emitted inside `<imposto>` instead of ICMS.
    pub issqn: Option<crate::tax_issqn::IssqnData>,
    /// Additional product information printed on the DANFE (`infAdProd`). Optional.
    pub inf_ad_prod: Option<String>,
    /// Per-item observations (`obsItem`). Optional.
    pub obs_item: Option<ObsItemData>,
    /// Referenced digital fiscal document for this item (`DFeRef`). Optional.
    pub dfe_referenciado: Option<DFeReferenciadoData>,
    // ── Import / export / devolution ─────────────────────────────────────
    /// Import declarations (`DI`). Optional — may contain multiple DIs per item.
    pub di: Option<Vec<DiData>>,
    /// Export details (`detExport`). Optional — may contain multiple entries per item.
    pub det_export: Option<Vec<DetExportData>>,
    /// Imposto devolvido (`impostoDevol`). Optional — for return/devolution invoices.
    pub imposto_devol: Option<ImpostoDevolData>,
    /// Whether this item counts towards the invoice total (`indTot`).
    /// `1` (default) = included in total, `0` = not included.
    pub ind_tot: Option<u8>,
    /// Indicator for used movable goods (`indBemMovelUsado`). Optional.
    pub ind_bem_movel_usado: Option<bool>,
    /// Approximate total tax for this item (`vTotTrib`). Optional.
    pub v_tot_trib: Option<Cents>,
    /// IS (Imposto Seletivo) data for this item. Optional.
    pub is_data: Option<crate::tax_is::IsData>,
    /// IBS/CBS (Imposto sobre Bens e Servicos / Contribuicao sobre Bens e Servicos) data. Optional.
    pub ibs_cbs: Option<crate::tax_ibs_cbs::IbsCbsData>,
    /// Valor Total do Item (`vItem`). PL_010 only — emitted inside `<det>` when IBS/CBS exists.
    /// When `None` and IBS/CBS is present, the builder auto-calculates from item values.
    pub v_item: Option<Cents>,
    /// PIS-ST (substituição tributária) data for this item. Optional.
    pub pis_st: Option<crate::tax_pis_cofins_ipi::PisStData>,
    /// COFINS-ST (substituição tributária) data for this item. Optional.
    pub cofins_st: Option<crate::tax_pis_cofins_ipi::CofinsStData>,
}

impl InvoiceItemData {
    /// Create a new `InvoiceItemData` with required fields.
    /// All optional fields default to `None` or zero.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        item_number: u32,
        product_code: impl Into<String>,
        description: impl Into<String>,
        ncm: impl Into<String>,
        cfop: impl Into<String>,
        unit_of_measure: impl Into<String>,
        quantity: f64,
        unit_price: Cents,
        total_price: Cents,
        icms_cst: impl Into<String>,
        icms_rate: Rate,
        icms_amount: Cents,
        pis_cst: impl Into<String>,
        cofins_cst: impl Into<String>,
    ) -> Self {
        Self {
            item_number,
            product_code: product_code.into(),
            description: description.into(),
            ncm: ncm.into(),
            nve: Vec::new(),
            cfop: cfop.into(),
            unit_of_measure: unit_of_measure.into(),
            quantity,
            unit_price,
            total_price,
            c_ean: None,
            c_barra: None,
            c_ean_trib: None,
            c_barra_trib: None,
            taxable_unit: None,
            taxable_quantity: None,
            taxable_unit_price: None,
            cest: None,
            cest_ind_escala: None,
            cest_cnpj_fab: None,
            c_benef: None,
            tp_cred_pres_ibs_zfm: None,
            g_cred: Vec::new(),
            extipi: None,
            x_ped: None,
            n_item_ped: None,
            n_fci: None,
            v_frete: None,
            v_seg: None,
            v_desc: None,
            v_outro: None,
            orig: None,
            icms_cst: icms_cst.into(),
            icms_rate,
            icms_amount,
            icms_mod_bc: None,
            icms_red_bc: None,
            icms_mod_bc_st: None,
            icms_p_mva_st: None,
            icms_red_bc_st: None,
            icms_v_bc_st: None,
            icms_p_icms_st: None,
            icms_v_icms_st: None,
            icms_v_icms_deson: None,
            icms_mot_des_icms: None,
            icms_p_fcp: None,
            icms_v_fcp: None,
            icms_v_bc_fcp: None,
            icms_p_fcp_st: None,
            icms_v_fcp_st: None,
            icms_v_bc_fcp_st: None,
            icms_p_cred_sn: None,
            icms_v_cred_icms_sn: None,
            icms_v_icms_substituto: None,
            icms_ind_deduz_deson: None,
            pis_cst: pis_cst.into(),
            pis_v_bc: None,
            pis_p_pis: None,
            pis_v_pis: None,
            pis_q_bc_prod: None,
            pis_v_aliq_prod: None,
            cofins_cst: cofins_cst.into(),
            cofins_v_bc: None,
            cofins_p_cofins: None,
            cofins_v_cofins: None,
            cofins_q_bc_prod: None,
            cofins_v_aliq_prod: None,
            ipi_cst: None,
            ipi_c_enq: None,
            ipi_v_bc: None,
            ipi_p_ipi: None,
            ipi_v_ipi: None,
            ipi_q_unid: None,
            ipi_v_unid: None,
            ii_v_bc: None,
            ii_v_desp_adu: None,
            ii_v_ii: None,
            ii_v_iof: None,
            rastro: None,
            veic_prod: None,
            med: None,
            arma: None,
            comb: None,
            n_recopi: None,
            issqn: None,
            inf_ad_prod: None,
            obs_item: None,
            dfe_referenciado: None,
            di: None,
            det_export: None,
            imposto_devol: None,
            ind_tot: None,
            ind_bem_movel_usado: None,
            v_tot_trib: None,
            is_data: None,
            ibs_cbs: None,
            v_item: None,
            pis_st: None,
            cofins_st: None,
        }
    }

    // Chainable setters for optional fields
    /// Set the EAN code.
    pub fn c_ean(mut self, v: impl Into<String>) -> Self {
        self.c_ean = Some(v.into());
        self
    }
    /// Set the non-GTIN barcode for commercial units (`cBarra`).
    pub fn c_barra(mut self, v: impl Into<String>) -> Self {
        self.c_barra = Some(v.into());
        self
    }
    /// Set the tributary EAN code.
    pub fn c_ean_trib(mut self, v: impl Into<String>) -> Self {
        self.c_ean_trib = Some(v.into());
        self
    }
    /// Set the non-GTIN barcode for the taxation unit (`cBarraTrib`).
    pub fn c_barra_trib(mut self, v: impl Into<String>) -> Self {
        self.c_barra_trib = Some(v.into());
        self
    }
    /// Set the taxable unit of measure (`uTrib`). Defaults to `unit_of_measure` if not set.
    pub fn taxable_unit(mut self, v: impl Into<String>) -> Self {
        self.taxable_unit = Some(v.into());
        self
    }
    /// Set the taxable quantity (`qTrib`). Defaults to `quantity` if not set.
    pub fn taxable_quantity(mut self, v: f64) -> Self {
        self.taxable_quantity = Some(v);
        self
    }
    /// Set the taxable unit price (`vUnTrib`). Defaults to `unit_price` if not set.
    pub fn taxable_unit_price(mut self, v: Cents) -> Self {
        self.taxable_unit_price = Some(v);
        self
    }
    /// Add an NVE code (Nomenclatura de Valor Aduaneiro e Estatística).
    /// Up to 8 NVE codes can be added per item (I05a).
    pub fn nve(mut self, v: impl Into<String>) -> Self {
        self.nve.push(v.into());
        self
    }
    /// Set the CEST code.
    pub fn cest(mut self, v: impl Into<String>) -> Self {
        self.cest = Some(v.into());
        self
    }
    /// Set the CEST scale indicator (`indEscala`).
    pub fn cest_ind_escala(mut self, v: impl Into<String>) -> Self {
        self.cest_ind_escala = Some(v.into());
        self
    }
    /// Set the CEST manufacturer CNPJ (`CNPJFab`).
    pub fn cest_cnpj_fab(mut self, v: impl Into<String>) -> Self {
        self.cest_cnpj_fab = Some(v.into());
        self
    }
    /// Set the tax benefit code (`cBenef`).
    pub fn c_benef(mut self, v: impl Into<String>) -> Self {
        self.c_benef = Some(v.into());
        self
    }
    /// Set the IBS ZFM credit classification (`tpCredPresIBSZFM`). PL_010 only.
    pub fn tp_cred_pres_ibs_zfm(mut self, v: impl Into<String>) -> Self {
        self.tp_cred_pres_ibs_zfm = Some(v.into());
        self
    }
    /// Set crédito presumido ICMS entries (`gCred`). Up to 4 per item.
    pub fn g_cred(mut self, v: Vec<GCredData>) -> Self {
        self.g_cred = v;
        self
    }
    /// Set the TIPI exception code (`EXTIPI`).
    pub fn extipi(mut self, v: impl Into<String>) -> Self {
        self.extipi = Some(v.into());
        self
    }
    /// Set the purchase order number (`xPed`).
    pub fn x_ped(mut self, v: impl Into<String>) -> Self {
        self.x_ped = Some(v.into());
        self
    }
    /// Set the purchase order item number (`nItemPed`).
    pub fn n_item_ped(mut self, v: impl Into<String>) -> Self {
        self.n_item_ped = Some(v.into());
        self
    }
    /// Set the FCI number (`nFCI`).
    pub fn n_fci(mut self, v: impl Into<String>) -> Self {
        self.n_fci = Some(v.into());
        self
    }
    /// Set the freight value.
    pub fn v_frete(mut self, v: Cents) -> Self {
        self.v_frete = Some(v);
        self
    }
    /// Set the insurance value.
    pub fn v_seg(mut self, v: Cents) -> Self {
        self.v_seg = Some(v);
        self
    }
    /// Set the discount value.
    pub fn v_desc(mut self, v: Cents) -> Self {
        self.v_desc = Some(v);
        self
    }
    /// Set the "other" value.
    pub fn v_outro(mut self, v: Cents) -> Self {
        self.v_outro = Some(v);
        self
    }
    /// Set the origin code.
    pub fn orig(mut self, v: impl Into<String>) -> Self {
        self.orig = Some(v.into());
        self
    }
    /// Set the ICMS base calculation modality.
    pub fn icms_mod_bc(mut self, v: i64) -> Self {
        self.icms_mod_bc = Some(v);
        self
    }
    /// Set the ICMS base reduction rate.
    pub fn icms_red_bc(mut self, v: Rate) -> Self {
        self.icms_red_bc = Some(v);
        self
    }
    /// Set the ICMS ST base calculation modality.
    pub fn icms_mod_bc_st(mut self, v: i64) -> Self {
        self.icms_mod_bc_st = Some(v);
        self
    }
    /// Set the ICMS ST MVA rate.
    pub fn icms_p_mva_st(mut self, v: Rate) -> Self {
        self.icms_p_mva_st = Some(v);
        self
    }
    /// Set the ICMS ST base reduction rate.
    pub fn icms_red_bc_st(mut self, v: Rate) -> Self {
        self.icms_red_bc_st = Some(v);
        self
    }
    /// Set the ICMS ST base value.
    pub fn icms_v_bc_st(mut self, v: Cents) -> Self {
        self.icms_v_bc_st = Some(v);
        self
    }
    /// Set the ICMS ST rate.
    pub fn icms_p_icms_st(mut self, v: Rate) -> Self {
        self.icms_p_icms_st = Some(v);
        self
    }
    /// Set the ICMS ST value.
    pub fn icms_v_icms_st(mut self, v: Cents) -> Self {
        self.icms_v_icms_st = Some(v);
        self
    }
    /// Set the desonerated ICMS value.
    pub fn icms_v_icms_deson(mut self, v: Cents) -> Self {
        self.icms_v_icms_deson = Some(v);
        self
    }
    /// Set the ICMS desonerating motive.
    pub fn icms_mot_des_icms(mut self, v: i64) -> Self {
        self.icms_mot_des_icms = Some(v);
        self
    }
    /// Set the FCP rate.
    pub fn icms_p_fcp(mut self, v: Rate) -> Self {
        self.icms_p_fcp = Some(v);
        self
    }
    /// Set the FCP value.
    pub fn icms_v_fcp(mut self, v: Cents) -> Self {
        self.icms_v_fcp = Some(v);
        self
    }
    /// Set the FCP base value.
    pub fn icms_v_bc_fcp(mut self, v: Cents) -> Self {
        self.icms_v_bc_fcp = Some(v);
        self
    }
    /// Set the FCP ST rate.
    pub fn icms_p_fcp_st(mut self, v: Rate) -> Self {
        self.icms_p_fcp_st = Some(v);
        self
    }
    /// Set the FCP ST value.
    pub fn icms_v_fcp_st(mut self, v: Cents) -> Self {
        self.icms_v_fcp_st = Some(v);
        self
    }
    /// Set the FCP ST base value.
    pub fn icms_v_bc_fcp_st(mut self, v: Cents) -> Self {
        self.icms_v_bc_fcp_st = Some(v);
        self
    }
    /// Set the Simples Nacional credit rate.
    pub fn icms_p_cred_sn(mut self, v: Rate) -> Self {
        self.icms_p_cred_sn = Some(v);
        self
    }
    /// Set the Simples Nacional credit ICMS value.
    pub fn icms_v_cred_icms_sn(mut self, v: Cents) -> Self {
        self.icms_v_cred_icms_sn = Some(v);
        self
    }
    /// Set the ICMS substitute value.
    pub fn icms_v_icms_substituto(mut self, v: Cents) -> Self {
        self.icms_v_icms_substituto = Some(v);
        self
    }
    /// Set the ICMS desoneration deduction indicator (`indDeduzDeson`).
    /// Value `"1"` means the desonerated value is deducted from vNF.
    pub fn icms_ind_deduz_deson(mut self, v: impl Into<String>) -> Self {
        self.icms_ind_deduz_deson = Some(v.into());
        self
    }
    /// Set the PIS base value.
    pub fn pis_v_bc(mut self, v: Cents) -> Self {
        self.pis_v_bc = Some(v);
        self
    }
    /// Set the PIS rate.
    pub fn pis_p_pis(mut self, v: Rate4) -> Self {
        self.pis_p_pis = Some(v);
        self
    }
    /// Set the PIS value.
    pub fn pis_v_pis(mut self, v: Cents) -> Self {
        self.pis_v_pis = Some(v);
        self
    }
    /// Set the PIS quantity base.
    pub fn pis_q_bc_prod(mut self, v: i64) -> Self {
        self.pis_q_bc_prod = Some(v);
        self
    }
    /// Set the PIS quantity rate.
    pub fn pis_v_aliq_prod(mut self, v: i64) -> Self {
        self.pis_v_aliq_prod = Some(v);
        self
    }
    /// Set the COFINS base value.
    pub fn cofins_v_bc(mut self, v: Cents) -> Self {
        self.cofins_v_bc = Some(v);
        self
    }
    /// Set the COFINS rate.
    pub fn cofins_p_cofins(mut self, v: Rate4) -> Self {
        self.cofins_p_cofins = Some(v);
        self
    }
    /// Set the COFINS value.
    pub fn cofins_v_cofins(mut self, v: Cents) -> Self {
        self.cofins_v_cofins = Some(v);
        self
    }
    /// Set the COFINS quantity base.
    pub fn cofins_q_bc_prod(mut self, v: i64) -> Self {
        self.cofins_q_bc_prod = Some(v);
        self
    }
    /// Set the COFINS quantity rate.
    pub fn cofins_v_aliq_prod(mut self, v: i64) -> Self {
        self.cofins_v_aliq_prod = Some(v);
        self
    }
    /// Set the IPI CST.
    pub fn ipi_cst(mut self, v: impl Into<String>) -> Self {
        self.ipi_cst = Some(v.into());
        self
    }
    /// Set the IPI enquadramento code.
    pub fn ipi_c_enq(mut self, v: impl Into<String>) -> Self {
        self.ipi_c_enq = Some(v.into());
        self
    }
    /// Set the IPI base value.
    pub fn ipi_v_bc(mut self, v: Cents) -> Self {
        self.ipi_v_bc = Some(v);
        self
    }
    /// Set the IPI rate.
    pub fn ipi_p_ipi(mut self, v: Rate) -> Self {
        self.ipi_p_ipi = Some(v);
        self
    }
    /// Set the IPI value.
    pub fn ipi_v_ipi(mut self, v: Cents) -> Self {
        self.ipi_v_ipi = Some(v);
        self
    }
    /// Set the IPI quantity.
    pub fn ipi_q_unid(mut self, v: i64) -> Self {
        self.ipi_q_unid = Some(v);
        self
    }
    /// Set the IPI unit value.
    pub fn ipi_v_unid(mut self, v: i64) -> Self {
        self.ipi_v_unid = Some(v);
        self
    }
    /// Set the II base value.
    pub fn ii_v_bc(mut self, v: Cents) -> Self {
        self.ii_v_bc = Some(v);
        self
    }
    /// Set the II customs expenses.
    pub fn ii_v_desp_adu(mut self, v: Cents) -> Self {
        self.ii_v_desp_adu = Some(v);
        self
    }
    /// Set the II value.
    pub fn ii_v_ii(mut self, v: Cents) -> Self {
        self.ii_v_ii = Some(v);
        self
    }
    /// Set the II IOF value.
    pub fn ii_v_iof(mut self, v: Cents) -> Self {
        self.ii_v_iof = Some(v);
        self
    }
    /// Set batch tracking data.
    pub fn rastro(mut self, v: Vec<RastroData>) -> Self {
        self.rastro = Some(v);
        self
    }
    /// Set vehicle product data.
    pub fn veic_prod(mut self, v: VeicProdData) -> Self {
        self.veic_prod = Some(v);
        self
    }
    /// Set medicine data.
    pub fn med(mut self, v: MedData) -> Self {
        self.med = Some(v);
        self
    }
    /// Set weapon data.
    pub fn arma(mut self, v: Vec<ArmaData>) -> Self {
        self.arma = Some(v);
        self
    }
    /// Set fuel product data.
    pub fn comb(mut self, v: CombData) -> Self {
        self.comb = Some(v);
        self
    }
    /// Set RECOPI number.
    pub fn n_recopi(mut self, v: impl Into<String>) -> Self {
        self.n_recopi = Some(v.into());
        self
    }
    /// Set ISSQN data for service items.
    pub fn issqn(mut self, v: crate::tax_issqn::IssqnData) -> Self {
        self.issqn = Some(v);
        self
    }
    /// Set additional product info.
    pub fn inf_ad_prod(mut self, v: impl Into<String>) -> Self {
        self.inf_ad_prod = Some(v.into());
        self
    }
    /// Set per-item observation data.
    pub fn obs_item(mut self, v: ObsItemData) -> Self {
        self.obs_item = Some(v);
        self
    }
    /// Set referenced DFe data.
    pub fn dfe_referenciado(mut self, v: DFeReferenciadoData) -> Self {
        self.dfe_referenciado = Some(v);
        self
    }
    /// Set import declarations (DI).
    pub fn di(mut self, v: Vec<DiData>) -> Self {
        self.di = Some(v);
        self
    }
    /// Set export details (detExport).
    pub fn det_export(mut self, v: Vec<DetExportData>) -> Self {
        self.det_export = Some(v);
        self
    }
    /// Set imposto devolvido data for return invoices.
    pub fn imposto_devol(mut self, v: ImpostoDevolData) -> Self {
        self.imposto_devol = Some(v);
        self
    }
    /// Set the total indicator (`indTot`). Default is `1` (included in total).
    /// Set to `0` to exclude from invoice total.
    pub fn ind_tot(mut self, v: u8) -> Self {
        self.ind_tot = Some(v);
        self
    }
    /// Set the used movable goods indicator (`indBemMovelUsado`).
    pub fn ind_bem_movel_usado(mut self, v: bool) -> Self {
        self.ind_bem_movel_usado = Some(v);
        self
    }
    /// Set the approximate total tax (`vTotTrib`).
    pub fn v_tot_trib(mut self, v: Cents) -> Self {
        self.v_tot_trib = Some(v);
        self
    }
    /// Set IS (Imposto Seletivo) data.
    pub fn is_data(mut self, v: crate::tax_is::IsData) -> Self {
        self.is_data = Some(v);
        self
    }
    /// Set IBS/CBS data.
    pub fn ibs_cbs(mut self, v: crate::tax_ibs_cbs::IbsCbsData) -> Self {
        self.ibs_cbs = Some(v);
        self
    }
    /// Set the total item value (`vItem`). PL_010 only.
    /// When not set and IBS/CBS data exists, the builder auto-calculates this value.
    pub fn v_item(mut self, v: Cents) -> Self {
        self.v_item = Some(v);
        self
    }
    /// Set PIS-ST (substituição tributária) data.
    pub fn pis_st(mut self, v: crate::tax_pis_cofins_ipi::PisStData) -> Self {
        self.pis_st = Some(v);
        self
    }
    /// Set COFINS-ST (substituição tributária) data.
    pub fn cofins_st(mut self, v: crate::tax_pis_cofins_ipi::CofinsStData) -> Self {
        self.cofins_st = Some(v);
        self
    }
}
