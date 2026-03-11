//! Formatting helpers for monetary amounts, rates, and decimal numbers.
//!
//! All functions accept raw integer representations — cents for monetary values
//! and scaled integers for rates — and return formatted `String`s suitable for
//! insertion into NF-e XML elements.

/// Format a cents integer to a decimal string with the given number of decimal places.
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_cents;
/// assert_eq!(format_cents(1050, 2), "10.50");
/// assert_eq!(format_cents(100000, 10), "1000.0000000000");
/// ```
pub fn format_cents(cents: i64, decimal_places: usize) -> String {
    let divisor = 100.0_f64;
    let value = cents as f64 / divisor;
    format!("{value:.decimal_places$}")
}

/// Format a cents integer to a decimal string with 2 decimal places.
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_cents_2;
/// assert_eq!(format_cents_2(1050), "10.50");
/// assert_eq!(format_cents_2(0), "0.00");
/// ```
pub fn format_cents_2(cents: i64) -> String {
    format_cents(cents, 2)
}

/// Format a cents integer to a decimal string with 10 decimal places (for unit prices).
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_cents_10;
/// assert_eq!(format_cents_10(100000), "1000.0000000000");
/// ```
pub fn format_cents_10(cents: i64) -> String {
    format_cents(cents, 10)
}

/// Format a floating-point number with `decimal_places` decimal places.
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_decimal;
/// assert_eq!(format_decimal(3.14159, 2), "3.14");
/// ```
pub fn format_decimal(value: f64, decimal_places: usize) -> String {
    format!("{value:.decimal_places$}")
}

/// Format a rate stored as hundredths of a percent to a decimal string.
///
/// For example, `1800` (= 18%) with 4 decimal places → `"18.0000"`.
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_rate;
/// assert_eq!(format_rate(1800, 4), "18.0000");
/// assert_eq!(format_rate(750, 2), "7.50");
/// ```
pub fn format_rate(hundredths: i64, decimal_places: usize) -> String {
    let value = hundredths as f64 / 100.0;
    format!("{value:.decimal_places$}")
}

/// Format a rate (stored as hundredths) with 4 decimal places.
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_rate_4;
/// assert_eq!(format_rate_4(1800), "18.0000");
/// ```
pub fn format_rate_4(hundredths: i64) -> String {
    format_rate(hundredths, 4)
}

/// Format a PIS/COFINS rate stored as `value × 10 000` to a 4-decimal string.
///
/// For example, `16500` (= 1.65%) → `"1.6500"`.
///
/// # Examples
///
/// ```
/// use fiscal_core::format_utils::format_rate4;
/// assert_eq!(format_rate4(16500), "1.6500");
/// ```
pub fn format_rate4(value: i64) -> String {
    let v = value as f64 / 10000.0;
    format!("{v:.4}")
}

/// Format an optional cents value to a decimal string, returning `None` when the
/// input is `None`.
pub fn format_cents_or_none(cents: Option<i64>, decimal_places: usize) -> Option<String> {
    cents.map(|c| format_cents(c, decimal_places))
}

/// Format an optional cents value to a decimal string, defaulting to `"0.00"` (or
/// `"0.` + `n` zeros`"`) when the input is `None`.
pub fn format_cents_or_zero(cents: Option<i64>, decimal_places: usize) -> String {
    match cents {
        Some(c) => format_cents(c, decimal_places),
        None => format_cents(0, decimal_places),
    }
}

/// Format an optional `rate4` value (scaled by 10 000) to a 4-decimal string,
/// defaulting to `"0.0000"` when the input is `None`.
pub fn format_rate4_or_zero(value: Option<i64>) -> String {
    match value {
        Some(v) => format_rate4(v),
        None => "0.0000".to_string(),
    }
}
