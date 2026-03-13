use crate::newtypes::Cents;

// ── ISSQN total data ────────────────────────────────────────────────────────

/// ISSQN total data (`<ISSQNtot>` inside `<total>`).
///
/// When the invoice has service items with ISSQN, this group is emitted
/// after `<ICMSTot>`.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IssqnTotData {
    /// Total services value (`vServ`) in cents. Optional — only emitted when > 0.
    pub v_serv: Option<Cents>,
    /// Total ISS base (`vBC`) in cents. Optional — only emitted when > 0.
    pub v_bc: Option<Cents>,
    /// Total ISS value (`vISS`) in cents. Optional — only emitted when > 0.
    pub v_iss: Option<Cents>,
    /// Total PIS on services (`vPIS`) in cents. Optional — only emitted when > 0.
    pub v_pis: Option<Cents>,
    /// Total COFINS on services (`vCOFINS`) in cents. Optional — only emitted when > 0.
    pub v_cofins: Option<Cents>,
    /// Service competence date (`dCompet`) in `YYYY-MM-DD` format.
    pub d_compet: String,
    /// Total deduction (`vDeducao`) in cents. Optional — only emitted when > 0.
    pub v_deducao: Option<Cents>,
    /// Total other retentions (`vOutro`) in cents. Optional — only emitted when > 0.
    pub v_outro: Option<Cents>,
    /// Total unconditional discount (`vDescIncond`) in cents. Optional — only emitted when > 0.
    pub v_desc_incond: Option<Cents>,
    /// Total conditional discount (`vDescCond`) in cents. Optional — only emitted when > 0.
    pub v_desc_cond: Option<Cents>,
    /// Total ISS retention (`vISSRet`) in cents. Optional — only emitted when > 0.
    pub v_iss_ret: Option<Cents>,
    /// Tax regime code (`cRegTrib`). Optional.
    pub c_reg_trib: Option<String>,
}

impl IssqnTotData {
    /// Create a new `IssqnTotData` with the required competence date.
    pub fn new(d_compet: impl Into<String>) -> Self {
        Self {
            d_compet: d_compet.into(),
            ..Default::default()
        }
    }

    /// Set the total services value.
    pub fn v_serv(mut self, v: Cents) -> Self {
        self.v_serv = Some(v);
        self
    }
    /// Set the total ISS base.
    pub fn v_bc(mut self, v: Cents) -> Self {
        self.v_bc = Some(v);
        self
    }
    /// Set the total ISS value.
    pub fn v_iss(mut self, v: Cents) -> Self {
        self.v_iss = Some(v);
        self
    }
    /// Set the total PIS on services.
    pub fn v_pis(mut self, v: Cents) -> Self {
        self.v_pis = Some(v);
        self
    }
    /// Set the total COFINS on services.
    pub fn v_cofins(mut self, v: Cents) -> Self {
        self.v_cofins = Some(v);
        self
    }
    /// Set the total deduction.
    pub fn v_deducao(mut self, v: Cents) -> Self {
        self.v_deducao = Some(v);
        self
    }
    /// Set the total other retentions.
    pub fn v_outro(mut self, v: Cents) -> Self {
        self.v_outro = Some(v);
        self
    }
    /// Set the total unconditional discount.
    pub fn v_desc_incond(mut self, v: Cents) -> Self {
        self.v_desc_incond = Some(v);
        self
    }
    /// Set the total conditional discount.
    pub fn v_desc_cond(mut self, v: Cents) -> Self {
        self.v_desc_cond = Some(v);
        self
    }
    /// Set the total ISS retention.
    pub fn v_iss_ret(mut self, v: Cents) -> Self {
        self.v_iss_ret = Some(v);
        self
    }
    /// Set the tax regime code (`cRegTrib`).
    pub fn c_reg_trib(mut self, v: impl Into<String>) -> Self {
        self.c_reg_trib = Some(v.into());
        self
    }
}
