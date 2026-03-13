//! TXT-to-XML converter and validator for NFe documents.
//!
//! Ported from TypeScript `convert.ts`, `valid-txt.ts`, and `txt-structures.ts`.

use std::collections::HashMap;

use crate::FiscalError;
use crate::xml_utils::escape_xml;

// ── Layout constants ────────────────────────────────────────────────────────

const NFE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/nfe";

// ── TXT structures ──────────────────────────────────────────────────────────

/// Return the TXT field structure map for a given version string and layout.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTxt`] if the version/layout combination
/// is not supported.
fn get_structure(
    version: &str,
    layout: &str,
) -> Result<HashMap<&'static str, &'static str>, FiscalError> {
    let ver: u32 = version.replace('.', "").parse().unwrap_or(0);
    let lay = layout.to_uppercase();

    if ver == 310 {
        return Ok(structure_310());
    }
    if ver == 400 {
        if lay == "SEBRAE" {
            return Ok(structure_400_sebrae());
        }
        if lay == "LOCAL_V12" {
            return Ok(structure_400_v12());
        }
        if lay == "LOCAL_V13" {
            return Ok(structure_400_v13());
        }
        return Ok(structure_400());
    }

    Err(FiscalError::InvalidTxt(format!(
        "Structure definition for TXT layout version {version} ({layout}) was not found."
    )))
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Convert SPED TXT format to NF-e XML (first invoice only).
///
/// Convenience wrapper around [`txt_to_xml_all`] that returns only the first
/// invoice XML. Use this when you know the TXT contains a single NF-e.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTxt`] if the TXT is empty, not a valid
/// NOTAFISCAL document, has structural errors, or if the access key is
/// malformed. Returns [`FiscalError::WrongDocument`] if the document header
/// is missing.
pub fn txt_to_xml(txt: &str, layout: &str) -> Result<String, FiscalError> {
    let mut xmls = txt_to_xml_all(txt, layout)?;
    // txt_to_xml_all guarantees at least one element on success.
    Ok(xmls.swap_remove(0))
}

/// Convert SPED TXT format to NF-e XML for **all** invoices in the file.
///
/// Parses the pipe-delimited TXT representation of one or more NF-e invoices
/// and produces a `Vec<String>` containing the XML for each invoice, in the
/// same order they appear in the TXT. Supports layouts:
/// `"local"`, `"local_v12"`, `"local_v13"`, `"sebrae"`.
///
/// This mirrors the PHP `Convert::parse()` / `toXml()` behaviour which
/// returns an array of XML strings — one per nota fiscal.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTxt`] if the TXT is empty, not a valid
/// NOTAFISCAL document, has structural errors, or if the access key is
/// malformed. Returns [`FiscalError::WrongDocument`] if the document header
/// is missing or the declared invoice count does not match.
pub fn txt_to_xml_all(txt: &str, layout: &str) -> Result<Vec<String>, FiscalError> {
    let txt = txt.trim();
    if txt.is_empty() {
        return Err(FiscalError::WrongDocument("Empty document".into()));
    }

    let lines: Vec<&str> = txt.lines().collect();
    let first_fields: Vec<&str> = lines[0].split('|').collect();
    if first_fields[0] != "NOTAFISCAL" {
        return Err(FiscalError::WrongDocument(
            "Wrong document: not a valid NFe TXT".into(),
        ));
    }

    let declared_count: usize = first_fields
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let rest: Vec<&str> = lines[1..].to_vec();

    // Slice invoices
    let invoices = slice_invoices(&rest, declared_count);
    if invoices.len() != declared_count {
        return Err(FiscalError::WrongDocument(format!(
            "Number of NFe declared ({declared_count}) does not match found ({})",
            invoices.len()
        )));
    }

    let norm_layout = normalize_layout(layout);
    let mut xmls = Vec::with_capacity(declared_count);

    for invoice in &invoices {
        let version = extract_layout_version(invoice)?;

        // Validate
        let errors = validate_txt_lines(invoice, &norm_layout);
        if !errors.is_empty() {
            return Err(FiscalError::InvalidTxt(errors.join("\n")));
        }

        // Parse
        let structure = get_structure(&version, &norm_layout)?;
        let mut parser = NFeParser::new(&version, &norm_layout, &structure);
        parser.parse(invoice);

        // Validate access key
        if !parser.inf_nfe_id.is_empty() {
            let key = parser
                .inf_nfe_id
                .strip_prefix("NFe")
                .unwrap_or(&parser.inf_nfe_id);
            if !key.is_empty() && key.len() != 44 {
                return Err(FiscalError::InvalidTxt(format!(
                    "A chave informada est\u{e1} incorreta [{}]",
                    parser.inf_nfe_id
                )));
            }
        }

        xmls.push(parser.build_xml());
    }

    Ok(xmls)
}

/// Validate TXT format structure without converting to XML.
///
/// Returns `Ok(true)` if the TXT passes structural validation, or
/// `Ok(false)` / `Err` if validation errors are found.
///
/// # Errors
///
/// Returns [`FiscalError::WrongDocument`] if the document header is missing
/// or empty.
pub fn validate_txt(txt: &str, layout: &str) -> Result<bool, FiscalError> {
    let txt = txt.replace(['\r', '\t'], "");
    let txt = txt.trim();
    if txt.is_empty() {
        return Err(FiscalError::WrongDocument("Empty document".into()));
    }

    let lines: Vec<&str> = txt.lines().collect();
    let first_fields: Vec<&str> = lines[0].split('|').collect();
    if first_fields[0] != "NOTAFISCAL" {
        return Err(FiscalError::WrongDocument(
            "Wrong document: not a valid NFe TXT".into(),
        ));
    }

    let rest: Vec<&str> = lines[1..].to_vec();
    let norm_layout = normalize_layout(layout);
    let errors = validate_txt_lines(&rest, &norm_layout);

    if errors.is_empty() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn normalize_layout(layout: &str) -> String {
    let up = layout.to_uppercase();
    match up.as_str() {
        "LOCAL" | "LOCAL_V12" | "LOCAL_V13" | "SEBRAE" => up,
        _ => "LOCAL_V12".to_string(),
    }
}

fn slice_invoices<'a>(rest: &[&'a str], declared: usize) -> Vec<Vec<&'a str>> {
    if declared <= 1 {
        return vec![rest.to_vec()];
    }

    let mut markers: Vec<(usize, usize)> = Vec::new();
    for (i, line) in rest.iter().enumerate() {
        if line.starts_with("A|") {
            if let Some(last) = markers.last_mut() {
                last.1 = i;
            }
            markers.push((i, 0));
        }
    }
    if let Some(last) = markers.last_mut() {
        last.1 = rest.len();
    }

    markers.iter().map(|(s, e)| rest[*s..*e].to_vec()).collect()
}

fn extract_layout_version(invoice: &[&str]) -> Result<String, FiscalError> {
    for line in invoice {
        let fields: Vec<&str> = line.split('|').collect();
        if fields[0] == "A" {
            return Ok(fields.get(1).unwrap_or(&"4.00").to_string());
        }
    }
    Err(FiscalError::InvalidTxt(
        "No 'A' entity found in invoice".into(),
    ))
}

/// Validate TXT lines for structural correctness (field counts, forbidden chars).
fn validate_txt_lines(lines: &[&str], layout: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut num = 0;
    let mut entities: Option<HashMap<&str, &str>> = None;

    for row in lines {
        if row.is_empty() {
            continue;
        }
        let fields: Vec<&str> = row.split('|').collect();
        let ref_upper = fields[0].to_uppercase();
        if ref_upper.is_empty() {
            continue;
        }
        if ref_upper == "NOTAFISCAL" {
            continue;
        }

        if ref_upper == "A" {
            num = 0;
            let ver = fields.get(1).unwrap_or(&"4.00");
            entities = get_structure(ver, layout).ok();
        }
        if ref_upper == "I" {
            num += 1;
        }

        // Check trailing pipe
        let last_char = row.chars().last().unwrap_or(' ');
        if last_char != '|' {
            let char_desc = match last_char {
                ' ' => "[ESP]".to_string(),
                '\r' => "[CR]".to_string(),
                '\t' => "[TAB]".to_string(),
                _ => String::new(),
            };
            errors.push(format!(
                "ERRO: ({num}) Todas as linhas devem terminar com 'pipe' e n\u{e3}o {char_desc}. [{row}]"
            ));
            continue;
        }

        let ent = match &entities {
            Some(e) => e,
            None => {
                errors.push("ERRO: O TXT n\u{e3}o cont\u{e9}m um marcador A".into());
                return errors;
            }
        };

        if !ent.contains_key(ref_upper.as_str()) {
            errors.push(format!(
                "ERRO: ({num}) Essa refer\u{ea}ncia n\u{e3}o est\u{e1} definida. [{row}]"
            ));
            continue;
        }

        let count = fields.len() - 1;
        let def = ent[ref_upper.as_str()];
        let default_count = def.split('|').count() - 1;
        if default_count != count {
            errors.push(format!(
                "ERRO: ({num}) O n\u{fa}mero de par\u{e2}metros na linha est\u{e1} errado (esperado #{default_count}) -> (encontrado #{count}). [ {row} ] Esperado [ {def} ]"
            ));
            continue;
        }

        // Check fields for forbidden characters
        for field in &fields {
            if field.is_empty() {
                continue;
            }
            if !field.trim().is_empty() && field.chars().all(|c| c == ' ') {
                errors.push(format!(
                    "ERRO: ({num}) Existem apenas espa\u{e7}os no campo dos dados. [{row}]"
                ));
                continue;
            }
            if field.contains('>')
                || field.contains('<')
                || field.contains('"')
                || field.contains('\'')
                || field.contains('\t')
                || field.contains('\r')
            {
                errors.push(format!(
                    "ERRO: ({num}) Existem caracteres especiais n\u{e3}o permitidos, como por ex. caracteres de controle, sinais de maior ou menor, aspas ou apostrofes, na entidade [{row}]"
                ));
                continue;
            }
        }
    }

    errors
}

// ── NFeParser ───────────────────────────────────────────────────────────────

/// Parsed fields from a single TXT line, keyed by field name.
type Fields = HashMap<String, String>;

/// Parse pipe-delimited fields against a structure definition.
fn fields_to_std(fields: &[&str], struct_def: &str) -> Fields {
    let struct_fields: Vec<&str> = struct_def.split('|').collect();
    let mut std = Fields::new();
    let len = struct_fields.len().saturating_sub(1);
    for (i, name) in struct_fields.iter().enumerate().take(len).skip(1) {
        let data = fields.get(i).copied().unwrap_or("");
        if !name.is_empty() && !data.is_empty() {
            std.insert(name.to_string(), data.to_string());
        }
    }
    std
}

fn xml_tag(name: &str, content: &str) -> String {
    format!("<{name}>{content}</{name}>")
}

fn add_child(arr: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(v) = value {
        arr.push(format!("<{name}>{}</{name}>", escape_xml(v)));
    }
}

fn add_child_str(arr: &mut Vec<String>, name: &str, value: &str) {
    arr.push(format!("<{name}>{}</{name}>", escape_xml(value)));
}

/// Accumulated data for a single det item.
#[allow(dead_code)]
struct ItemBuild {
    n_item: usize,
    prod: Fields,
    cest: Option<Fields>,
    g_cred: Option<Fields>,
    v_tot_trib: String,
    icms_tag: String,
    icms_data: Option<Fields>,
    ipi_header: Option<Fields>,
    ipi_cst: String,
    ipi_v_ipi: String,
    ipi_v_bc: String,
    ipi_p_ipi: String,
    pis_cst: String,
    pis_v_bc: String,
    pis_p_pis: String,
    pis_v_pis: String,
    cofins_cst: String,
    cofins_v_bc: String,
    cofins_p_cofins: String,
    cofins_v_cofins: String,
}

struct NFeParser<'a> {
    structure: &'a HashMap<&'a str, &'a str>,
    base_layout: String,

    // Header
    inf_nfe_id: String,
    inf_nfe_versao: String,

    // Collected data
    ide_data: Fields,
    nf_ref: Vec<String>,
    emit_fields: Fields,
    ender_emit_fields: Fields,
    dest_fields: Fields,
    ender_dest_fields: Fields,
    items: Vec<ItemBuild>,
    totals_fields: Fields,
    transp_fields: Fields,
    transporta_fields: Option<Fields>,
    volumes: Vec<Fields>,
    fat_fields: Option<Fields>,
    dup_items: Vec<Fields>,
    pag_fields: Option<Fields>,
    det_pag_list: Vec<Fields>,
    inf_adic_fields: Fields,

    // Current item accumulation
    current_item_num: usize,
    cur_prod: Fields,
    cur_cest: Option<Fields>,
    cur_g_cred: Option<Fields>,
    cur_v_tot_trib: String,
    cur_icms_tag: String,
    cur_icms_data: Option<Fields>,
    cur_ipi_header: Option<Fields>,
    cur_ipi_cst: String,
    cur_ipi_v_ipi: String,
    cur_ipi_v_bc: String,
    cur_ipi_p_ipi: String,
    cur_pis_cst: String,
    cur_pis_v_bc: String,
    cur_pis_p_pis: String,
    cur_pis_v_pis: String,
    cur_cofins_cst: String,
    cur_cofins_v_bc: String,
    cur_cofins_p_cofins: String,
    cur_cofins_v_cofins: String,
}

impl<'a> NFeParser<'a> {
    fn new(version: &str, layout: &str, structure: &'a HashMap<&'a str, &'a str>) -> Self {
        let _ = version;
        Self {
            structure,
            base_layout: layout.to_uppercase(),
            inf_nfe_id: String::new(),
            inf_nfe_versao: "4.00".into(),
            ide_data: Fields::new(),
            nf_ref: Vec::new(),
            emit_fields: Fields::new(),
            ender_emit_fields: Fields::new(),
            dest_fields: Fields::new(),
            ender_dest_fields: Fields::new(),
            items: Vec::new(),
            totals_fields: Fields::new(),
            transp_fields: Fields::new(),
            transporta_fields: None,
            volumes: Vec::new(),
            fat_fields: None,
            dup_items: Vec::new(),
            pag_fields: None,
            det_pag_list: Vec::new(),
            inf_adic_fields: Fields::new(),
            current_item_num: 0,
            cur_prod: Fields::new(),
            cur_cest: None,
            cur_g_cred: None,
            cur_v_tot_trib: String::new(),
            cur_icms_tag: String::new(),
            cur_icms_data: None,
            cur_ipi_header: None,
            cur_ipi_cst: String::new(),
            cur_ipi_v_ipi: String::new(),
            cur_ipi_v_bc: String::new(),
            cur_ipi_p_ipi: String::new(),
            cur_pis_cst: String::new(),
            cur_pis_v_bc: String::new(),
            cur_pis_p_pis: String::new(),
            cur_pis_v_pis: String::new(),
            cur_cofins_cst: String::new(),
            cur_cofins_v_bc: String::new(),
            cur_cofins_p_cofins: String::new(),
            cur_cofins_v_cofins: String::new(),
        }
    }

    fn parse(&mut self, invoice: &[&str]) {
        for line in invoice {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let fields: Vec<&str> = trimmed.split('|').collect();
            let ref_upper = fields[0].to_uppercase();
            let struct_def = match self.structure.get(ref_upper.as_str()) {
                Some(d) => *d,
                None => continue,
            };
            let std = fields_to_std(&fields, struct_def);
            self.handle(&ref_upper, &std);
        }
        self.finalize_current_item();
    }

    fn handle(&mut self, ref_name: &str, std: &Fields) {
        match ref_name {
            "A" => {
                self.inf_nfe_versao = std.get("versao").cloned().unwrap_or_else(|| "4.00".into());
                self.inf_nfe_id = std.get("Id").cloned().unwrap_or_default();
            }
            "B" => {
                self.ide_data = std.clone();
            }
            "BA" | "BB" => {}
            "BA02" | "BB02" => {
                if let Some(v) = std.get("refNFe") {
                    self.nf_ref.push(v.clone());
                }
            }
            "C" => {
                self.emit_fields = std.clone();
            }
            "C02" => {
                if let Some(v) = std.get("CNPJ") {
                    self.emit_fields.insert("CNPJ".into(), v.clone());
                }
            }
            "C02A" => {
                if let Some(v) = std.get("CPF") {
                    self.emit_fields.insert("CPF".into(), v.clone());
                }
            }
            "C05" => {
                self.ender_emit_fields = std.clone();
            }
            "D" => {}
            "E" => {
                self.dest_fields = std.clone();
            }
            "E02" => {
                if let Some(v) = std.get("CNPJ") {
                    self.dest_fields.insert("CNPJ".into(), v.clone());
                }
            }
            "E03" => {
                if let Some(v) = std.get("CPF") {
                    self.dest_fields.insert("CPF".into(), v.clone());
                }
            }
            "E03A" => {
                let v = std.get("idEstrangeiro").cloned().unwrap_or_default();
                self.dest_fields.insert("idEstrangeiro".into(), v);
            }
            "E05" => {
                self.ender_dest_fields = std.clone();
            }
            "H" => {
                self.finalize_current_item();
                self.current_item_num = std.get("item").and_then(|s| s.parse().ok()).unwrap_or(0);
                self.cur_prod = Fields::new();
                self.cur_cest = None;
                self.cur_g_cred = None;
                self.cur_v_tot_trib = String::new();
                self.cur_icms_tag = String::new();
                self.cur_icms_data = None;
                self.cur_ipi_header = None;
                self.cur_ipi_cst = String::new();
                self.cur_ipi_v_ipi = String::new();
                self.cur_ipi_v_bc = String::new();
                self.cur_ipi_p_ipi = String::new();
                self.cur_pis_cst = String::new();
                self.cur_pis_v_bc = String::new();
                self.cur_pis_p_pis = String::new();
                self.cur_pis_v_pis = String::new();
                self.cur_cofins_cst = String::new();
                self.cur_cofins_v_bc = String::new();
                self.cur_cofins_p_cofins = String::new();
                self.cur_cofins_v_cofins = String::new();
            }
            "I" => {
                self.cur_prod = std.clone();
            }
            "I05C" => {
                self.cur_cest = Some(std.clone());
            }
            "I05G" => {
                self.cur_g_cred = Some(std.clone());
            }
            "M" => {
                self.cur_v_tot_trib = std.get("vTotTrib").cloned().unwrap_or_default();
            }
            "N" => {}
            "N02" | "N03" | "N04" | "N05" | "N06" | "N07" | "N08" | "N09" | "N10" => {
                self.cur_icms_tag = ref_name.to_string();
                self.cur_icms_data = Some(std.clone());
            }
            "O" => {
                self.cur_ipi_header = Some(std.clone());
            }
            "O07" => {
                self.cur_ipi_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_ipi_v_ipi = std.get("vIPI").cloned().unwrap_or_default();
            }
            "O08" => {
                self.cur_ipi_cst = std.get("CST").cloned().unwrap_or_default();
            }
            "O10" => {
                self.cur_ipi_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_ipi_p_ipi = std.get("pIPI").cloned().unwrap_or_default();
            }
            "Q" => {}
            "Q02" => {
                self.cur_pis_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_pis_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_pis_p_pis = std.get("pPIS").cloned().unwrap_or_default();
                self.cur_pis_v_pis = std.get("vPIS").cloned().unwrap_or_default();
            }
            "S" => {}
            "S02" => {
                self.cur_cofins_cst = std.get("CST").cloned().unwrap_or_default();
                self.cur_cofins_v_bc = std.get("vBC").cloned().unwrap_or_default();
                self.cur_cofins_p_cofins = std.get("pCOFINS").cloned().unwrap_or_default();
                self.cur_cofins_v_cofins = std.get("vCOFINS").cloned().unwrap_or_default();
            }
            "W" => {
                self.finalize_current_item();
            }
            "W02" => {
                self.totals_fields = std.clone();
            }
            "W04C" | "W04E" | "W04G" => {}
            "X" => {
                self.transp_fields = std.clone();
            }
            "X03" => {
                self.transporta_fields = Some(std.clone());
            }
            "X04" => {
                if let Some(ref mut t) = self.transporta_fields {
                    if let Some(v) = std.get("CNPJ") {
                        t.insert("CNPJ".into(), v.clone());
                    }
                }
            }
            "X05" => {
                if let Some(ref mut t) = self.transporta_fields {
                    if let Some(v) = std.get("CPF") {
                        t.insert("CPF".into(), v.clone());
                    }
                }
            }
            "X26" => {
                self.volumes.push(std.clone());
            }
            "Y" => {
                self.pag_fields = Some(std.clone());
            }
            "Y02" => {
                self.fat_fields = Some(std.clone());
            }
            "Y07" => {
                self.dup_items.push(std.clone());
            }
            "YA" => {
                if self.base_layout == "SEBRAE" {
                    self.pag_fields = Some(std.clone());
                } else {
                    self.det_pag_list.push(std.clone());
                }
            }
            "YA01" => {
                self.det_pag_list.push(std.clone());
            }
            "Z" => {
                self.inf_adic_fields = std.clone();
            }
            _ => {}
        }
    }

    fn finalize_current_item(&mut self) {
        if !self.cur_prod.contains_key("cProd") && !self.cur_prod.contains_key("xProd") {
            return;
        }
        self.items.push(ItemBuild {
            n_item: self.current_item_num,
            prod: std::mem::take(&mut self.cur_prod),
            cest: self.cur_cest.take(),
            g_cred: self.cur_g_cred.take(),
            v_tot_trib: std::mem::take(&mut self.cur_v_tot_trib),
            icms_tag: std::mem::take(&mut self.cur_icms_tag),
            icms_data: self.cur_icms_data.take(),
            ipi_header: self.cur_ipi_header.take(),
            ipi_cst: std::mem::take(&mut self.cur_ipi_cst),
            ipi_v_ipi: std::mem::take(&mut self.cur_ipi_v_ipi),
            ipi_v_bc: std::mem::take(&mut self.cur_ipi_v_bc),
            ipi_p_ipi: std::mem::take(&mut self.cur_ipi_p_ipi),
            pis_cst: std::mem::take(&mut self.cur_pis_cst),
            pis_v_bc: std::mem::take(&mut self.cur_pis_v_bc),
            pis_p_pis: std::mem::take(&mut self.cur_pis_p_pis),
            pis_v_pis: std::mem::take(&mut self.cur_pis_v_pis),
            cofins_cst: std::mem::take(&mut self.cur_cofins_cst),
            cofins_v_bc: std::mem::take(&mut self.cur_cofins_v_bc),
            cofins_p_cofins: std::mem::take(&mut self.cur_cofins_p_cofins),
            cofins_v_cofins: std::mem::take(&mut self.cur_cofins_v_cofins),
        });
    }

    // ── XML building ────────────────────────────────────────────────────

    fn build_xml(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!(
            "<NFe xmlns=\"{NFE_NAMESPACE}\"><infNFe Id=\"{}\" versao=\"{}\">",
            self.inf_nfe_id, self.inf_nfe_versao
        ));

        parts.push(self.build_ide());
        parts.push(self.build_emit());
        parts.push(self.build_dest());

        for (i, item) in self.items.iter().enumerate() {
            parts.push(self.build_det(item, i + 1));
        }

        parts.push(self.build_total());
        parts.push(self.build_transp());

        if self.fat_fields.is_some() {
            parts.push(self.build_cobr());
        }

        parts.push(self.build_pag());

        if self.inf_adic_fields.contains_key("infAdFisco")
            || self.inf_adic_fields.contains_key("infCpl")
        {
            parts.push(self.build_inf_adic());
        }

        parts.push("</infNFe></NFe>".into());
        parts.join("")
    }

    fn build_ide(&self) -> String {
        let d = &self.ide_data;
        let mut c = Vec::new();
        add_child(&mut c, "cUF", d.get("cUF").map(|s| s.as_str()));
        let cnf = d.get("cNF").map(|s| s.as_str()).unwrap_or("");
        let cnf_padded = format!("{:0>8}", cnf);
        add_child_str(&mut c, "cNF", &cnf_padded);
        add_child(&mut c, "natOp", d.get("natOp").map(|s| s.as_str()));
        add_child(&mut c, "mod", d.get("mod").map(|s| s.as_str()));
        add_child(&mut c, "serie", d.get("serie").map(|s| s.as_str()));
        add_child(&mut c, "nNF", d.get("nNF").map(|s| s.as_str()));
        add_child(&mut c, "dhEmi", d.get("dhEmi").map(|s| s.as_str()));
        if let Some(v) = d.get("dhSaiEnt") {
            if !v.is_empty() {
                add_child_str(&mut c, "dhSaiEnt", v);
            }
        }
        add_child(&mut c, "tpNF", d.get("tpNF").map(|s| s.as_str()));
        add_child(&mut c, "idDest", d.get("idDest").map(|s| s.as_str()));
        add_child(&mut c, "cMunFG", d.get("cMunFG").map(|s| s.as_str()));
        add_child(&mut c, "tpImp", d.get("tpImp").map(|s| s.as_str()));
        add_child(&mut c, "tpEmis", d.get("tpEmis").map(|s| s.as_str()));
        let cdv = d.get("cDV").map(|s| s.as_str()).unwrap_or("0");
        add_child_str(&mut c, "cDV", cdv);
        add_child(&mut c, "tpAmb", d.get("tpAmb").map(|s| s.as_str()));
        add_child(&mut c, "finNFe", d.get("finNFe").map(|s| s.as_str()));
        add_child(&mut c, "indFinal", d.get("indFinal").map(|s| s.as_str()));
        add_child(&mut c, "indPres", d.get("indPres").map(|s| s.as_str()));
        if let Some(v) = d.get("indIntermed") {
            if !v.is_empty() {
                add_child_str(&mut c, "indIntermed", v);
            }
        }
        add_child(&mut c, "procEmi", d.get("procEmi").map(|s| s.as_str()));
        add_child(&mut c, "verProc", d.get("verProc").map(|s| s.as_str()));

        for r in &self.nf_ref {
            c.push(xml_tag("NFref", &xml_tag("refNFe", r)));
        }

        if let (Some(dhcont), Some(xjust)) = (d.get("dhCont"), d.get("xJust")) {
            if !dhcont.is_empty() && !xjust.is_empty() {
                add_child_str(&mut c, "dhCont", dhcont);
                add_child_str(&mut c, "xJust", xjust);
            }
        }

        xml_tag("ide", &c.join(""))
    }

    fn build_emit(&self) -> String {
        let e = &self.emit_fields;
        let mut c = Vec::new();
        if let Some(v) = e.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = e.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        add_child(&mut c, "xNome", e.get("xNome").map(|s| s.as_str()));
        if let Some(v) = e.get("xFant") {
            if !v.is_empty() {
                add_child_str(&mut c, "xFant", v);
            }
        }

        // enderEmit
        let ee = &self.ender_emit_fields;
        let mut ec = Vec::new();
        add_child(&mut ec, "xLgr", ee.get("xLgr").map(|s| s.as_str()));
        add_child(&mut ec, "nro", ee.get("nro").map(|s| s.as_str()));
        if let Some(v) = ee.get("xCpl") {
            if !v.is_empty() {
                add_child_str(&mut ec, "xCpl", v);
            }
        }
        add_child(&mut ec, "xBairro", ee.get("xBairro").map(|s| s.as_str()));
        add_child(&mut ec, "cMun", ee.get("cMun").map(|s| s.as_str()));
        add_child(&mut ec, "xMun", ee.get("xMun").map(|s| s.as_str()));
        add_child(&mut ec, "UF", ee.get("UF").map(|s| s.as_str()));
        add_child(&mut ec, "CEP", ee.get("CEP").map(|s| s.as_str()));
        if let Some(v) = ee.get("cPais") {
            if !v.is_empty() {
                add_child_str(&mut ec, "cPais", v);
            }
        }
        if let Some(v) = ee.get("xPais") {
            if !v.is_empty() {
                add_child_str(&mut ec, "xPais", v);
            }
        }
        if let Some(v) = ee.get("fone") {
            if !v.is_empty() {
                add_child_str(&mut ec, "fone", v);
            }
        }
        c.push(xml_tag("enderEmit", &ec.join("")));

        if let Some(v) = e.get("IE") {
            if !v.is_empty() {
                add_child_str(&mut c, "IE", v);
            }
        }
        if let Some(v) = e.get("IEST") {
            if !v.is_empty() {
                add_child_str(&mut c, "IEST", v);
            }
        }
        if let Some(v) = e.get("IM") {
            if !v.is_empty() {
                add_child_str(&mut c, "IM", v);
            }
        }
        if let Some(v) = e.get("CNAE") {
            if !v.is_empty() {
                add_child_str(&mut c, "CNAE", v);
            }
        }
        add_child(&mut c, "CRT", e.get("CRT").map(|s| s.as_str()));

        xml_tag("emit", &c.join(""))
    }

    fn build_dest(&self) -> String {
        let d = &self.dest_fields;
        let mut c = Vec::new();
        if let Some(v) = d.get("CNPJ") {
            add_child_str(&mut c, "CNPJ", v);
        }
        if let Some(v) = d.get("CPF") {
            add_child_str(&mut c, "CPF", v);
        }
        if let Some(v) = d.get("idEstrangeiro") {
            add_child_str(&mut c, "idEstrangeiro", v);
        }
        add_child(&mut c, "xNome", d.get("xNome").map(|s| s.as_str()));

        // enderDest
        let ee = &self.ender_dest_fields;
        if ee.get("xLgr").is_some() {
            let mut ec = Vec::new();
            add_child(&mut ec, "xLgr", ee.get("xLgr").map(|s| s.as_str()));
            add_child(&mut ec, "nro", ee.get("nro").map(|s| s.as_str()));
            if let Some(v) = ee.get("xCpl") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "xCpl", v);
                }
            }
            add_child(&mut ec, "xBairro", ee.get("xBairro").map(|s| s.as_str()));
            add_child(&mut ec, "cMun", ee.get("cMun").map(|s| s.as_str()));
            add_child(&mut ec, "xMun", ee.get("xMun").map(|s| s.as_str()));
            add_child(&mut ec, "UF", ee.get("UF").map(|s| s.as_str()));
            add_child(&mut ec, "CEP", ee.get("CEP").map(|s| s.as_str()));
            if let Some(v) = ee.get("cPais") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "cPais", v);
                }
            }
            if let Some(v) = ee.get("xPais") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "xPais", v);
                }
            }
            if let Some(v) = ee.get("fone") {
                if !v.is_empty() {
                    add_child_str(&mut ec, "fone", v);
                }
            }
            c.push(xml_tag("enderDest", &ec.join("")));
        }

        if let Some(v) = d.get("indIEDest") {
            if !v.is_empty() {
                add_child_str(&mut c, "indIEDest", v);
            }
        }
        if let Some(v) = d.get("IE") {
            if !v.is_empty() {
                add_child_str(&mut c, "IE", v);
            }
        }
        if let Some(v) = d.get("ISUF") {
            if !v.is_empty() {
                add_child_str(&mut c, "ISUF", v);
            }
        }
        if let Some(v) = d.get("IM") {
            if !v.is_empty() {
                add_child_str(&mut c, "IM", v);
            }
        }
        if let Some(v) = d.get("email") {
            if !v.is_empty() {
                add_child_str(&mut c, "email", v);
            }
        }

        xml_tag("dest", &c.join(""))
    }

    fn build_det(&self, item: &ItemBuild, n_item: usize) -> String {
        let mut c = Vec::new();
        c.push(self.build_prod(item));
        c.push(self.build_imposto(item));
        format!("<det nItem=\"{n_item}\">{}</det>", c.join(""))
    }

    fn build_prod(&self, item: &ItemBuild) -> String {
        let p = &item.prod;
        let mut c = Vec::new();
        add_child(&mut c, "cProd", p.get("cProd").map(|s| s.as_str()));
        add_child_str(
            &mut c,
            "cEAN",
            p.get("cEAN").map(|s| s.as_str()).unwrap_or("SEM GTIN"),
        );
        add_child(&mut c, "xProd", p.get("xProd").map(|s| s.as_str()));
        add_child(&mut c, "NCM", p.get("NCM").map(|s| s.as_str()));
        if let Some(cest) = &item.cest {
            if let Some(v) = cest.get("CEST") {
                add_child_str(&mut c, "CEST", v);
            }
        }
        if let Some(v) = p.get("cBenef") {
            if !v.is_empty() {
                add_child_str(&mut c, "cBenef", v);
            }
        }
        if let Some(v) = p.get("EXTIPI") {
            if !v.is_empty() {
                add_child_str(&mut c, "EXTIPI", v);
            }
        }
        add_child(&mut c, "CFOP", p.get("CFOP").map(|s| s.as_str()));
        add_child(&mut c, "uCom", p.get("uCom").map(|s| s.as_str()));
        add_child(&mut c, "qCom", p.get("qCom").map(|s| s.as_str()));
        add_child(&mut c, "vUnCom", p.get("vUnCom").map(|s| s.as_str()));
        add_child(&mut c, "vProd", p.get("vProd").map(|s| s.as_str()));
        add_child_str(
            &mut c,
            "cEANTrib",
            p.get("cEANTrib").map(|s| s.as_str()).unwrap_or("SEM GTIN"),
        );
        add_child(&mut c, "uTrib", p.get("uTrib").map(|s| s.as_str()));
        add_child(&mut c, "qTrib", p.get("qTrib").map(|s| s.as_str()));
        add_child(&mut c, "vUnTrib", p.get("vUnTrib").map(|s| s.as_str()));
        if let Some(v) = p.get("vFrete") {
            if !v.is_empty() {
                add_child_str(&mut c, "vFrete", v);
            }
        }
        if let Some(v) = p.get("vSeg") {
            if !v.is_empty() {
                add_child_str(&mut c, "vSeg", v);
            }
        }
        if let Some(v) = p.get("vDesc") {
            if !v.is_empty() {
                add_child_str(&mut c, "vDesc", v);
            }
        }
        if let Some(v) = p.get("vOutro") {
            if !v.is_empty() {
                add_child_str(&mut c, "vOutro", v);
            }
        }
        let ind_tot = p.get("indTot").map(|s| s.as_str()).unwrap_or("1");
        add_child_str(&mut c, "indTot", ind_tot);
        if let Some(v) = p.get("xPed") {
            if !v.is_empty() {
                add_child_str(&mut c, "xPed", v);
            }
        }
        if let Some(v) = p.get("nItemPed") {
            if !v.is_empty() {
                add_child_str(&mut c, "nItemPed", v);
            }
        }

        // gCred
        if let Some(gc) = &item.g_cred {
            let mut gcc = Vec::new();
            add_child(
                &mut gcc,
                "cCredPresumido",
                gc.get("cCredPresumido").map(|s| s.as_str()),
            );
            add_child(
                &mut gcc,
                "pCredPresumido",
                gc.get("pCredPresumido").map(|s| s.as_str()),
            );
            add_child(
                &mut gcc,
                "vCredPresumido",
                gc.get("vCredPresumido").map(|s| s.as_str()),
            );
            c.push(xml_tag("gCred", &gcc.join("")));
        }

        xml_tag("prod", &c.join(""))
    }

    fn build_imposto(&self, item: &ItemBuild) -> String {
        let mut c = Vec::new();
        if !item.v_tot_trib.is_empty() {
            add_child_str(&mut c, "vTotTrib", &item.v_tot_trib);
        }
        if item.icms_data.is_some() {
            c.push(self.build_icms(item));
        }
        if item.ipi_header.is_some() || !item.ipi_cst.is_empty() {
            c.push(self.build_ipi(item));
        }
        if !item.pis_cst.is_empty() {
            c.push(self.build_pis(item));
        }
        if !item.cofins_cst.is_empty() {
            c.push(self.build_cofins(item));
        }
        xml_tag("imposto", &c.join(""))
    }

    fn build_icms(&self, item: &ItemBuild) -> String {
        let d = match &item.icms_data {
            Some(d) => d,
            None => return String::new(),
        };
        let cst = d
            .get("CST")
            .or_else(|| d.get("CSOSN"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let group_tag = icms_group_tag(cst);

        let mut ic = Vec::new();
        add_child(&mut ic, "orig", d.get("orig").map(|s| s.as_str()));
        if let Some(v) = d.get("CST") {
            add_child_str(&mut ic, "CST", v);
        }
        if let Some(v) = d.get("CSOSN") {
            add_child_str(&mut ic, "CSOSN", v);
        }
        for &field in &[
            "modBC",
            "pRedBC",
            "vBC",
            "pICMS",
            "vICMS",
            "vICMSOp",
            "pDif",
            "vICMSDif",
            "vBCFCP",
            "pFCP",
            "vFCP",
            "modBCST",
            "pMVAST",
            "pRedBCST",
            "vBCST",
            "pICMSST",
            "vICMSST",
            "vBCFCPST",
            "pFCPST",
            "vFCPST",
            "vICMSDeson",
            "motDesICMS",
            "vBCSTRet",
            "pST",
            "vICMSSTRet",
            "vICMSSubstituto",
            "vBCFCPSTRet",
            "pFCPSTRet",
            "vFCPSTRet",
            "pRedBCEfet",
            "vBCEfet",
            "pICMSEfet",
            "vICMSEfet",
            "pBCOp",
            "UFST",
            "pCredSN",
            "vCredICMSSN",
            "indDeduzDeson",
        ] {
            if let Some(v) = d.get(field) {
                if !v.is_empty() {
                    add_child_str(&mut ic, field, v);
                }
            }
        }

        xml_tag("ICMS", &xml_tag(&group_tag, &ic.join("")))
    }

    fn build_ipi(&self, item: &ItemBuild) -> String {
        let mut c = Vec::new();
        if let Some(h) = &item.ipi_header {
            if let Some(v) = h.get("qSelo") {
                if !v.is_empty() {
                    add_child_str(&mut c, "qSelo", v);
                }
            }
            if let Some(v) = h.get("cEnq") {
                if !v.is_empty() {
                    add_child_str(&mut c, "cEnq", v);
                }
            }
        }

        let cst = &item.ipi_cst;
        let trib_csts = ["00", "49", "50", "99"];
        if trib_csts.contains(&cst.as_str()) {
            let mut tc = Vec::new();
            add_child_str(&mut tc, "CST", cst);
            if !item.ipi_v_bc.is_empty() {
                add_child_str(&mut tc, "vBC", &item.ipi_v_bc);
            }
            if !item.ipi_p_ipi.is_empty() {
                add_child_str(&mut tc, "pIPI", &item.ipi_p_ipi);
            }
            if !item.ipi_v_ipi.is_empty() {
                add_child_str(&mut tc, "vIPI", &item.ipi_v_ipi);
            }
            c.push(xml_tag("IPITrib", &tc.join("")));
        } else if !cst.is_empty() {
            c.push(xml_tag("IPINT", &format!("<CST>{cst}</CST>")));
        }

        xml_tag("IPI", &c.join(""))
    }

    fn build_pis(&self, item: &ItemBuild) -> String {
        let cst = &item.pis_cst;
        let aliq_csts = ["01", "02"];
        let inner_tag = if aliq_csts.contains(&cst.as_str()) {
            "PISAliq"
        } else {
            "PISOutr"
        };

        let mut c = Vec::new();
        add_child_str(&mut c, "CST", cst);
        if !item.pis_v_bc.is_empty() {
            add_child_str(&mut c, "vBC", &item.pis_v_bc);
        }
        if !item.pis_p_pis.is_empty() {
            add_child_str(&mut c, "pPIS", &item.pis_p_pis);
        }
        if !item.pis_v_pis.is_empty() {
            add_child_str(&mut c, "vPIS", &item.pis_v_pis);
        }

        xml_tag("PIS", &xml_tag(inner_tag, &c.join("")))
    }

    fn build_cofins(&self, item: &ItemBuild) -> String {
        let cst = &item.cofins_cst;
        let aliq_csts = ["01", "02"];
        let inner_tag = if aliq_csts.contains(&cst.as_str()) {
            "COFINSAliq"
        } else {
            "COFINSOutr"
        };

        let mut c = Vec::new();
        add_child_str(&mut c, "CST", cst);
        if !item.cofins_v_bc.is_empty() {
            add_child_str(&mut c, "vBC", &item.cofins_v_bc);
        }
        if !item.cofins_p_cofins.is_empty() {
            add_child_str(&mut c, "pCOFINS", &item.cofins_p_cofins);
        }
        if !item.cofins_v_cofins.is_empty() {
            add_child_str(&mut c, "vCOFINS", &item.cofins_v_cofins);
        }

        xml_tag("COFINS", &xml_tag(inner_tag, &c.join("")))
    }

    fn build_total(&self) -> String {
        let t = &self.totals_fields;
        let mut c = Vec::new();
        for &field in &[
            "vBC",
            "vICMS",
            "vICMSDeson",
            "vFCP",
            "vBCST",
            "vST",
            "vFCPST",
            "vFCPSTRet",
            "vProd",
            "vFrete",
            "vSeg",
            "vDesc",
            "vII",
            "vIPI",
            "vIPIDevol",
            "vPIS",
            "vCOFINS",
            "vOutro",
            "vNF",
        ] {
            add_child(&mut c, field, t.get(field).map(|s| s.as_str()));
        }
        if let Some(v) = t.get("vTotTrib") {
            if !v.is_empty() {
                add_child_str(&mut c, "vTotTrib", v);
            }
        }
        xml_tag("total", &xml_tag("ICMSTot", &c.join("")))
    }

    fn build_transp(&self) -> String {
        let mut c = Vec::new();
        add_child(
            &mut c,
            "modFrete",
            self.transp_fields.get("modFrete").map(|s| s.as_str()),
        );

        if let Some(t) = &self.transporta_fields {
            let mut tc = Vec::new();
            if let Some(v) = t.get("CNPJ") {
                add_child_str(&mut tc, "CNPJ", v);
            }
            if let Some(v) = t.get("CPF") {
                add_child_str(&mut tc, "CPF", v);
            }
            add_child(&mut tc, "xNome", t.get("xNome").map(|s| s.as_str()));
            if let Some(v) = t.get("IE") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "IE", v);
                }
            }
            if let Some(v) = t.get("xEnder") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "xEnder", v);
                }
            }
            if let Some(v) = t.get("xMun") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "xMun", v);
                }
            }
            if let Some(v) = t.get("UF") {
                if !v.is_empty() {
                    add_child_str(&mut tc, "UF", v);
                }
            }
            c.push(xml_tag("transporta", &tc.join("")));
        }

        for vol in &self.volumes {
            let mut vc = Vec::new();
            if let Some(v) = vol.get("qVol") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "qVol", v);
                }
            }
            if let Some(v) = vol.get("esp") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "esp", v);
                }
            }
            if let Some(v) = vol.get("marca") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "marca", v);
                }
            }
            if let Some(v) = vol.get("nVol") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "nVol", v);
                }
            }
            if let Some(v) = vol.get("pesoL") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "pesoL", v);
                }
            }
            if let Some(v) = vol.get("pesoB") {
                if !v.is_empty() {
                    add_child_str(&mut vc, "pesoB", v);
                }
            }
            c.push(xml_tag("vol", &vc.join("")));
        }

        xml_tag("transp", &c.join(""))
    }

    fn build_cobr(&self) -> String {
        let mut c = Vec::new();
        if let Some(f) = &self.fat_fields {
            let mut fc = Vec::new();
            add_child(&mut fc, "nFat", f.get("nFat").map(|s| s.as_str()));
            add_child(&mut fc, "vOrig", f.get("vOrig").map(|s| s.as_str()));
            add_child(&mut fc, "vDesc", f.get("vDesc").map(|s| s.as_str()));
            add_child(&mut fc, "vLiq", f.get("vLiq").map(|s| s.as_str()));
            c.push(xml_tag("fat", &fc.join("")));
        }
        for dup in &self.dup_items {
            let mut dc = Vec::new();
            add_child(&mut dc, "nDup", dup.get("nDup").map(|s| s.as_str()));
            add_child(&mut dc, "dVenc", dup.get("dVenc").map(|s| s.as_str()));
            add_child(&mut dc, "vDup", dup.get("vDup").map(|s| s.as_str()));
            c.push(xml_tag("dup", &dc.join("")));
        }
        xml_tag("cobr", &c.join(""))
    }

    fn build_pag(&self) -> String {
        let mut c = Vec::new();
        for dp in &self.det_pag_list {
            let mut dc = Vec::new();
            if let Some(v) = dp.get("indPag") {
                if !v.is_empty() {
                    add_child_str(&mut dc, "indPag", v);
                }
            }
            add_child(&mut dc, "tPag", dp.get("tPag").map(|s| s.as_str()));
            add_child(&mut dc, "vPag", dp.get("vPag").map(|s| s.as_str()));
            c.push(xml_tag("detPag", &dc.join("")));
        }
        if let Some(pf) = &self.pag_fields {
            if let Some(v) = pf.get("vTroco") {
                if !v.is_empty() {
                    add_child_str(&mut c, "vTroco", v);
                }
            }
        }
        xml_tag("pag", &c.join(""))
    }

    fn build_inf_adic(&self) -> String {
        let mut c = Vec::new();
        if let Some(v) = self.inf_adic_fields.get("infAdFisco") {
            if !v.is_empty() {
                add_child_str(&mut c, "infAdFisco", v);
            }
        }
        if let Some(v) = self.inf_adic_fields.get("infCpl") {
            if !v.is_empty() {
                add_child_str(&mut c, "infCpl", v);
            }
        }
        xml_tag("infAdic", &c.join(""))
    }
}

fn icms_group_tag(cst: &str) -> String {
    match cst {
        "00" => "ICMS00".into(),
        "10" => "ICMS10".into(),
        "20" => "ICMS20".into(),
        "30" => "ICMS30".into(),
        "40" | "41" | "50" => "ICMS40".into(),
        "51" => "ICMS51".into(),
        "60" => "ICMS60".into(),
        "70" => "ICMS70".into(),
        "90" => "ICMS90".into(),
        _ => format!("ICMS{cst}"),
    }
}

// ── TXT Structure Definitions ───────────────────────────────────────────────

macro_rules! structure {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut map = HashMap::new();
        $(map.insert($key, $val);)*
        map
    }};
}

fn structure_310() -> HashMap<&'static str, &'static str> {
    structure! {
        "NOTAFISCAL" => "NOTAFISCAL|1|",
        "A" => "A|versao|Id|pk_nItem|",
        "B" => "B|cUF|cNF|natOp|indPag|mod|serie|nNF|dhEmi|dhSaiEnt|tpNF|idDest|cMunFG|tpImp|tpEmis|cDV|tpAmb|finNFe|indFinal|indPres|procEmi|verProc|dhCont|xJust|",
        "BA" => "BA|",
        "BA02" => "BA02|refNFe|",
        "C" => "C|xNome|xFant|IE|IEST|IM|CNAE|CRT|",
        "C02" => "C02|CNPJ|",
        "C02A" => "C02a|cpf|",
        "C05" => "C05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "E" => "E|xNome|indIEDest|IE|ISUF|IM|email|",
        "E02" => "E02|CNPJ|",
        "E03" => "E03|CPF|",
        "E03A" => "E03a|idEstrangeiro|",
        "E05" => "E05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "H" => "H|item|infAdProd|",
        "I" => "I|cProd|cEAN|xProd|NCM|EXTIPI|CFOP|uCom|qCom|vUnCom|vProd|cEANTrib|uTrib|qTrib|vUnTrib|vFrete|vSeg|vDesc|vOutro|indTot|xPed|nItemPed|nFCI|",
        "I05C" => "I05C|CEST|",
        "M" => "M|vTotTrib|",
        "N" => "N|",
        "N02" => "N02|orig|CST|modBC|vBC|pICMS|vICMS|",
        "N03" => "N03|orig|CST|modBC|vBC|pICMS|vICMS|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|",
        "N04" => "N04|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vICMSDeson|motDesICMS|",
        "N05" => "N05|orig|CST|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vICMSDeson|motDesICMS|",
        "N06" => "N06|orig|CST|vICMSDeson|motDesICMS|",
        "N07" => "N07|orig|CST|modBC|pRedBC|vBC|pICMS|vICMSOp|pDif|vICMSDif|vICMS|",
        "N08" => "N08|orig|CST|vBCSTRet|vICMSSTRet|",
        "N09" => "N09|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vICMSDeson|motDesICMS|",
        "N10" => "N10|orig|CST|modBC|vBC|pRedBC|pICMS|vICMS|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vICMSDeson|motDesICMS|",
        "O" => "O|clEnq|CNPJProd|cSelo|qSelo|cEnq|",
        "O07" => "O07|CST|vIPI|",
        "O08" => "O08|CST|",
        "O10" => "O10|vBC|pIPI|",
        "Q" => "Q|",
        "Q02" => "Q02|CST|vBC|pPIS|vPIS|",
        "Q04" => "Q04|CST|",
        "S" => "S|",
        "S02" => "S02|CST|vBC|pCOFINS|vCOFINS|",
        "S04" => "S04|CST|",
        "W" => "W|",
        "W02" => "W02|vBC|vICMS|vICMSDeson|vBCST|vST|vProd|vFrete|vSeg|vDesc|vII|vIPI|vPIS|vCOFINS|vOutro|vNF|vTotTrib|",
        "X" => "X|modFrete|",
        "X03" => "X03|xNome|IE|xEnder|xMun|UF|",
        "X04" => "X04|CNPJ|",
        "X05" => "X05|CPF|",
        "X26" => "X26|qVol|esp|marca|nVol|pesoL|pesoB|",
        "Y" => "Y|",
        "Y02" => "Y02|nFat|vOrig|vDesc|vLiq|",
        "Y07" => "Y07|nDup|dVenc|vDup|",
        "YA" => "YA|tPag|vPag|CNPJ|tBand|cAut|tpIntegra|",
        "Z" => "Z|infAdFisco|infCpl|"
    }
}

fn structure_400() -> HashMap<&'static str, &'static str> {
    structure! {
        "NOTAFISCAL" => "NOTAFISCAL|qtd|",
        "A" => "A|versao|Id|pk_nItem|",
        "B" => "B|cUF|cNF|natOp|mod|serie|nNF|dhEmi|dhSaiEnt|tpNF|idDest|cMunFG|cMunFGIBS|tpImp|tpEmis|cDV|tpAmb|finNFe|indFinal|indPres|indIntermed|procEmi|verProc|dhCont|xJust|",
        "BA" => "BA|",
        "BA02" => "BA02|refNFe|",
        "BB" => "BB|",
        "BB02" => "BB02|refNFe|",
        "C" => "C|xNome|xFant|IE|IEST|IM|CNAE|CRT|",
        "C02" => "C02|CNPJ|",
        "C02A" => "C02a|CPF|",
        "C05" => "C05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "D" => "D|CNPJ|xOrgao|matr|xAgente|fone|UF|nDAR|dEmi|vDAR|repEmi|dPag|",
        "E" => "E|xNome|indIEDest|IE|ISUF|IM|email|",
        "E02" => "E02|CNPJ|",
        "E03" => "E03|CPF|",
        "E03A" => "E03a|idEstrangeiro|",
        "E05" => "E05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "H" => "H|item|infAdProd|",
        "I" => "I|cProd|cEAN|xProd|NCM|cBenef|EXTIPI|CFOP|uCom|qCom|vUnCom|vProd|cEANTrib|uTrib|qTrib|vUnTrib|vFrete|vSeg|vDesc|vOutro|indTot|xPed|nItemPed|nFCI|indBemMovelUsado|",
        "I05A" => "I05A|NVE|",
        "I05C" => "I05C|CEST|indEscala|CNPJFab|",
        "I05G" => "I05C|cCredPresumido|pCredPresumido|vCredPresumido|",
        "I80" => "I80|nLote|qLote|dFab|dVal|cAgreg|",
        "M" => "M|vTotTrib|",
        "N" => "N|",
        "N02" => "N02|orig|CST|modBC|vBC|pICMS|vICMS|pFCP|vFCP|",
        "N03" => "N03|orig|CST|modBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|",
        "N04" => "N04|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|vICMSDeson|motDesICMS|",
        "N05" => "N05|orig|CST|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N06" => "N06|orig|CST|vICMSDeson|motDesICMS|indDeduzDeson|",
        "N07" => "N07|orig|CST|modBC|pRedBC|vBC|pICMS|vICMSOp|pDif|vICMSDif|vICMS|vBCFCP|pFCP|vFCP|",
        "N08" => "N08|orig|CST|vBCSTRet|pST|vICMSSTRet|vBCFCPSTRet|pFCPSTRet|vFCPSTRet|pRedBCEfet|vBCEfet|pICMSEfet|vICMSEfet|vICMSSubstituto|",
        "N09" => "N09|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N10" => "N10|orig|CST|modBC|vBC|pRedBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "O" => "O|CNPJProd|cSelo|qSelo|cEnq|",
        "O07" => "O07|CST|vIPI|",
        "O08" => "O08|CST|",
        "O10" => "O10|vBC|pIPI|",
        "Q" => "Q|",
        "Q02" => "Q02|CST|vBC|pPIS|vPIS|",
        "Q04" => "Q04|CST|",
        "S" => "S|",
        "S02" => "S02|CST|vBC|pCOFINS|vCOFINS|",
        "S04" => "S04|CST|",
        "W" => "W|",
        "W02" => "W02|vBC|vICMS|vICMSDeson|vFCP|vBCST|vST|vFCPST|vFCPSTRet|vProd|vFrete|vSeg|vDesc|vII|vIPI|vIPIDevol|vPIS|vCOFINS|vOutro|vNF|vTotTrib|vFCPUFDest|vICMSUFDest|vICMSUFRemet|",
        "X" => "X|modFrete|",
        "X03" => "X03|xNome|IE|xEnder|xMun|UF|",
        "X04" => "X04|CNPJ|",
        "X05" => "X05|CPF|",
        "X26" => "X26|qVol|esp|marca|nVol|pesoL|pesoB|",
        "Y" => "Y|vTroco|",
        "Y02" => "Y02|nFat|vOrig|vDesc|vLiq|",
        "Y07" => "Y07|nDup|dVenc|vDup|",
        "YA" => "YA|indPag|tPag|vPag|CNPJ|tBand|cAut|tpIntegra|xPag|",
        "Z" => "Z|infAdFisco|infCpl|",
        "ZD" => "ZD|CNPJ|xContato|email|fone|CSRT|idCSRT|"
    }
}

fn structure_400_v12() -> HashMap<&'static str, &'static str> {
    structure! {
        "NOTAFISCAL" => "NOTAFISCAL|qtd|",
        "A" => "A|versao|Id|pk_nItem|",
        "B" => "B|cUF|cNF|natOp|mod|serie|nNF|dhEmi|dhSaiEnt|tpNF|idDest|cMunFG|tpImp|tpEmis|cDV|tpAmb|finNFe|indFinal|indPres|indIntermed|procEmi|verProc|dhCont|xJust|",
        "BA" => "BA|",
        "BA02" => "BA02|refNFe|",
        "C" => "C|xNome|xFant|IE|IEST|IM|CNAE|CRT|",
        "C02" => "C02|CNPJ|",
        "C02A" => "C02a|CPF|",
        "C05" => "C05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "D" => "D|CNPJ|xOrgao|matr|xAgente|fone|UF|nDAR|dEmi|vDAR|repEmi|dPag|",
        "E" => "E|xNome|indIEDest|IE|ISUF|IM|email|",
        "E02" => "E02|CNPJ|",
        "E03" => "E03|CPF|",
        "E03A" => "E03a|idEstrangeiro|",
        "E05" => "E05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "F" => "F|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|email|IE|",
        "F02" => "F02|CNPJ|",
        "F02A" => "F02a|CPF|",
        "F02B" => "F02b|xNome|",
        "G" => "G|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|email|IE|",
        "G02" => "G02|CNPJ|",
        "G02A" => "G02a|CPF|",
        "G02B" => "G02b|xNome|",
        "GA" => "GA|",
        "GA02" => "GA02|CNPJ|",
        "GA03" => "GA03|CPF|",
        "H" => "H|item|infAdProd|",
        "I" => "I|cProd|cEAN|xProd|NCM|cBenef|EXTIPI|CFOP|uCom|qCom|vUnCom|vProd|cEANTrib|uTrib|qTrib|vUnTrib|vFrete|vSeg|vDesc|vOutro|indTot|xPed|nItemPed|nFCI|",
        "I05A" => "I05A|NVE|",
        "I05C" => "I05C|CEST|indEscala|CNPJFab|",
        "I05G" => "I05C|cCredPresumido|pCredPresumido|vCredPresumido|",
        "I18" => "I18|nDI|dDI|xLocDesemb|UFDesemb|dDesemb|tpViaTransp|vAFRMM|tpIntermedio|CNPJ|UFTerceiro|cExportador|",
        "I25" => "I25|nAdicao|nSeqAdic|cFabricante|vDescDI|nDraw|",
        "I50" => "I50|nDraw|",
        "I52" => "I52|nRE|chNFe|qExport|",
        "I80" => "I80|nLote|qLote|dFab|dVal|cAgreg|",
        "JA" => "JA|tpOp|chassi|cCor|xCor|pot|cilin|pesoL|pesoB|nSerie|tpComb|nMotor|CMT|dist|anoMod|anoFab|tpPint|tpVeic|espVeic|VIN|condVeic|cMod|cCorDENATRAN|lota|tpRest|",
        "K" => "K|cProdANVISA|vPMC|xMotivoIsencao|",
        "L" => "L|tpArma|nSerie|nCano|descr|",
        "LA" => "LA|cProdANP|descANP|pGLP|pGNn|pGNi|vPart|CODIF|qTemp|UFCons|",
        "LB" => "LB|nRECOPI|",
        "M" => "M|vTotTrib|",
        "N" => "N|",
        "N02" => "N02|orig|CST|modBC|vBC|pICMS|vICMS|pFCP|vFCP|",
        "N03" => "N03|orig|CST|modBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|",
        "N04" => "N04|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|vICMSDeson|motDesICMS|",
        "N05" => "N05|orig|CST|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N06" => "N06|orig|CST|vICMSDeson|motDesICMS|indDeduzDeson|",
        "N07" => "N07|orig|CST|modBC|pRedBC|vBC|pICMS|vICMSOp|pDif|vICMSDif|vICMS|vBCFCP|pFCP|vFCP|",
        "N08" => "N08|orig|CST|vBCSTRet|pST|vICMSSTRet|vBCFCPSTRet|pFCPSTRet|vFCPSTRet|pRedBCEfet|vBCEfet|pICMSEfet|vICMSEfet|vICMSSubstituto|",
        "N09" => "N09|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N10" => "N10|orig|CST|modBC|vBC|pRedBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N10A" => "N10a|orig|CST|modBC|vBC|pRedBC|pICMS|vICMS|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|pBCOp|UFST|",
        "N10B" => "N10b|orig|CST|vBCSTRet|vICMSSTRet|vBCSTDest|vICMSSTDest|vBCFCPSTRet|pFCPSTRet|vFCPSTRet|pST|vICMSSubstituto|pRedBCEfet|vBCEfet|pICMSEfet|vICMSEfet|",
        "N10C" => "N10c|orig|CSOSN|pCredSN|vCredICMSSN|",
        "N10D" => "N10d|orig|CSOSN|",
        "N10E" => "N10e|orig|CSOSN|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|pCredSN|vCredICMSSN|",
        "N10F" => "N10f|orig|CSOSN|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|",
        "N10G" => "N10g|orig|CSOSN|vBCSTRet|pST|vICMSSTRet|vBCFCPSTRet|pFCPSTRet|vFCPSTRet|pRedBCEfet|vBCEfet|pICMSEfet|vICMSEfet|vICMSSubstituto|",
        "N10H" => "N10h|orig|CSOSN|modBC|vBC|pRedBC|pICMS|vICMS|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|pCredSN|vCredICMSSN|",
        "NA" => "NA|vBCUFDest|vBCFCPUFDest|pFCPUFDest|pICMSUFDest|pICMSInter|pICMSInterPart|vFCPUFDest|vICMSUFDest|vICMSUFRemet|",
        "O" => "O|CNPJProd|cSelo|qSelo|cEnq|",
        "O07" => "O07|CST|vIPI|",
        "O08" => "O08|CST|",
        "O10" => "O10|vBC|pIPI|",
        "O11" => "O11|qUnid|vUnid|",
        "P" => "P|vBC|vDespAdu|vII|vIOF|",
        "Q" => "Q|",
        "Q02" => "Q02|CST|vBC|pPIS|vPIS|",
        "Q03" => "Q03|CST|qBCProd|vAliqProd|vPIS|",
        "Q04" => "Q04|CST|",
        "Q05" => "Q05|CST|vPIS|",
        "R" => "R|vPIS|",
        "S" => "S|",
        "S02" => "S02|CST|vBC|pCOFINS|vCOFINS|",
        "S03" => "S03|CST|qBCProd|vAliqProd|vCOFINS|",
        "S04" => "S04|CST|",
        "S05" => "S05|CST|vCOFINS|",
        "T" => "T|vCOFINS|",
        "U" => "U|vBC|vAliq|vISSQN|cMunFG|cListServ|vDeducao|vOutro|vDescIncond|vDescCond|vISSRet|indISS|cServico|cMun|cPais|nProcesso|indIncentivo|",
        "UA" => "UA|pDevol|vIPIDevol|",
        "W" => "W|",
        "W02" => "W02|vBC|vICMS|vICMSDeson|vFCP|vBCST|vST|vFCPST|vFCPSTRet|vProd|vFrete|vSeg|vDesc|vII|vIPI|vIPIDevol|vPIS|vCOFINS|vOutro|vNF|vTotTrib|vFCPUFDest|vICMSUFDest|vICMSUFRemet|",
        "W17" => "W17|vServ|vBC|vISS|vPIS|vCOFINS|dCompet|vDeducao|vOutro|vDescIncond|vDescCond|vISSRet|cRegTrib|",
        "W23" => "W23|vRetPIS|vRetCOFINS|vRetCSLL|vBCIRRF|vIRRF|vBCRetPrev|vRetPrev|",
        "X" => "X|modFrete|",
        "X03" => "X03|xNome|IE|xEnder|xMun|UF|",
        "X04" => "X04|CNPJ|",
        "X05" => "X05|CPF|",
        "X11" => "X11|vServ|vBCRet|pICMSRet|vICMSRet|CFOP|cMunFG|",
        "X18" => "X18|placa|UF|RNTC|",
        "X22" => "X22|placa|UF|RNTC|",
        "X26" => "X26|qVol|esp|marca|nVol|pesoL|pesoB|",
        "X33" => "X33|nLacre|",
        "Y" => "Y|vTroco|",
        "Y02" => "Y02|nFat|vOrig|vDesc|vLiq|",
        "Y07" => "Y07|nDup|dVenc|vDup|",
        "YA" => "YA|indPag|tPag|vPag|CNPJ|tBand|cAut|tpIntegra|xPag|",
        "YB" => "YB|CNPJ|idCadIntTran|",
        "Z" => "Z|infAdFisco|infCpl|",
        "Z04" => "Z04|xCampo|xTexto|",
        "Z07" => "Z07|xCampo|xTexto|",
        "Z10" => "Z10|nProc|indProc|",
        "ZA" => "ZA|UFSaidaPais|xLocExporta|xLocDespacho|",
        "ZD" => "ZD|CNPJ|xContato|email|fone|CSRT|idCSRT|",
        "ZX01" => "ZX01|qrcode|urlChave|"
    }
}

fn structure_400_v13() -> HashMap<&'static str, &'static str> {
    // v1.3 is a superset of v1.2 with extra fields in B and I
    let mut m = structure_400_v12();
    m.insert("B", "B|cUF|cNF|natOp|mod|serie|nNF|dhEmi|dhSaiEnt|tpNF|idDest|cMunFG|tpImp|tpEmis|cDV|tpAmb|finNFe|indFinal|indPres|indIntermed|procEmi|verProc|dhCont|xJust|indPag|dPrevEntrega|tpNFDebito|tpNFCredito|cMunFGIBS|");
    m.insert("B31", "B31|tpEnteGov|pRedutor|tpOperGov|");
    m.insert("BB01", "BB01|refNFe|");
    m.insert("I", "I|cProd|cEAN|xProd|NCM|cBenef|EXTIPI|CFOP|uCom|qCom|vUnCom|vProd|cEANTrib|uTrib|qTrib|vUnTrib|vFrete|vSeg|vDesc|vOutro|indTot|xPed|nItemPed|nFCI|vItem|tpCredPresIBSZFM|indBemMovelUsado|");
    m
}

fn structure_400_sebrae() -> HashMap<&'static str, &'static str> {
    structure! {
        "NOTAFISCAL" => "NOTAFISCAL|qtd|",
        "A" => "A|versao|Id|",
        "B" => "B|cUF|cNF|natOp|mod|serie|nNF|dhEmi|dhSaiEnt|tpNF|idDest|cMunFG|tpImp|tpEmis|cDV|tpAmb|finNFe|indFinal|indPres|indIntermed|procEmi|verProc|dhCont|xJust|",
        "BA" => "BA|",
        "BA02" => "BA02|refNFe|",
        "C" => "C|xNome|xFant|IE|IEST|IM|CNAE|CRT|",
        "C02" => "C02|CNPJ|",
        "C02A" => "C02a|CPF|",
        "C05" => "C05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "D" => "D|CNPJ|xOrgao|matr|xAgente|fone|UF|nDAR|dEmi|vDAR|repEmi|dPag|",
        "E" => "E|xNome|indIEDest|IE|ISUF|IM|email|",
        "E02" => "E02|CNPJ|",
        "E03" => "E03|CPF|",
        "E03A" => "E03a|idEstrangeiro|",
        "E05" => "E05|xLgr|nro|xCpl|xBairro|cMun|xMun|UF|CEP|cPais|xPais|fone|",
        "H" => "H|item|infAdProd|",
        "I" => "I|cProd|cEAN|xProd|NCM|EXTIPI|CFOP|uCom|qCom|vUnCom|vProd|cEANTrib|uTrib|qTrib|vUnTrib|vFrete|vSeg|vDesc|vOutro|indTot|xPed|nItemPed|nFCI|",
        "I05C" => "I05C|CEST|indEscala|CNPJFab|cBenef|",
        "I80" => "I80|nLote|qLote|dFab|dVal|cAgreg|",
        "M" => "M|vTotTrib|",
        "N" => "N|",
        "N02" => "N02|orig|CST|modBC|vBC|pICMS|vICMS|pFCP|vFCP|",
        "N03" => "N03|orig|CST|modBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|",
        "N04" => "N04|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|vICMSDeson|motDesICMS|",
        "N05" => "N05|orig|CST|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N06" => "N06|orig|CST|vICMSDeson|motDesICMS|indDeduzDeson|",
        "N07" => "N07|orig|CST|modBC|pRedBC|vBC|pICMS|vICMSOp|pDif|vICMSDif|vICMS|vBCFCP|pFCP|vFCP|",
        "N08" => "N08|orig|CST|vBCSTRet|pST|vICMSSubstituto|vICMSSTRet|vBCFCPSTRet|pFCPSTRet|vFCPSTRet|pRedBCEfet|vBCEfet|pICMSEfet|vICMSEfet|",
        "N09" => "N09|orig|CST|modBC|pRedBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "N10" => "N10|orig|CST|modBC|vBC|pRedBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS|",
        "O" => "O|CNPJProd|cSelo|qSelo|cEnq|",
        "O07" => "O07|CST|vIPI|",
        "O08" => "O08|CST|",
        "O10" => "O10|vBC|pIPI|",
        "Q" => "Q|",
        "Q02" => "Q02|CST|vBC|pPIS|vPIS|",
        "Q04" => "Q04|CST|",
        "S" => "S|",
        "S02" => "S02|CST|vBC|pCOFINS|vCOFINS|",
        "S04" => "S04|CST|",
        "W" => "W|",
        "W02" => "W02|vBC|vICMS|vICMSDeson|vFCP|vBCST|vST|vFCPST|vFCPSTRet|vProd|vFrete|vSeg|vDesc|vII|vIPI|vIPIDevol|vPIS|vCOFINS|vOutro|vNF|vTotTrib|",
        "W04C" => "W04c|vFCPUFDest|",
        "W04E" => "W04e|vICMSUFDest|",
        "W04G" => "W04g|vICMSUFRemet|",
        "X" => "X|modFrete|",
        "X03" => "X03|xNome|IE|xEnder|xMun|UF|",
        "X04" => "X04|CNPJ|",
        "X05" => "X05|CPF|",
        "X26" => "X26|qVol|esp|marca|nVol|pesoL|pesoB|",
        "Y" => "Y|",
        "Y02" => "Y02|nFat|vOrig|vDesc|vLiq|",
        "Y07" => "Y07|nDup|dVenc|vDup|",
        "YA" => "YA|vTroco|",
        "YA01" => "YA01|indPag|tPag|vPag|",
        "YA04" => "YA04|tpIntegra|CNPJ|tBand|cAut|",
        "Z" => "Z|infAdFisco|infCpl|",
        "ZD" => "ZD|CNPJ|xContato|email|fone|CSRT|idCSRT|"
    }
}
