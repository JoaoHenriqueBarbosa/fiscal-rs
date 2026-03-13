//! Build the `<ide>` (identification) group of the NF-e XML.

use super::tax_id::TaxId;
use crate::types::{InvoiceBuildData, InvoiceModel, ReferenceDoc};
use crate::xml_utils::{TagContent, tag};

/// Format date/time for NF-e (ISO 8601 with Brazil timezone offset).
///
/// SEFAZ rejects UTC `"Z"` suffix — requires explicit offset like `-03:00`.
pub fn format_datetime_nfe(dt: &chrono::DateTime<chrono::FixedOffset>, state_code: &str) -> String {
    let offset = match state_code {
        "AC" => "-05:00",
        "AM" | "RO" | "RR" | "MT" | "MS" => "-04:00",
        _ => "-03:00",
    };
    format!("{}{offset}", dt.format("%Y-%m-%dT%H:%M:%S"))
}

/// Build the `<ide>` element with all identification fields.
pub(crate) fn build_ide(
    data: &InvoiceBuildData,
    state_ibge: &str,
    numeric_code: &str,
    access_key: &str,
) -> String {
    let ref_elements = build_references(data.references.as_deref());

    let dh_emi = format_datetime_nfe(&data.issued_at, &data.issuer.state_code);
    let series_str = data.series.to_string();
    let number_str = data.number.to_string();
    let tp_nf = data.operation_type.unwrap_or(1).to_string();
    let fin_nfe = data.purpose_code.unwrap_or(1).to_string();

    let mut children = vec![
        tag("cUF", &[], TagContent::Text(state_ibge)),
        tag("cNF", &[], TagContent::Text(numeric_code)),
        tag("natOp", &[], TagContent::Text(&data.operation_nature)),
        tag("mod", &[], TagContent::Text(data.model.as_str())),
        tag("serie", &[], TagContent::Text(&series_str)),
        tag("nNF", &[], TagContent::Text(&number_str)),
        tag("dhEmi", &[], TagContent::Text(&dh_emi)),
    ];

    // dhSaiEnt: optional, model 55 only (matching PHP behaviour)
    if data.model == InvoiceModel::Nfe {
        if let Some(ref exit_dt) = data.exit_at {
            let dh_sai_ent = format_datetime_nfe(exit_dt, &data.issuer.state_code);
            children.push(tag("dhSaiEnt", &[], TagContent::Text(&dh_sai_ent)));
        }
    }

    children.extend([
        tag("tpNF", &[], TagContent::Text(&tp_nf)),
        tag(
            "idDest",
            &[],
            TagContent::Text(data.destination_indicator.as_deref().unwrap_or("1")),
        ),
        tag("cMunFG", &[], TagContent::Text(&data.issuer.city_code.0)),
        tag(
            "tpImp",
            &[],
            TagContent::Text(data.print_format.as_deref().unwrap_or("1")),
        ),
        tag("tpEmis", &[], TagContent::Text(data.emission_type.as_str())),
        tag("cDV", &[], TagContent::Text(&access_key[43..44])),
        tag("tpAmb", &[], TagContent::Text(data.environment.as_str())),
        tag("finNFe", &[], TagContent::Text(&fin_nfe)),
        tag(
            "indFinal",
            &[],
            TagContent::Text(data.consumer_type.as_deref().unwrap_or("0")),
        ),
        tag(
            "indPres",
            &[],
            TagContent::Text(data.buyer_presence.as_deref().unwrap_or("0")),
        ),
    ]);

    // indIntermed: only emit when explicitly set (PHP uses false for required,
    // meaning null/empty values are omitted)
    if let Some(ref ind) = data.intermediary_indicator {
        children.push(tag("indIntermed", &[], TagContent::Text(ind)));
    }

    children.extend([
        tag(
            "procEmi",
            &[],
            TagContent::Text(data.emission_process.as_deref().unwrap_or("0")),
        ),
        tag(
            "verProc",
            &[],
            TagContent::Text(data.ver_proc.as_deref().unwrap_or("FinOpenPOS 1.0")),
        ),
    ]);

    // dhCont / xJust: contingency timestamp and justification (matching PHP behaviour)
    if let Some(ref cont) = data.contingency {
        let dh_cont = format_datetime_nfe(&cont.at, &data.issuer.state_code);
        children.push(tag("dhCont", &[], TagContent::Text(&dh_cont)));
        children.push(tag("xJust", &[], TagContent::Text(&cont.reason)));
    }

    children.extend(ref_elements);

    // gCompraGov and gPagAntecipado (PL_010+) go inside <ide> after NFref
    // Only emitted when schema is PL_010 or later (matching PHP: $this->schema > 9)
    if data.schema_version.is_pl010() {
        if let Some(ref cg) = data.compra_gov {
            children.push(super::optional::build_compra_gov(cg));
        }
        if let Some(ref pa) = data.pag_antecipado {
            children.push(super::optional::build_pag_antecipado(pa));
        }
    }

    tag("ide", &[], TagContent::Children(children))
}

/// Build referenced document elements (`<NFref>` inside `<ide>`).
fn build_references(references: Option<&[ReferenceDoc]>) -> Vec<String> {
    let Some(refs) = references else {
        return vec![];
    };

    refs.iter()
        .map(|r| match r {
            ReferenceDoc::Nfe { access_key } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag("refNFe", &[], TagContent::Text(access_key))]),
            ),
            ReferenceDoc::Nf {
                state_code,
                year_month,
                tax_id,
                model,
                series,
                number,
            } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag(
                    "refNF",
                    &[],
                    TagContent::Children(vec![
                        tag("cUF", &[], TagContent::Text(&state_code.0)),
                        tag("AAMM", &[], TagContent::Text(year_month)),
                        tag("CNPJ", &[], TagContent::Text(tax_id)),
                        tag("mod", &[], TagContent::Text(model)),
                        tag("serie", &[], TagContent::Text(series)),
                        tag("nNF", &[], TagContent::Text(number)),
                    ]),
                )]),
            ),
            ReferenceDoc::Nfp {
                state_code,
                year_month,
                tax_id,
                ie,
                model,
                series,
                number,
            } => {
                let tid = TaxId::new(tax_id);
                tag(
                    "NFref",
                    &[],
                    TagContent::Children(vec![tag(
                        "refNFP",
                        &[],
                        TagContent::Children(vec![
                            tag("cUF", &[], TagContent::Text(&state_code.0)),
                            tag("AAMM", &[], TagContent::Text(year_month)),
                            tag(tid.tag_name(), &[], TagContent::Text(tax_id)),
                            tag("IE", &[], TagContent::Text(ie)),
                            tag("mod", &[], TagContent::Text(model)),
                            tag("serie", &[], TagContent::Text(series)),
                            tag("nNF", &[], TagContent::Text(number)),
                        ]),
                    )]),
                )
            }
            ReferenceDoc::Cte { access_key } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag("refCTe", &[], TagContent::Text(access_key))]),
            ),
            ReferenceDoc::Ecf {
                model,
                ecf_number,
                coo_number,
            } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag(
                    "refECF",
                    &[],
                    TagContent::Children(vec![
                        tag("mod", &[], TagContent::Text(model)),
                        tag("nECF", &[], TagContent::Text(ecf_number)),
                        tag("nCOO", &[], TagContent::Text(coo_number)),
                    ]),
                )]),
            ),
        })
        .collect()
}
