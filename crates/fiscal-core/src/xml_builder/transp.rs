//! Build the `<transp>` (transport) group of the NF-e XML.

use super::tax_id::TaxId;
use crate::format_utils::{format_cents, format_decimal};
use crate::types::InvoiceBuildData;
use crate::xml_utils::{TagContent, tag};

/// Build the `<transp>` element with carrier, vehicle, volumes, etc.
pub(crate) fn build_transp(data: &InvoiceBuildData) -> String {
    let Some(ref t) = data.transport else {
        return tag(
            "transp",
            &[],
            TagContent::Children(vec![tag("modFrete", &[], TagContent::Text("9"))]),
        );
    };

    let mut children = vec![tag("modFrete", &[], TagContent::Text(&t.freight_mode))];

    // transporta (carrier) — must come BEFORE retTransp per XSD schema order:
    // modFrete, transporta, retTransp, veicTransp, reboque, vol
    if let Some(ref c) = t.carrier {
        let mut carrier_children = Vec::new();
        if let Some(ref tid) = c.tax_id {
            let t = TaxId::new(tid);
            let padded = t.padded();
            carrier_children.push(tag(t.tag_name(), &[], TagContent::Text(&padded)));
        }
        if let Some(ref name) = c.name {
            carrier_children.push(tag("xNome", &[], TagContent::Text(name)));
        }
        if let Some(ref ie) = c.state_tax_id {
            carrier_children.push(tag("IE", &[], TagContent::Text(ie)));
        }
        if let Some(ref addr) = c.address {
            carrier_children.push(tag("xEnder", &[], TagContent::Text(addr)));
        }
        if let Some(ref mun) = c.municipality {
            carrier_children.push(tag("xMun", &[], TagContent::Text(mun)));
        }
        if let Some(ref uf) = c.state_code {
            carrier_children.push(tag("UF", &[], TagContent::Text(uf)));
        }
        children.push(tag(
            "transporta",
            &[],
            TagContent::Children(carrier_children),
        ));
    }

    // retTransp (retained ICMS on transport service)
    if let Some(ref r) = t.retained_icms {
        children.push(tag(
            "retTransp",
            &[],
            TagContent::Children(vec![
                tag("vServ", &[], TagContent::Text(&format_cents(r.v_serv.0, 2))),
                tag(
                    "vBCRet",
                    &[],
                    TagContent::Text(&format_cents(r.v_bc_ret.0, 2)),
                ),
                tag(
                    "pICMSRet",
                    &[],
                    TagContent::Text(&format_decimal(r.p_icms_ret.0 as f64 / 100.0, 4)),
                ),
                tag(
                    "vICMSRet",
                    &[],
                    TagContent::Text(&format_cents(r.v_icms_ret.0, 2)),
                ),
                tag("CFOP", &[], TagContent::Text(&r.cfop)),
                tag("cMunFG", &[], TagContent::Text(&r.city_code.0)),
            ]),
        ));
    }

    // veicTransp
    if let Some(ref v) = t.vehicle {
        let mut veic_children = vec![
            tag("placa", &[], TagContent::Text(&v.plate)),
            tag("UF", &[], TagContent::Text(&v.state_code)),
        ];
        if let Some(ref rntc) = v.rntc {
            veic_children.push(tag("RNTC", &[], TagContent::Text(rntc)));
        }
        children.push(tag("veicTransp", &[], TagContent::Children(veic_children)));
    }

    // reboque (trailers)
    if let Some(ref trailers) = t.trailers {
        for trailer in trailers {
            let mut trl_children = vec![
                tag("placa", &[], TagContent::Text(&trailer.plate)),
                tag("UF", &[], TagContent::Text(&trailer.state_code)),
            ];
            if let Some(ref rntc) = trailer.rntc {
                trl_children.push(tag("RNTC", &[], TagContent::Text(rntc)));
            }
            children.push(tag("reboque", &[], TagContent::Children(trl_children)));
        }
    }

    // vagao — mutually exclusive with veicTransp/reboque
    let has_veic_or_reboque =
        t.vehicle.is_some() || t.trailers.as_ref().is_some_and(|v| !v.is_empty());
    if let Some(ref vagao) = t.vagao {
        if !has_veic_or_reboque {
            children.push(tag("vagao", &[], TagContent::Text(vagao)));
        }
    }

    // balsa — mutually exclusive with veicTransp/reboque/vagao
    if let Some(ref balsa) = t.balsa {
        if !has_veic_or_reboque && t.vagao.is_none() {
            children.push(tag("balsa", &[], TagContent::Text(balsa)));
        }
    }

    // vol (volumes)
    if let Some(ref volumes) = t.volumes {
        for vol in volumes {
            let mut vol_children = Vec::new();
            if let Some(qty) = vol.quantity {
                vol_children.push(tag("qVol", &[], TagContent::Text(&qty.to_string())));
            }
            if let Some(ref sp) = vol.species {
                vol_children.push(tag("esp", &[], TagContent::Text(sp)));
            }
            if let Some(ref brand) = vol.brand {
                vol_children.push(tag("marca", &[], TagContent::Text(brand)));
            }
            if let Some(ref num) = vol.number {
                vol_children.push(tag("nVol", &[], TagContent::Text(num)));
            }
            if let Some(w) = vol.net_weight {
                vol_children.push(tag("pesoL", &[], TagContent::Text(&format_decimal(w, 3))));
            }
            if let Some(w) = vol.gross_weight {
                vol_children.push(tag("pesoB", &[], TagContent::Text(&format_decimal(w, 3))));
            }
            if let Some(ref seals) = vol.seals {
                for seal in seals {
                    vol_children.push(tag(
                        "lacres",
                        &[],
                        TagContent::Children(vec![tag("nLacre", &[], TagContent::Text(seal))]),
                    ));
                }
            }
            children.push(tag("vol", &[], TagContent::Children(vol_children)));
        }
    }

    tag("transp", &[], TagContent::Children(children))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::IbgeCode;
    use crate::types::{
        InvoiceBuildData, InvoiceModel, IssuerData, SefazEnvironment, TaxRegime, TransportData,
        VehicleData,
    };

    fn sample_build_data_with_transport(t: TransportData) -> InvoiceBuildData {
        let issuer = IssuerData::new(
            "12345678000199",
            "123456789",
            "Test Company",
            TaxRegime::SimplesNacional,
            "SP",
            IbgeCode("3550308".to_string()),
            "Sao Paulo",
            "Av Paulista",
            "1000",
            "Bela Vista",
            "01310100",
        );

        InvoiceBuildData {
            schema_version: crate::types::SchemaVersion::PL009,
            model: InvoiceModel::Nfe,
            series: 1,
            number: 1,
            emission_type: crate::types::EmissionType::Normal,
            environment: SefazEnvironment::Homologation,
            issued_at: chrono::Utc::now()
                .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).expect("valid offset")),
            operation_nature: "VENDA".to_string(),
            issuer,
            recipient: None,
            items: Vec::new(),
            payments: Vec::new(),
            change_amount: None,
            payment_card_details: None,
            contingency: None,
            operation_type: None,
            purpose_code: None,
            intermediary_indicator: None,
            emission_process: None,
            consumer_type: None,
            buyer_presence: None,
            print_format: None,
            references: None,
            transport: Some(t),
            billing: None,
            withdrawal: None,
            delivery: None,
            authorized_xml: None,
            additional_info: None,
            intermediary: None,
            ret_trib: None,
            tech_responsible: None,
            purchase: None,
            export: None,
            issqn_tot: None,
            cana: None,
            agropecuario: None,
            compra_gov: None,
            pag_antecipado: None,
            is_tot: None,
            ibs_cbs_tot: None,
            v_nf_tot_override: None,
            exit_at: None,
            destination_indicator: None,
            ver_proc: None,
            only_ascii: false,
            calculation_method: crate::types::CalculationMethod::V2,
        }
    }

    #[test]
    fn vagao_emitted_when_no_vehicle_or_trailers() {
        let t = TransportData::new("1").vagao("VAGAO-001");
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        assert!(xml.contains("<vagao>VAGAO-001</vagao>"));
        assert!(!xml.contains("<balsa>"));
    }

    #[test]
    fn balsa_emitted_when_no_vehicle_trailers_or_vagao() {
        let t = TransportData::new("1").balsa("BALSA-001");
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        assert!(xml.contains("<balsa>BALSA-001</balsa>"));
        assert!(!xml.contains("<vagao>"));
    }

    #[test]
    fn vagao_suppressed_when_vehicle_present() {
        let t = TransportData::new("1")
            .vehicle(VehicleData::new("ABC1234", "SP"))
            .vagao("VAGAO-001");
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        assert!(!xml.contains("<vagao>"));
        assert!(xml.contains("<veicTransp>"));
    }

    #[test]
    fn vagao_suppressed_when_trailers_present() {
        let t = TransportData::new("1")
            .trailers(vec![VehicleData::new("XYZ9876", "RJ")])
            .vagao("VAGAO-001");
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        assert!(!xml.contains("<vagao>"));
        assert!(xml.contains("<reboque>"));
    }

    #[test]
    fn balsa_suppressed_when_vagao_present() {
        let t = TransportData::new("1")
            .vagao("VAGAO-001")
            .balsa("BALSA-001");
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        assert!(xml.contains("<vagao>VAGAO-001</vagao>"));
        assert!(!xml.contains("<balsa>"));
    }

    #[test]
    fn balsa_suppressed_when_vehicle_present() {
        let t = TransportData::new("1")
            .vehicle(VehicleData::new("ABC1234", "SP"))
            .balsa("BALSA-001");
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        assert!(!xml.contains("<balsa>"));
        assert!(xml.contains("<veicTransp>"));
    }

    #[test]
    fn vagao_position_after_reboque_before_vol() {
        use crate::types::VolumeData;

        let t = TransportData::new("1")
            .vagao("V-100")
            .volumes(vec![VolumeData::new().quantity(5)]);
        let data = sample_build_data_with_transport(t);
        let xml = build_transp(&data);

        let vagao_pos = xml.find("<vagao>").expect("<vagao> must exist");
        let vol_pos = xml.find("<vol>").expect("<vol> must exist");
        assert!(vagao_pos < vol_pos, "vagao must come before vol");
    }
}
