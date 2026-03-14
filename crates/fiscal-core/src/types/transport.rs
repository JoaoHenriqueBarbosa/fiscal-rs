use crate::newtypes::{Cents, IbgeCode, Rate};
use serde::{Deserialize, Serialize};

/// Transport section (`<transp>`) data for an NF-e document.
///
/// The freight mode is required; all other fields are optional.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct TransportData {
    /// Freight modality code (`modFrete`): `"0"` (issuer) through `"9"` (no freight).
    pub freight_mode: String,
    /// Carrier identification (transportadora).
    pub carrier: Option<CarrierData>,
    /// Main transport vehicle.
    pub vehicle: Option<VehicleData>,
    /// Trailer vehicles (reboque).
    pub trailers: Option<Vec<VehicleData>>,
    /// List of transported volumes (`vol`).
    pub volumes: Option<Vec<VolumeData>>,
    /// ICMS retained on transport services (`retTransp`).
    pub retained_icms: Option<RetainedIcmsTransp>,
    /// Rail car number (`vagao`). Optional — mutually exclusive with `vehicle`/`trailers`.
    pub vagao: Option<String>,
    /// Barge / ferry identification (`balsa`). Optional — mutually exclusive with `vehicle`/`trailers`/`vagao`.
    pub balsa: Option<String>,
}

impl TransportData {
    /// Create a new `TransportData` with the required freight mode.
    pub fn new(freight_mode: impl Into<String>) -> Self {
        Self {
            freight_mode: freight_mode.into(),
            ..Default::default()
        }
    }

    /// Set the carrier data.
    pub fn carrier(mut self, carrier: CarrierData) -> Self {
        self.carrier = Some(carrier);
        self
    }

    /// Set the vehicle data.
    pub fn vehicle(mut self, vehicle: VehicleData) -> Self {
        self.vehicle = Some(vehicle);
        self
    }

    /// Set the trailers.
    pub fn trailers(mut self, trailers: Vec<VehicleData>) -> Self {
        self.trailers = Some(trailers);
        self
    }

    /// Set the volumes.
    pub fn volumes(mut self, volumes: Vec<VolumeData>) -> Self {
        self.volumes = Some(volumes);
        self
    }

    /// Set the retained ICMS on transport.
    pub fn retained_icms(mut self, retained: RetainedIcmsTransp) -> Self {
        self.retained_icms = Some(retained);
        self
    }

    /// Set the rail car number (`vagao`).
    pub fn vagao(mut self, v: impl Into<String>) -> Self {
        self.vagao = Some(v.into());
        self
    }

    /// Set the barge / ferry identification (`balsa`).
    pub fn balsa(mut self, v: impl Into<String>) -> Self {
        self.balsa = Some(v.into());
        self
    }
}

/// Carrier (transportadora) identification for freight transport.
///
/// All fields are optional to accommodate scenarios where only partial
/// carrier information is available.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct CarrierData {
    /// CNPJ or CPF of the carrier.
    pub tax_id: Option<String>,
    /// Legal name of the carrier (`xNome`).
    pub name: Option<String>,
    /// State tax registration (IE) of the carrier.
    pub state_tax_id: Option<String>,
    /// Two-letter state code (UF) of the carrier.
    pub state_code: Option<String>,
    /// Full address string of the carrier (`xEnder`).
    pub address: Option<String>,
    /// Municipality name of the carrier (`xMun`).
    pub municipality: Option<String>,
}

impl CarrierData {
    /// Create a new empty `CarrierData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tax ID.
    pub fn tax_id(mut self, v: impl Into<String>) -> Self {
        self.tax_id = Some(v.into());
        self
    }

    /// Set the name.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.name = Some(v.into());
        self
    }

    /// Set the state tax ID.
    pub fn state_tax_id(mut self, v: impl Into<String>) -> Self {
        self.state_tax_id = Some(v.into());
        self
    }

    /// Set the state code.
    pub fn state_code(mut self, v: impl Into<String>) -> Self {
        self.state_code = Some(v.into());
        self
    }

    /// Set the address.
    pub fn address(mut self, v: impl Into<String>) -> Self {
        self.address = Some(v.into());
        self
    }

    /// Set the municipality name (`xMun`).
    pub fn municipality(mut self, v: impl Into<String>) -> Self {
        self.municipality = Some(v.into());
        self
    }
}

/// Vehicle identification for transport (`veicTransp`) or trailers (`reboque`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct VehicleData {
    /// Vehicle licence plate (`placa`).
    pub plate: String,
    /// State (UF) where the vehicle is registered.
    pub state_code: String,
    /// ANTT registration code (`RNTC`). Optional.
    pub rntc: Option<String>,
}

impl VehicleData {
    /// Create a new `VehicleData` with required fields.
    pub fn new(plate: impl Into<String>, state_code: impl Into<String>) -> Self {
        Self {
            plate: plate.into(),
            state_code: state_code.into(),
            rntc: None,
        }
    }

    /// Set the RNTC code.
    pub fn rntc(mut self, rntc: impl Into<String>) -> Self {
        self.rntc = Some(rntc.into());
        self
    }
}

/// A single transported volume (`<vol>`) with optional identification and weights.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct VolumeData {
    /// Number of volumes (`qVol`).
    pub quantity: Option<u32>,
    /// Species / type of packaging (`esp`), e.g. `"CAIXA"`.
    pub species: Option<String>,
    /// Brand on the packaging (`marca`).
    pub brand: Option<String>,
    /// Volume number / identifier (`nVol`).
    pub number: Option<String>,
    /// Net weight in kilograms (`pesoL`).
    pub net_weight: Option<f64>,
    /// Gross weight in kilograms (`pesoB`).
    pub gross_weight: Option<f64>,
    /// List of seal numbers (`lacres`).
    pub seals: Option<Vec<String>>,
}

impl VolumeData {
    /// Create a new empty `VolumeData`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the quantity.
    pub fn quantity(mut self, v: u32) -> Self {
        self.quantity = Some(v);
        self
    }
    /// Set the species.
    pub fn species(mut self, v: impl Into<String>) -> Self {
        self.species = Some(v.into());
        self
    }
    /// Set the brand.
    pub fn brand(mut self, v: impl Into<String>) -> Self {
        self.brand = Some(v.into());
        self
    }
    /// Set the number.
    pub fn number(mut self, v: impl Into<String>) -> Self {
        self.number = Some(v.into());
        self
    }
    /// Set the net weight.
    pub fn net_weight(mut self, v: f64) -> Self {
        self.net_weight = Some(v);
        self
    }
    /// Set the gross weight.
    pub fn gross_weight(mut self, v: f64) -> Self {
        self.gross_weight = Some(v);
        self
    }
    /// Set the seals.
    pub fn seals(mut self, v: Vec<String>) -> Self {
        self.seals = Some(v);
        self
    }
}

/// ICMS retained on transport services (`<retTransp>`).
///
/// Applicable when the carrier is subject to ICMS withholding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[non_exhaustive]
pub struct RetainedIcmsTransp {
    /// Transport service value (`vServ`).
    pub v_serv: Cents,
    /// ICMS calculation base for the retained amount (`vBCRet`).
    pub v_bc_ret: Cents,
    /// ICMS rate applied to the retained amount (`pICMSRet`).
    pub p_icms_ret: Rate,
    /// Retained ICMS value (`vICMSRet`).
    pub v_icms_ret: Cents,
    /// CFOP code applicable to the transport service.
    pub cfop: String,
    /// IBGE city code of the municipality where the tax event occurred.
    pub city_code: IbgeCode,
}

impl RetainedIcmsTransp {
    /// Create a new `RetainedIcmsTransp` with all required fields.
    pub fn new(
        v_serv: Cents,
        v_bc_ret: Cents,
        p_icms_ret: Rate,
        v_icms_ret: Cents,
        cfop: impl Into<String>,
        city_code: IbgeCode,
    ) -> Self {
        Self {
            v_serv,
            v_bc_ret,
            p_icms_ret,
            v_icms_ret,
            cfop: cfop.into(),
            city_code,
        }
    }
}
