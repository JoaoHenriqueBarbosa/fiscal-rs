use crate::newtypes::Cents;

/// Billing section (`<cobr>`) with optional invoice header and installments.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct BillingData {
    /// Billing invoice summary (`<fat>`).
    pub invoice: Option<BillingInvoice>,
    /// Individual billing installments (`<dup>`).
    pub installments: Option<Vec<Installment>>,
}

impl BillingData {
    /// Create a new empty `BillingData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the billing invoice header.
    pub fn invoice(mut self, inv: BillingInvoice) -> Self {
        self.invoice = Some(inv);
        self
    }

    /// Set the installments.
    pub fn installments(mut self, inst: Vec<Installment>) -> Self {
        self.installments = Some(inst);
        self
    }
}

/// Billing invoice summary (`<fat>`) with original, discount, and net values.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BillingInvoice {
    /// Invoice / bill number (`nFat`).
    pub number: String,
    /// Original invoice value before discounts (`vOrig`).
    pub original_value: Cents,
    /// Discount amount (`vDesc`). Optional.
    pub discount_value: Option<Cents>,
    /// Net invoice value after discounts (`vLiq`).
    pub net_value: Cents,
}

impl BillingInvoice {
    /// Create a new `BillingInvoice` with required fields.
    pub fn new(number: impl Into<String>, original_value: Cents, net_value: Cents) -> Self {
        Self {
            number: number.into(),
            original_value,
            discount_value: None,
            net_value,
        }
    }

    /// Set the discount value.
    pub fn discount_value(mut self, v: Cents) -> Self {
        self.discount_value = Some(v);
        self
    }
}

/// A single billing installment (`<dup>`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Installment {
    /// Installment number (`nDup`), e.g. `"001"`.
    pub number: String,
    /// Due date (`dVenc`) in `YYYY-MM-DD` format.
    pub due_date: String,
    /// Instalment amount (`vDup`).
    pub value: Cents,
}

impl Installment {
    /// Create a new `Installment`.
    pub fn new(number: impl Into<String>, due_date: impl Into<String>, value: Cents) -> Self {
        Self {
            number: number.into(),
            due_date: due_date.into(),
            value,
        }
    }
}
