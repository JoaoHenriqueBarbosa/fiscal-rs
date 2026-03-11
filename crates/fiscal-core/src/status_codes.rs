/// SEFAZ status code constants (cStat) used across the fiscal module.
///
/// Reference: Manual de Orientacao do Contribuinte (MOC) v7.0+
pub mod sefaz_status {
    /// 100 -- Authorized (Autorizado o uso da NF-e)
    pub const AUTHORIZED: &str = "100";
    /// 101 -- Cancelled
    pub const CANCELLED: &str = "101";
    /// 102 -- Number voided (Inutilizacao de numero homologada)
    pub const VOIDED: &str = "102";
    /// 107 -- Service running (Servico em Operacao)
    pub const SERVICE_RUNNING: &str = "107";
    /// 110 -- Usage denied (Uso Denegado)
    pub const DENIED: &str = "110";
    /// 135 -- Event registered and linked to NF-e
    pub const EVENT_REGISTERED: &str = "135";
    /// 136 -- Event registered but not linked to NF-e
    pub const EVENT_ALREADY_REGISTERED: &str = "136";
    /// 150 -- Authorized (late / fora do prazo)
    pub const AUTHORIZED_LATE: &str = "150";
    /// 155 -- Already cancelled (late)
    pub const ALREADY_CANCELLED: &str = "155";
    /// 204 -- Duplicate
    pub const DUPLICATE: &str = "204";
    /// 205 -- Denied in SEFAZ database
    pub const DENIED_IN_DATABASE: &str = "205";
    /// 301 -- Denied: issuer fiscal irregularity
    pub const DENIED_ISSUER_IRREGULAR: &str = "301";
    /// 302 -- Denied: recipient fiscal irregularity
    pub const DENIED_RECIPIENT_IRREGULAR: &str = "302";
    /// 303 -- Denied: recipient not enabled in UF
    pub const DENIED_RECIPIENT_NOT_ENABLED: &str = "303";
}

/// Valid status codes for protocol attachment.
///
/// These statuses indicate the NF-e was processed (authorized or denied)
/// and the protocol can be attached to form `nfeProc`.
pub const VALID_PROTOCOL_STATUSES: &[&str] = &[
    "100", // Authorized
    "150", // Authorized (late)
    "110", // Usage denied
    "205", // Denied in database
    "301", // Denied: issuer irregular
    "302", // Denied: recipient irregular
    "303", // Denied: recipient not enabled
];

/// Valid status codes for event attachment.
///
/// These statuses indicate the event was accepted and can be attached
/// to form `procEventoNFe`.
pub const VALID_EVENT_STATUSES: &[&str] = &[
    "135", // Event registered
    "136", // Event registered (unlinked)
    "155", // Already cancelled (late)
];
