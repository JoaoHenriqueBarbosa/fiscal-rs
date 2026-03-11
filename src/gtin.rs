use crate::FiscalError;

/// Validate a GTIN-8/12/13/14 barcode number.
///
/// - Empty string and `"SEM GTIN"` are considered valid (exempt).
/// - Valid GTIN-8/12/13/14 with correct check digit returns `Ok(true)`.
/// - Non-numeric input or invalid check digit returns `Err`.
///
/// # Examples
///
/// ```
/// use fiscal::gtin::is_valid_gtin;
///
/// assert_eq!(is_valid_gtin(""), Ok(true));
/// assert_eq!(is_valid_gtin("SEM GTIN"), Ok(true));
/// assert!(is_valid_gtin("ABC").is_err());
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - The input contains non-numeric characters
/// - The length is not 8, 12, 13, or 14
/// - The check digit is invalid
pub fn is_valid_gtin(gtin: &str) -> Result<bool, FiscalError> {
    if gtin.is_empty() || gtin == "SEM GTIN" {
        return Ok(true);
    }

    if gtin.chars().any(|c| !c.is_ascii_digit()) {
        return Err(FiscalError::InvalidGtin(format!(
            "GTIN must contain only digits: \"{gtin}\" is not valid."
        )));
    }

    let len = gtin.len();
    if len != 8 && len != 12 && len != 13 && len != 14 {
        return Err(FiscalError::InvalidGtin(format!(
            "GTIN must be 8, 12, 13, or 14 digits. Got {len} digits."
        )));
    }

    let expected_dv = calculate_check_digit(gtin)?;
    let actual_dv = gtin.as_bytes()[len - 1] - b'0';

    if actual_dv != expected_dv {
        return Err(FiscalError::InvalidGtin(format!(
            "GTIN \"{gtin}\" has an invalid check digit."
        )));
    }

    Ok(true)
}

/// Calculate the GTIN check digit using the standard algorithm.
///
/// Works for GTIN-8, GTIN-12, GTIN-13, and GTIN-14.
/// The input must include the check digit position (full barcode).
///
/// # Errors
///
/// Returns `Err` if the input contains non-digit characters.
pub fn calculate_check_digit(gtin: &str) -> Result<u8, FiscalError> {
    let len = gtin.len();
    if len < 2 {
        return Err(FiscalError::InvalidGtin(
            "GTIN must have at least 2 digits".to_string(),
        ));
    }

    // Pad the digits (excluding last) to 15 positions, left-padded with zeros
    let without_check = &gtin[..len - 1];
    let padded = format!("{:0>15}", without_check);

    let mut total: u32 = 0;
    for (pos, ch) in padded.bytes().enumerate() {
        let val = (ch - b'0') as u32;
        // Alternating multiplier: positions 0,2,4... get x1; positions 1,3,5... get x3
        let multiplier = ((pos + 1) % 2) * 2 + 1;
        total += multiplier as u32 * val;
    }

    let dv = (10 - (total % 10)) % 10;
    Ok(dv as u8)
}
