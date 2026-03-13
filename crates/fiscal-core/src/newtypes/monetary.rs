//! Monetary amount and tax-rate newtypes: [`Cents`], [`Rate`], [`Rate4`].

use std::fmt;
use std::ops::{Add, AddAssign};

// ── Monetary amount ─────────────────────────────────────────────────────────

/// Monetary amount in cents (R$ 10.50 = `Cents(1050)`).
///
/// `Display` renders with 2 decimal places: `"10.50"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cents(pub i64);

impl From<i64> for Cents {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Cents> for i64 {
    #[inline]
    fn from(c: Cents) -> Self {
        c.0
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 100;
        let frac = abs % 100;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:02}")
        } else {
            write!(f, "{whole}.{frac:02}")
        }
    }
}

impl Add for Cents {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Cents {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

// ── Tax rate (hundredths) ───────────────────────────────────────────────────

/// Tax rate in hundredths of a percent (18% = `Rate(1800)`).
///
/// `Display` renders with 4 decimal places: `"18.0000"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rate(pub i64);

impl From<i64> for Rate {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Rate> for i64 {
    #[inline]
    fn from(r: Rate) -> Self {
        r.0
    }
}

impl fmt::Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 100;
        let frac = abs % 100;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:02}00")
        } else {
            write!(f, "{whole}.{frac:02}00")
        }
    }
}

impl Add for Rate {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Rate {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

// ── PIS/COFINS rate (×10 000) ───────────────────────────────────────────────

/// PIS/COFINS rate scaled by 10 000 (1.65% = `Rate4(16500)`).
///
/// `Display` renders with 4 decimal places: `"1.6500"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rate4(pub i64);

impl From<i64> for Rate4 {
    #[inline]
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl From<Rate4> for i64 {
    #[inline]
    fn from(r: Rate4) -> Self {
        r.0
    }
}

impl fmt::Display for Rate4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.0.unsigned_abs();
        let whole = abs / 10_000;
        let frac = abs % 10_000;
        if self.0 < 0 {
            write!(f, "-{whole}.{frac:04}")
        } else {
            write!(f, "{whole}.{frac:04}")
        }
    }
}

impl Add for Rate4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Rate4 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- From<i64> for Cents and From<Cents> for i64 -------------------------

    #[test]
    fn cents_from_i64() {
        let c = Cents::from(42_i64);
        assert_eq!(c, Cents(42));
    }

    #[test]
    fn i64_from_cents() {
        let v: i64 = Cents(999).into();
        assert_eq!(v, 999);
    }

    // -- From<i64> for Rate and From<Rate> for i64 ---------------------------

    #[test]
    fn rate_from_i64() {
        let r = Rate::from(1800_i64);
        assert_eq!(r, Rate(1800));
    }

    #[test]
    fn i64_from_rate() {
        let v: i64 = Rate(1800).into();
        assert_eq!(v, 1800);
    }

    // -- From<i64> for Rate4 and From<Rate4> for i64 -------------------------

    #[test]
    fn rate4_from_i64() {
        let r = Rate4::from(16500_i64);
        assert_eq!(r, Rate4(16500));
    }

    #[test]
    fn i64_from_rate4() {
        let v: i64 = Rate4(16500).into();
        assert_eq!(v, 16500);
    }

    // -- Rate arithmetic -------------------------------------------------------

    #[test]
    fn rate_add_assign() {
        let mut r = Rate(100);
        r += Rate(50);
        assert_eq!(r, Rate(150));
    }

    // -- Rate4 arithmetic -------------------------------------------------------

    #[test]
    fn rate4_add_assign() {
        let mut r = Rate4(10000);
        r += Rate4(5000);
        assert_eq!(r, Rate4(15000));
    }

    // -- Cents -----------------------------------------------------------------

    #[test]
    fn cents_display() {
        assert_eq!(Cents(1050).to_string(), "10.50");
        assert_eq!(Cents(0).to_string(), "0.00");
        assert_eq!(Cents(1).to_string(), "0.01");
        assert_eq!(Cents(-350).to_string(), "-3.50");
    }

    #[test]
    fn cents_add() {
        assert_eq!(Cents(100) + Cents(250), Cents(350));
        let mut c = Cents(500);
        c += Cents(100);
        assert_eq!(c, Cents(600));
    }

    #[test]
    fn cents_conversion() {
        let c: Cents = 1050_i64.into();
        assert_eq!(c, Cents(1050));
        let v: i64 = c.into();
        assert_eq!(v, 1050);
    }

    // -- Rate -----------------------------------------------------------------

    #[test]
    fn rate_display() {
        assert_eq!(Rate(1800).to_string(), "18.0000");
        assert_eq!(Rate(0).to_string(), "0.0000");
        assert_eq!(Rate(750).to_string(), "7.5000");
        assert_eq!(Rate(-1800).to_string(), "-18.0000");
    }

    #[test]
    fn rate_add() {
        assert_eq!(Rate(1000) + Rate(800), Rate(1800));
        let mut r = Rate(500);
        r += Rate(300);
        assert_eq!(r, Rate(800));
    }

    // -- Rate4 ----------------------------------------------------------------

    #[test]
    fn rate4_display() {
        assert_eq!(Rate4(16500).to_string(), "1.6500");
        assert_eq!(Rate4(0).to_string(), "0.0000");
        assert_eq!(Rate4(76000).to_string(), "7.6000");
        assert_eq!(Rate4(-16500).to_string(), "-1.6500");
    }

    #[test]
    fn rate4_add() {
        assert_eq!(Rate4(16500) + Rate4(7600), Rate4(24100));
        let mut r = Rate4(10000);
        r += Rate4(5000);
        assert_eq!(r, Rate4(15000));
    }

    // -- Explicit From/Into round-trip tests for coverage on #[inline] impls --

    #[test]
    fn cents_from_into_roundtrip() {
        let original: i64 = 42;
        let c: Cents = original.into();
        let back: i64 = c.into();
        assert_eq!(original, back);
    }

    #[test]
    fn rate_from_into_roundtrip() {
        let original: i64 = 1800;
        let r: Rate = original.into();
        let back: i64 = r.into();
        assert_eq!(original, back);
    }

    #[test]
    fn rate4_from_into_roundtrip() {
        let original: i64 = 16500;
        let r: Rate4 = original.into();
        let back: i64 = r.into();
        assert_eq!(original, back);
    }
}
