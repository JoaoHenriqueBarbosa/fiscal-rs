use crate::tax_element::{filter_fields, optional_field, serialize_tax_element, TaxElement, TaxField};

/// IS (Imposto Seletivo / IBS+CBS) input data -- PL_010 tax reform.
/// Goes inside `<imposto>` as an alternative/addition to ICMS.
///
/// String fields are pre-formatted (e.g. "100.00", "5.0000") because
/// the IS schema uses mixed decimal precisions that don't map to a
/// single cents/rate convention.
#[derive(Debug, Clone, Default)]
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
