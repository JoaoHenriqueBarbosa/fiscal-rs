use fiscal::format_utils::*;
use fiscal::state_codes::*;
use fiscal::tax_element::*;
use fiscal::tax_icms::*;
use fiscal::xml_utils::*;
use proptest::prelude::*;
use fiscal::newtypes::{Cents};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn reasonable_cents() -> impl Strategy<Value = i64> {
    -999_999_999i64..=999_999_999i64
}

fn decimal_places() -> impl Strategy<Value = usize> {
    2..=10usize
}

fn safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}"
}

// ---------------------------------------------------------------------------
// format_utils -- monetary formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn format_cents_has_one_dot_and_n_decimals(cents in reasonable_cents(), dp in decimal_places()) {
        let s = format_cents(cents, dp);
        let parts: Vec<&str> = s.split('.').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[1].len(), dp);
    }

    #[test]
    fn format_cents_2_has_one_dot_and_2_decimals(cents in reasonable_cents()) {
        let s = format_cents_2(cents);
        let parts: Vec<&str> = s.split('.').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[1].len(), 2);
    }

    #[test]
    fn format_cents_zero_is_zero_dot_zeroes(dp in decimal_places()) {
        let s = format_cents(0, dp);
        let expected_frac: String = "0".repeat(dp);
        let expected = format!("0.{}", expected_frac);
        prop_assert_eq!(s, expected);
    }

    #[test]
    fn format_rate_has_n_decimal_places(hundredths in reasonable_cents(), dp in decimal_places()) {
        let s = format_rate(hundredths, dp);
        let parts: Vec<&str> = s.split('.').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[1].len(), dp);
    }

    #[test]
    fn format_rate_4_has_4_decimal_places(hundredths in reasonable_cents()) {
        let s = format_rate_4(hundredths);
        let parts: Vec<&str> = s.split('.').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[1].len(), 4);
    }

    #[test]
    fn format_rate4_has_4_decimal_places(value in reasonable_cents()) {
        let s = format_rate4(value);
        let parts: Vec<&str> = s.split('.').collect();
        prop_assert_eq!(parts.len(), 2);
        prop_assert_eq!(parts[1].len(), 4);
    }

    #[test]
    fn format_cents_or_zero_none_produces_zero(dp in decimal_places()) {
        let s = format_cents_or_zero(None, dp);
        let expected_frac: String = "0".repeat(dp);
        let expected = format!("0.{}", expected_frac);
        prop_assert_eq!(s, expected);
    }

    #[test]
    fn format_cents_or_zero_some_matches_format_cents(cents in reasonable_cents(), dp in decimal_places()) {
        let from_option = format_cents_or_zero(Some(cents), dp);
        let direct = format_cents(cents, dp);
        prop_assert_eq!(from_option, direct);
    }

    #[test]
    fn format_rate4_or_zero_some_matches_format_rate4(value in reasonable_cents()) {
        let from_option = format_rate4_or_zero(Some(value));
        let direct = format_rate4(value);
        prop_assert_eq!(from_option, direct);
    }

    #[test]
    fn format_cents_or_none_none_is_none(dp in decimal_places()) {
        prop_assert!(format_cents_or_none(None, dp).is_none());
    }

    #[test]
    fn format_cents_or_none_some_matches_format_cents(cents in reasonable_cents(), dp in decimal_places()) {
        let result = format_cents_or_none(Some(cents), dp);
        let direct = format_cents(cents, dp);
        prop_assert_eq!(result.unwrap(), direct);
    }

    // For non-negative cents, the last 2 decimal digits match cents % 100
    #[test]
    fn format_cents_2_decimal_part_matches_mod_100(cents in 0..1_000_000_000i64) {
        let s = format_cents_2(cents);
        let frac_str = s.split('.').nth(1).unwrap();
        let frac: i64 = frac_str.parse().unwrap();
        let expected = cents % 100;
        prop_assert_eq!(frac, expected);
    }

    // format_cents string length is consistent for non-negative values
    #[test]
    fn format_cents_len_consistent(cents in 0..1_000_000_000i64, dp in decimal_places()) {
        let s = format_cents(cents, dp);
        let integer_part = cents / 100;
        let expected_int_digits = if integer_part == 0 { 1 } else { format!("{}", integer_part).len() };
        let expected_len = expected_int_digits + 1 + dp;
        prop_assert_eq!(s.len(), expected_len);
    }
}

// Tests that don't need proptest parameters
#[test]
fn format_rate4_or_zero_none_is_zero() {
    assert_eq!(format_rate4_or_zero(None), "0.0000");
}

#[test]
fn format_cents_or_zero_none_2dp_is_zero() {
    assert_eq!(format_cents_or_zero(None, 2), "0.00");
}

#[test]
fn format_cents_zero_2dp() {
    assert_eq!(format_cents(0, 2), "0.00");
}

// ---------------------------------------------------------------------------
// xml_utils -- XML building
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn escape_xml_never_contains_raw_special_chars(input in "\\PC{1,100}") {
        let escaped = escape_xml(&input);
        for (i, ch) in escaped.char_indices() {
            match ch {
                '<' | '>' | '"' | '\'' => {
                    prop_assert!(false, "found raw special char at position {}", i);
                }
                '&' => {
                    let rest = &escaped[i..];
                    prop_assert!(
                        rest.starts_with("&amp;")
                            || rest.starts_with("&lt;")
                            || rest.starts_with("&gt;")
                            || rest.starts_with("&quot;")
                            || rest.starts_with("&apos;"),
                        "found '&' not part of known entity at position {}", i
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn escape_xml_safe_string_unchanged(input in safe_string()) {
        let escaped = escape_xml(&input);
        prop_assert_eq!(escaped, input);
    }

    #[test]
    fn tag_starts_with_lt_ends_with_gt(
        name in "[a-zA-Z][a-zA-Z0-9]{0,15}",
        text in safe_string(),
    ) {
        let output = tag(&name, &[], TagContent::Text(&text));
        prop_assert!(output.starts_with('<'), "output should start with '<'");
        prop_assert!(output.ends_with('>'), "output should end with '>'");
    }

    #[test]
    fn tag_with_text_contains_escaped_text(
        name in "[a-zA-Z][a-zA-Z0-9]{0,15}",
        text in safe_string(),
    ) {
        let output = tag(&name, &[], TagContent::Text(&text));
        let escaped_text = escape_xml(&text);
        prop_assert!(output.contains(&escaped_text),
            "tag output should contain escaped text");
    }

    #[test]
    fn tag_none_is_empty_element(name in "[a-zA-Z][a-zA-Z0-9]{0,15}") {
        let output = tag(&name, &[], TagContent::None);
        let expected = format!("<{}></{}>", name, name);
        prop_assert_eq!(output, expected);
    }

    #[test]
    fn tag_extract_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9]{0,15}",
        text in safe_string(),
    ) {
        let xml = tag(&name, &[], TagContent::Text(&text));
        let extracted = extract_xml_tag_value(&xml, &name);
        prop_assert!(extracted.is_some(), "should find tag in xml");
        let escaped_text = escape_xml(&text);
        prop_assert_eq!(extracted.unwrap(), escaped_text);
    }

    #[test]
    fn extract_xml_tag_value_missing_tag_does_not_panic(
        xml in safe_string(),
        tag_name in "[a-zA-Z]{3,8}",
    ) {
        // Should not panic on arbitrary input
        let _ = extract_xml_tag_value(&xml, &tag_name);
    }
}

// ---------------------------------------------------------------------------
// tax_element -- serialization
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serialize_tax_element_no_outer_tag(
        variant in "[a-zA-Z]{3,10}",
        field_name in "[a-zA-Z]{2,8}",
        field_value in safe_string(),
    ) {
        let element = TaxElement {
            outer_tag: None,
            outer_fields: vec![],
            variant_tag: variant.clone(),
            fields: vec![TaxField::new(&field_name, &field_value)],
        };
        let xml = serialize_tax_element(&element);
        let open = format!("<{}>", variant);
        let close = format!("</{}>", variant);
        prop_assert!(xml.starts_with(&open), "should start with variant tag");
        prop_assert!(xml.ends_with(&close), "should end with variant closing tag");
    }

    #[test]
    fn serialize_tax_element_with_outer_tag(
        outer in "[a-zA-Z]{3,10}",
        variant in "[a-zA-Z]{3,10}",
        field_name in "[a-zA-Z]{2,8}",
        field_value in safe_string(),
    ) {
        let element = TaxElement {
            outer_tag: Some(outer.clone()),
            outer_fields: vec![],
            variant_tag: variant.clone(),
            fields: vec![TaxField::new(&field_name, &field_value)],
        };
        let xml = serialize_tax_element(&element);
        let outer_open = format!("<{}>", outer);
        let outer_close = format!("</{}>", outer);
        let variant_open = format!("<{}>", variant);
        let variant_close = format!("</{}>", variant);
        prop_assert!(xml.starts_with(&outer_open), "should start with outer tag");
        prop_assert!(xml.ends_with(&outer_close), "should end with outer closing tag");
        prop_assert!(xml.contains(&variant_open), "should contain variant opening tag");
        prop_assert!(xml.contains(&variant_close), "should contain variant closing tag");
    }

    #[test]
    fn filter_fields_removes_nones_keeps_somes(
        count_some in 0..20usize,
        count_none in 0..20usize,
    ) {
        let mut fields: Vec<Option<TaxField>> = Vec::new();
        for i in 0..count_some {
            fields.push(Some(TaxField::new(format!("f{}", i), "v")));
        }
        for _ in 0..count_none {
            fields.push(None);
        }
        let result = filter_fields(fields);
        prop_assert_eq!(result.len(), count_some);
    }

    #[test]
    fn tax_field_serializes_to_xml_pattern(
        name in "[a-zA-Z]{2,10}",
        value in safe_string(),
    ) {
        let element = TaxElement {
            outer_tag: None,
            outer_fields: vec![],
            variant_tag: "Wrapper".to_string(),
            fields: vec![TaxField::new(&name, &value)],
        };
        let xml = serialize_tax_element(&element);
        let expected_field = format!("<{}>{}</{}>", name, value, name);
        prop_assert!(xml.contains(&expected_field),
            "xml should contain field pattern");
    }

    #[test]
    fn optional_field_none_returns_none(name in "[a-zA-Z]{2,10}") {
        prop_assert!(optional_field(&name, None).is_none());
    }

    #[test]
    fn optional_field_some_returns_tax_field(
        name in "[a-zA-Z]{2,10}",
        value in safe_string(),
    ) {
        let result = optional_field(&name, Some(&value));
        prop_assert!(result.is_some());
        let field = result.unwrap();
        prop_assert_eq!(&field.name, &name);
        prop_assert_eq!(&field.value, &value);
    }

    #[test]
    fn required_field_none_returns_err(name in "[a-zA-Z]{2,10}") {
        let result = required_field(&name, None);
        prop_assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        prop_assert!(msg.contains(&name), "error should mention field name: {}", msg);
    }

    #[test]
    fn required_field_some_returns_ok(
        name in "[a-zA-Z]{2,10}",
        value in safe_string(),
    ) {
        let result = required_field(&name, Some(&value));
        prop_assert!(result.is_ok());
        let field = result.unwrap();
        prop_assert_eq!(&field.name, &name);
        prop_assert_eq!(&field.value, &value);
    }
}

// ---------------------------------------------------------------------------
// state_codes
// ---------------------------------------------------------------------------

const ALL_UFS: [&str; 27] = [
    "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MT", "MS",
    "MG", "PA", "PB", "PR", "PE", "PI", "RJ", "RN", "RS", "RO", "RR", "SC",
    "SP", "SE", "TO",
];

#[test]
fn state_ibge_codes_has_all_27_ufs() {
    assert_eq!(STATE_IBGE_CODES.len(), 27);
    for uf in &ALL_UFS {
        assert!(
            STATE_IBGE_CODES.contains_key(uf),
            "STATE_IBGE_CODES missing UF: {}",
            uf,
        );
    }
}

#[test]
fn get_state_code_and_get_state_by_code_are_inverses() {
    for uf in &ALL_UFS {
        let code = get_state_code(uf).unwrap_or_else(|_| panic!("get_state_code failed for {}", uf));
        let back = get_state_by_code(code).unwrap_or_else(|_| panic!("get_state_by_code failed for {}", code));
        assert_eq!(*uf, back, "round-trip failed: {} -> {} -> {}", uf, code, back);
    }
}

#[test]
fn ibge_to_uf_has_all_27_codes() {
    assert_eq!(IBGE_TO_UF.len(), 27);
}

proptest! {
    #[test]
    fn unknown_uf_returns_err(uf in "[A-Z]{2}") {
        if !ALL_UFS.contains(&uf.as_str()) {
            prop_assert!(get_state_code(&uf).is_err());
        }
    }

    #[test]
    fn unknown_ibge_code_returns_err(code in "[0-9]{2}") {
        let result = get_state_by_code(&code);
        if IBGE_TO_UF.contains_key(code.as_str()) {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }
}

// ---------------------------------------------------------------------------
// tax_icms -- totals
// ---------------------------------------------------------------------------

#[test]
fn create_icms_totals_is_all_zero() {
    let t = create_icms_totals();
    assert_eq!(t.v_bc, Cents(0));
    assert_eq!(t.v_icms, Cents(0));
    assert_eq!(t.v_icms_deson, Cents(0));
    assert_eq!(t.v_bc_st, Cents(0));
    assert_eq!(t.v_st, Cents(0));
    assert_eq!(t.v_fcp, Cents(0));
    assert_eq!(t.v_fcp_st, Cents(0));
    assert_eq!(t.v_fcp_st_ret, Cents(0));
    assert_eq!(t.v_fcp_uf_dest, Cents(0));
    assert_eq!(t.v_icms_uf_dest, Cents(0));
    assert_eq!(t.v_icms_uf_remet, Cents(0));
    assert_eq!(t.q_bc_mono, 0);
    assert_eq!(t.v_icms_mono, Cents(0));
    assert_eq!(t.q_bc_mono_reten, 0);
    assert_eq!(t.v_icms_mono_reten, Cents(0));
    assert_eq!(t.q_bc_mono_ret, 0);
    assert_eq!(t.v_icms_mono_ret, Cents(0));
}

#[test]
fn merge_icms_totals_double_zero_is_still_zero() {
    let mut target = create_icms_totals();
    let source = create_icms_totals();
    merge_icms_totals(&mut target, &source);
    assert_eq!(target, create_icms_totals());
}

proptest! {
    #[test]
    fn merge_icms_totals_with_zero_is_identity(
        v_bc in reasonable_cents(),
        v_icms in reasonable_cents(),
        v_icms_deson in reasonable_cents(),
        v_bc_st in reasonable_cents(),
        v_st in reasonable_cents(),
        v_fcp in reasonable_cents(),
        v_fcp_st in reasonable_cents(),
        v_fcp_st_ret in reasonable_cents(),
        v_fcp_uf_dest in reasonable_cents(),
        v_icms_uf_dest in reasonable_cents(),
    ) {
        let source = IcmsTotals {
            v_bc: Cents(v_bc), v_icms: Cents(v_icms), v_icms_deson: Cents(v_icms_deson),
            v_bc_st: Cents(v_bc_st), v_st: Cents(v_st), v_fcp: Cents(v_fcp), v_fcp_st: Cents(v_fcp_st),
            v_fcp_st_ret: Cents(v_fcp_st_ret), v_fcp_uf_dest: Cents(v_fcp_uf_dest),
            v_icms_uf_dest: Cents(v_icms_uf_dest),
            ..IcmsTotals::default()
        };
        let mut target = create_icms_totals();
        merge_icms_totals(&mut target, &source);
        prop_assert_eq!(target, source);
    }

    #[test]
    fn merge_icms_totals_is_associative(
        a_bc in reasonable_cents(),
        a_icms in reasonable_cents(),
        a_st in reasonable_cents(),
        b_bc in reasonable_cents(),
        b_icms in reasonable_cents(),
        b_st in reasonable_cents(),
        c_bc in reasonable_cents(),
        c_icms in reasonable_cents(),
        c_st in reasonable_cents(),
    ) {
        let a = IcmsTotals { v_bc: Cents(a_bc), v_icms: Cents(a_icms), v_st: Cents(a_st), ..IcmsTotals::new() };
        let b = IcmsTotals { v_bc: Cents(b_bc), v_icms: Cents(b_icms), v_st: Cents(b_st), ..IcmsTotals::new() };
        let c = IcmsTotals { v_bc: Cents(c_bc), v_icms: Cents(c_icms), v_st: Cents(c_st), ..IcmsTotals::new() };

        // (a + b) + c
        let mut left = a.clone();
        merge_icms_totals(&mut left, &b);
        merge_icms_totals(&mut left, &c);

        // a + (b + c)
        let mut bc = b.clone();
        merge_icms_totals(&mut bc, &c);
        let mut right = a.clone();
        merge_icms_totals(&mut right, &bc);

        prop_assert_eq!(left, right);
    }

    #[test]
    fn merge_icms_totals_is_commutative(
        a_bc in reasonable_cents(),
        a_icms in reasonable_cents(),
        a_deson in reasonable_cents(),
        a_fcp in reasonable_cents(),
        b_bc in reasonable_cents(),
        b_icms in reasonable_cents(),
        b_deson in reasonable_cents(),
        b_fcp in reasonable_cents(),
    ) {
        let a = IcmsTotals {
            v_bc: Cents(a_bc), v_icms: Cents(a_icms), v_icms_deson: Cents(a_deson), v_fcp: Cents(a_fcp),
            ..Default::default()
        };
        let b = IcmsTotals {
            v_bc: Cents(b_bc), v_icms: Cents(b_icms), v_icms_deson: Cents(b_deson), v_fcp: Cents(b_fcp),
            ..Default::default()
        };

        // a + b
        let mut ab = a.clone();
        merge_icms_totals(&mut ab, &b);

        // b + a
        let mut ba = b.clone();
        merge_icms_totals(&mut ba, &a);

        prop_assert_eq!(ab, ba);
    }
}
