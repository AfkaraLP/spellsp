use anyhow::anyhow;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use zspell::Dictionary;

use crate::args::Language;

pub async fn get_dict(
    client: &reqwest::Client,
    lang: Language,
) -> anyhow::Result<zspell::Dictionary> {
    let dict = get_dic_file(client, lang).await?;
    let aff = get_aff_file(client, lang).await?;

    zspell::builder()
        .config_str(&aff)
        .dict_str(&dict)
        .build()
        .map_err(|e| anyhow!("failed building dict: {e:#?}"))
}

async fn get_aff_file(client: &reqwest::Client, lang: Language) -> anyhow::Result<String> {
    let aff = client
        .get(format!("{}/index.aff", get_lang_path(lang)))
        .send()
        .await?
        .text()
        .await?;
    Ok(aff)
}

async fn get_dic_file(client: &reqwest::Client, lang: Language) -> anyhow::Result<String> {
    let dict = client
        .get(format!("{}/index.dic", get_lang_path(lang)))
        .send()
        .await?
        .text()
        .await?;
    Ok(dict)
}

pub fn get_lang_path(lang: Language) -> String {
    let lang_string = match lang {
        Language::En => "en",
        Language::Ru => "ru",
        Language::Bg => "bg",
        Language::Br => "br",
        Language::Ca => "ca",
        Language::CaValencia => "ca-valencia",
        Language::Cs => "cs",
        Language::Cy => "cy",
        Language::Da => "da",
        Language::De => "de",
        Language::DeAt => "de-AT",
        Language::DeCh => "de-CH",
        Language::El => "el",
        Language::ElPolyton => "el-polyton",
        Language::EnGb => "en-GB",
        Language::EnAu => "en-AU",
        Language::EnCa => "en-CA",
        Language::Eo => "eo",
        Language::Es => "es",
        Language::EsAr => "es-AR",
        Language::EsBo => "es-BO",
        Language::EsCl => "es-CL",
        Language::EsCo => "es-CO",
        Language::EsCr => "es-CR",
        Language::EsCu => "es-CU",
        Language::EsDo => "es-DO",
        Language::EsEc => "es-EC",
        Language::EsGt => "es-GT",
        Language::EsHn => "es-HN",
        Language::EsMx => "es-MX",
        Language::EsNi => "es-NI",
        Language::EsPa => "es-PA",
        Language::EsPe => "es-PE",
        Language::EsPh => "es-PH",
        Language::EsPr => "es-PR",
        Language::EsPy => "es-PY",
        Language::EsSv => "es-SV",
        Language::EsUs => "es-US",
        Language::EsUy => "es-UY",
        Language::EsVe => "es-VE",
        Language::Et => "et",
        Language::Eu => "eu",
        Language::Fa => "fa",
        Language::Fo => "fo",
        Language::Fr => "fr",
        Language::Fur => "fur",
        Language::Fy => "fy",
        Language::Ga => "ga",
        Language::Gd => "gd",
        Language::Gl => "gl",
        Language::He => "he",
        Language::Hr => "hr",
        Language::Hu => "hu",
        Language::Hy => "hy",
        Language::Hyw => "hyw",
        Language::Ia => "ia",
        Language::Ie => "ie",
        Language::Is => "is",
        Language::It => "it",
        Language::Ka => "ka",
        Language::Ko => "ko",
        Language::La => "la",
        Language::Lb => "lb",
        Language::Lt => "lt",
        Language::Ltg => "ltg",
        Language::Lv => "lv",
        Language::Mk => "mk",
        Language::Mn => "mn",
        Language::Nb => "nb",
        Language::Nds => "nds",
        Language::Ne => "ne",
        Language::Nl => "nl",
        Language::Nn => "nn",
        Language::Oc => "oc",
        Language::Pl => "pl",
        Language::Pt => "pt",
        Language::PtPt => "pt-PT",
        Language::Ro => "ro",
        Language::Rw => "rw",
        Language::Sk => "sk",
        Language::Sl => "sl",
        Language::Sr => "sr",
        Language::SrLatn => "sr-Latn",
        Language::Sv => "sv",
        Language::SvFi => "sv-FI",
        Language::Tk => "tk",
        Language::Tlh => "tlh",
        Language::TlhLatn => "tlh-Latn",
        Language::Tr => "tr",
        Language::Uk => "uk",
        Language::Vi => "vi",
    };
    format!(
        "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/{lang_string}"
    )
}

pub fn byte_to_position(byte: usize, reftext: impl AsRef<str>) -> Position {
    let text = reftext.as_ref();
    let mut remaining = byte;
    for (line_idx, line) in text.split_inclusive('\n').enumerate() {
        let line_bytes = line.len();
        if remaining <= line_bytes {
            return Position {
                line: u32::try_from(line_idx).unwrap_or(u32::MAX),
                character: u32::try_from(remaining).unwrap_or(u32::MAX),
            };
        }
        remaining -= line_bytes;
    }

    let last_line = text.lines().count().saturating_sub(1);
    let last_col = text.lines().last().map_or(0, str::len);
    Position {
        line: u32::try_from(last_line).unwrap_or(u32::MAX),
        character: u32::try_from(last_col).unwrap_or(u32::MAX),
    }
}

pub fn spellcheck_diagnostics(text: impl AsRef<str>, dict: &Dictionary) -> Vec<Diagnostic> {
    let errs = dict.check_indices(text.as_ref());
    let mut diags: Vec<Diagnostic> = Vec::new();
    for (index_byte, word) in errs {
        let diag = generated_typo_diagnostic(&text, index_byte, word);
        diags.push(diag);
    }
    diags
}

fn generated_typo_diagnostic(text: &impl AsRef<str>, index_byte: usize, word: &str) -> Diagnostic {
    let start_pos = byte_to_position(index_byte, text);
    let end_pos = byte_to_position(index_byte + word.len(), text);
    Diagnostic {
        range: Range {
            start: start_pos,
            end: end_pos,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("Spellcheck".to_string()),
        message: "Found Typo Here!".into(),
        related_information: None,
        tags: None,
        data: None,
    }
}
