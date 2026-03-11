/// Format cents integer to decimal string. E.g. 1050 -> "10.50"
pub fn format_cents(cents: i64, decimal_places: usize) -> String {
    let divisor = 100.0_f64;
    let value = cents as f64 / divisor;
    format!("{value:.decimal_places$}")
}

/// Format cents with default 2 decimal places
pub fn format_cents_2(cents: i64) -> String {
    format_cents(cents, 2)
}

/// Format cents with 10 decimal places (for unit prices)
pub fn format_cents_10(cents: i64) -> String {
    format_cents(cents, 10)
}

/// Format a number with N decimal places
pub fn format_decimal(value: f64, decimal_places: usize) -> String {
    format!("{value:.decimal_places$}")
}

/// Format rate stored as hundredths to decimal string. E.g. 1800 -> "18.0000"
pub fn format_rate(hundredths: i64, decimal_places: usize) -> String {
    let value = hundredths as f64 / 100.0;
    format!("{value:.decimal_places$}")
}

/// Format rate with default 4 decimal places
pub fn format_rate_4(hundredths: i64) -> String {
    format_rate(hundredths, 4)
}

/// Format rate stored as value * 10000 to 4-decimal string. E.g. 16500 -> "1.6500"
pub fn format_rate4(value: i64) -> String {
    let v = value as f64 / 10000.0;
    format!("{v:.4}")
}

/// Format cents to decimal string, returning None for None input
pub fn format_cents_or_none(cents: Option<i64>, decimal_places: usize) -> Option<String> {
    cents.map(|c| format_cents(c, decimal_places))
}

/// Format cents to decimal string, defaulting to "0.00" for None
pub fn format_cents_or_zero(cents: Option<i64>, decimal_places: usize) -> String {
    match cents {
        Some(c) => format_cents(c, decimal_places),
        None => format_cents(0, decimal_places),
    }
}

/// Format rate4 (value * 10000) to 4-decimal string, defaulting to "0.0000" for None
pub fn format_rate4_or_zero(value: Option<i64>) -> String {
    match value {
        Some(v) => format_rate4(v),
        None => "0.0000".to_string(),
    }
}
