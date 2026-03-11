//! IS (Imposto Seletivo) XML generation for NF-e items — PL_010 tax reform.
//!
//! The IS (Imposto Seletivo) is a new consumption tax introduced by Brazil's
//! 2024 tax reform. This module provides [`IsData`] and [`build_is_xml`] to
//! generate the `<IS>` element placed inside `<imposto>`.

use crate::tax_element::{
    TaxElement, TaxField, filter_fields, optional_field, serialize_tax_element,
};

/// IS (Imposto Seletivo / IBS+CBS) input data -- PL_010 tax reform.
/// Goes inside `<imposto>` as an alternative/addition to ICMS.
///
/// String fields are pre-formatted (e.g. "100.00", "5.0000") because
/// the IS schema uses mixed decimal precisions that don't map to a
/// single cents/rate convention.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IsData {
    /// IS tax situation code
    pub cst_is: String,
    /// IS tax classification code
    pub c_class_trib_is: String,
    /// Tax base (optional, e.g. "100.00")
    pub v_bc_is: Option<String>,
    /// IS rate (optional, e.g. "5.0000")
    pub p_is: Option<String>,
    /// Specific rate (optional, e.g. "1.5000")
    pub p_is_espec: Option<String>,
    /// Taxable unit of measure (optional, e.g. "LT")
    pub u_trib: Option<String>,
    /// Taxable quantity (optional, e.g. "10.0000")
    pub q_trib: Option<String>,
    /// IS tax value (e.g. "5.00")
    pub v_is: String,
}

impl IsData {
    /// Create a new `IsData` with required fields.
    ///
    /// `cst_is` is the IS tax situation code, `c_class_trib_is` is the
    /// IS classification code, and `v_is` is the pre-formatted IS value string
    /// (e.g. `"5.00"`).
    pub fn new(
        cst_is: impl Into<String>,
        c_class_trib_is: impl Into<String>,
        v_is: impl Into<String>,
    ) -> Self {
        Self {
            cst_is: cst_is.into(),
            c_class_trib_is: c_class_trib_is.into(),
            v_is: v_is.into(),
            ..Default::default()
        }
    }
    /// Set the IS calculation base (`vBCIS`), e.g. `"100.00"`.
    pub fn v_bc_is(mut self, v: impl Into<String>) -> Self {
        self.v_bc_is = Some(v.into());
        self
    }
    /// Set the IS ad-valorem rate (`pIS`), e.g. `"5.0000"`.
    pub fn p_is(mut self, v: impl Into<String>) -> Self {
        self.p_is = Some(v.into());
        self
    }
    /// Set the IS specific rate (`pISEspec`), e.g. `"1.5000"`.
    pub fn p_is_espec(mut self, v: impl Into<String>) -> Self {
        self.p_is_espec = Some(v.into());
        self
    }
    /// Set the taxable unit of measure (`uTrib`), e.g. `"LT"`.
    pub fn u_trib(mut self, v: impl Into<String>) -> Self {
        self.u_trib = Some(v.into());
        self
    }
    /// Set the taxable quantity (`qTrib`), e.g. `"10.0000"`.
    pub fn q_trib(mut self, v: impl Into<String>) -> Self {
        self.q_trib = Some(v.into());
        self
    }
}

/// Calculate IS tax element (domain logic, no XML dependency).
///
/// Three mutually exclusive modes based on which fields are present:
/// - `v_bc_is` present: ad-valorem mode (includes pIS, pISEspec)
/// - `u_trib` + `q_trib` present: specific quantity mode
/// - Neither: simple CST + classification + value
fn calculate_is(data: &IsData) -> TaxElement {
    let mut fields: Vec<Option<TaxField>> = vec![
        Some(TaxField::new("CSTIS", &data.cst_is)),
        Some(TaxField::new("cClassTribIS", &data.c_class_trib_is)),
    ];

    if let Some(ref v_bc_is) = data.v_bc_is {
        fields.push(Some(TaxField::new("vBCIS", v_bc_is)));
        fields.push(optional_field("pIS", data.p_is.as_deref()));
        fields.push(optional_field("pISEspec", data.p_is_espec.as_deref()));
    }

    if let (Some(u_trib), Some(q_trib)) = (&data.u_trib, &data.q_trib) {
        fields.push(Some(TaxField::new("uTrib", u_trib)));
        fields.push(Some(TaxField::new("qTrib", q_trib)));
    }

    fields.push(Some(TaxField::new("vIS", &data.v_is)));

    TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "IS".into(),
        fields: filter_fields(fields),
    }
}

/// Build IS (IBS/CBS) XML string.
pub fn build_is_xml(data: &IsData) -> String {
    serialize_tax_element(&calculate_is(data))
}
