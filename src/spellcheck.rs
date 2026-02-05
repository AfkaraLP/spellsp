use anyhow::anyhow;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use zspell::Dictionary;

use crate::{
    args::Language,
    data_dirs::{DATA_DIRECTORIES, read_or_create},
};

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

async fn get_from_wooorm(c: &reqwest::Client, url: impl AsRef<str>) -> anyhow::Result<String> {
    Ok(c.get(url.as_ref()).send().await?.text().await?)
}

async fn get_aff_file(client: &reqwest::Client, lang: Language) -> anyhow::Result<String> {
    let data_dir = &DATA_DIRECTORIES;
    let lang_cache_path = data_dir.data.join(lang.into_wooorm_dictionary_lang_str());
    let aff_path = lang_cache_path.join("index.aff");
    let aff = read_or_create(aff_path, async {
        let url = format!("{}/index.aff", lang.wooorm_dictionary_github_root());
        get_from_wooorm(client, url).await
    })
    .await?;

    Ok(aff)
}

async fn get_dic_file(client: &reqwest::Client, lang: Language) -> anyhow::Result<String> {
    let data_dir = &DATA_DIRECTORIES;
    let lang_cache_path = data_dir.data.join(lang.into_wooorm_dictionary_lang_str());
    let dic_path = lang_cache_path.join("index.dic");
    let dic = read_or_create(dic_path, async {
        let url = format!("{}/index.dic", lang.wooorm_dictionary_github_root());
        get_from_wooorm(client, url).await
    })
    .await?;

    Ok(dic)
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
