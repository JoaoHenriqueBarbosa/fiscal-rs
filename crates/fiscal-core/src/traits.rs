//! Sealed public traits for tax calculation and XML serialization.

use crate::sealed::private::Sealed;
use crate::tax_element::{TaxElement, serialize_tax_element};
use crate::tax_icms::{IcmsVariant, build_icms_xml, create_icms_totals};
use crate::tax_is::{IsData, build_is_xml};
use crate::tax_issqn::{IssqnData, build_issqn_xml};
use crate::tax_pis_cofins_ipi::{
    CofinsData, IiData, IpiData, PisData, build_cofins_xml, build_ii_xml, build_ipi_xml,
    build_pis_xml,
};

// ── TaxCalculation ──────────────────────────────────────────────────────────

/// Tax data that can build an XML string fragment.
///
/// This trait is sealed — only types within this crate can implement it.
/// External crates can call methods on implementors but cannot add new ones.
pub trait TaxCalculation: Sealed {
    /// Build the XML string for this tax data.
    fn build_xml(&self) -> String;
}

impl Sealed for IcmsVariant {}
impl TaxCalculation for IcmsVariant {
    /// Build the ICMS XML string, delegating to [`build_icms_xml`].
    ///
    /// A throwaway [`crate::tax_icms::IcmsTotals`] accumulator is used since
    /// the trait cannot surface totals side-effects. Returns an empty string
    /// if the underlying call returns an error.
    fn build_xml(&self) -> String {
        let mut totals = create_icms_totals();
        build_icms_xml(self, &mut totals).unwrap_or_default()
    }
}

impl Sealed for PisData {}
impl TaxCalculation for PisData {
    /// Build the PIS XML string, delegating to [`build_pis_xml`].
    fn build_xml(&self) -> String {
        build_pis_xml(self)
    }
}

impl Sealed for CofinsData {}
impl TaxCalculation for CofinsData {
    /// Build the COFINS XML string, delegating to [`build_cofins_xml`].
    fn build_xml(&self) -> String {
        build_cofins_xml(self)
    }
}

impl Sealed for IpiData {}
impl TaxCalculation for IpiData {
    /// Build the IPI XML string, delegating to [`build_ipi_xml`].
    fn build_xml(&self) -> String {
        build_ipi_xml(self)
    }
}

impl Sealed for IiData {}
impl TaxCalculation for IiData {
    /// Build the II (import tax) XML string, delegating to [`build_ii_xml`].
    fn build_xml(&self) -> String {
        build_ii_xml(self)
    }
}

impl Sealed for IssqnData {}
impl TaxCalculation for IssqnData {
    /// Build the ISSQN XML string, delegating to [`build_issqn_xml`].
    fn build_xml(&self) -> String {
        build_issqn_xml(self)
    }
}

impl Sealed for IsData {}
impl TaxCalculation for IsData {
    /// Build the IS (IBS/CBS) XML string, delegating to [`build_is_xml`].
    fn build_xml(&self) -> String {
        build_is_xml(self)
    }
}

// ── XmlSerializable ─────────────────────────────────────────────────────────

/// Types that can serialize themselves to NF-e XML fragments.
///
/// This trait is sealed — only types within this crate can implement it.
pub trait XmlSerializable: Sealed {
    /// Serialize to an XML string fragment.
    fn to_xml(&self) -> String;
}

impl Sealed for TaxElement {}
impl XmlSerializable for TaxElement {
    /// Serialize the element to XML, delegating to [`serialize_tax_element`].
    fn to_xml(&self) -> String {
        serialize_tax_element(self)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::{Cents, Rate, Rate4};
    use crate::tax_element::TaxField;
    use crate::tax_icms::{IcmsCst, IcmsVariant};
    use crate::tax_is::IsData;
    use crate::tax_issqn::IssqnData;
    use crate::tax_pis_cofins_ipi::{CofinsData, IiData, IpiData, PisData};

    // -- IcmsVariant ----------------------------------------------------------

    #[test]
    fn icms_variant_trait_matches_free_fn() {
        let cst = IcmsCst::Cst00 {
            orig: "0".to_string(),
            mod_bc: "3".to_string(),
            v_bc: Cents(10000),
            p_icms: Rate(1200),
            v_icms: Cents(1200),
            p_fcp: None,
            v_fcp: None,
        };
        let variant = IcmsVariant::from(cst);

        let mut totals = create_icms_totals();
        let expected = build_icms_xml(&variant, &mut totals).unwrap();
        let got = variant.build_xml();
        assert_eq!(got, expected);
    }

    // -- PisData --------------------------------------------------------------

    #[test]
    fn pis_data_trait_matches_free_fn() {
        let data = PisData::new("01")
            .v_bc(Cents(10000))
            .p_pis(Rate4(16500))
            .v_pis(Cents(165));

        assert_eq!(data.build_xml(), build_pis_xml(&data));
    }

    // -- CofinsData -----------------------------------------------------------

    #[test]
    fn cofins_data_trait_matches_free_fn() {
        let data = CofinsData::new("01")
            .v_bc(Cents(10000))
            .p_cofins(Rate4(76000))
            .v_cofins(Cents(760));

        assert_eq!(data.build_xml(), build_cofins_xml(&data));
    }

    // -- IpiData --------------------------------------------------------------

    #[test]
    fn ipi_data_trait_matches_free_fn() {
        let data = IpiData::new("00", "999")
            .v_bc(Cents(10000))
            .p_ipi(Rate(500))
            .v_ipi(Cents(500));

        assert_eq!(data.build_xml(), build_ipi_xml(&data));
    }

    // -- IiData ---------------------------------------------------------------

    #[test]
    fn ii_data_trait_matches_free_fn() {
        let data = IiData::new(Cents(10000), Cents(200), Cents(1000), Cents(50));

        assert_eq!(data.build_xml(), build_ii_xml(&data));
    }

    // -- IssqnData ------------------------------------------------------------

    #[test]
    fn issqn_data_trait_matches_free_fn() {
        let data = IssqnData::new(10000, 500, 500, "4106902", "01.01");

        assert_eq!(data.build_xml(), build_issqn_xml(&data));
    }

    // -- IsData ---------------------------------------------------------------

    #[test]
    fn is_data_trait_matches_free_fn() {
        let data = IsData::new("00", "1234", "5.00")
            .v_bc_is("100.00")
            .p_is("5.0000");

        assert_eq!(data.build_xml(), build_is_xml(&data));
    }

    // -- TaxElement -----------------------------------------------------------

    #[test]
    fn tax_element_trait_matches_free_fn() {
        let element = TaxElement {
            outer_tag: Some("PIS".to_string()),
            outer_fields: vec![],
            variant_tag: "PISAliq".to_string(),
            fields: vec![
                TaxField::new("CST", "01"),
                TaxField::new("vBC", "100.00"),
                TaxField::new("pPIS", "1.6500"),
                TaxField::new("vPIS", "1.65"),
            ],
        };

        assert_eq!(element.to_xml(), serialize_tax_element(&element));
    }
}
