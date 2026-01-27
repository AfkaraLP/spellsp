use anyhow::anyhow;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use zspell::Dictionary;

pub async fn get_dict(
    client: &reqwest::Client,
    lang: impl AsRef<str>,
) -> anyhow::Result<zspell::Dictionary> {
    let dict = get_dic_file(client, &lang).await?;
    let aff = get_aff_file(client, lang).await?;

    zspell::builder()
        .config_str(&aff)
        .dict_str(&dict)
        .build()
        .map_err(|e| anyhow!("failed building dict: {e:#?}"))
}

async fn get_aff_file(client: &reqwest::Client, lang: impl AsRef<str>) -> anyhow::Result<String> {
    let aff = client
        .get(format!("{}/index.aff", get_lang_path(&lang)))
        .send()
        .await?
        .text()
        .await?;
    Ok(aff)
}

async fn get_dic_file(client: &reqwest::Client, lang: &impl AsRef<str>) -> anyhow::Result<String> {
    let dict = client
        .get(format!("{}/index.dic", get_lang_path(lang)))
        .send()
        .await?
        .text()
        .await?;
    Ok(dict)
}

pub fn get_lang_path(lang: impl AsRef<str>) -> String {
    format!(
        "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/{}",
        lang.as_ref()
    )
}

pub fn byte_to_position(byte: usize, reftext: impl AsRef<str>) -> Position {
    let text = reftext.as_ref();
    let mut remaining = byte;
    for (line_idx, line) in text.split_inclusive('\n').enumerate() {
        let line_bytes = line.len();
        if remaining <= line_bytes {
            return Position {
                line: line_idx as u32,
                character: remaining as u32,
            };
        }
        remaining -= line_bytes;
    }

    let last_line = text.lines().count().saturating_sub(1);
    let last_col = text.lines().last().map_or(0, str::len);
    Position {
        line: last_line as u32,
        character: last_col as u32,
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
