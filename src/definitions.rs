use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::Position;

pub fn word_at_position(text: &str, pos: Position) -> Option<&str> {
    let line = text.lines().nth(pos.line as usize)?;
    if pos.character as usize >= line.len() {
        return None;
    }

    let mut start = pos.character as usize;
    while start > 0 {
        let prev_char = line.get(..start)?.chars().next_back()?;
        if !prev_char.is_alphanumeric() && prev_char != '_' {
            break;
        }
        start -= prev_char.len_utf8();
    }

    let mut end = pos.character as usize;
    while end < line.len() {
        let c = line.get(end..)?.chars().next()?;
        if !c.is_alphanumeric() && c != '_' {
            break;
        }
        end += c.len_utf8();
    }

    if start == end {
        None
    } else {
        Some(line.get(start..end)?)
    }
}

pub async fn get_definitions(word: impl AsRef<str>) -> anyhow::Result<Vec<Definition>> {
    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        word.as_ref()
    );
    let res = reqwest::get(url).await?.text().await?;
    let res: Vec<Definition> = serde_json::from_str(&res)?;
    Ok(res)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Definition {
    pub word: Option<String>,
    pub phonetic: Option<String>,
    pub origin: Option<String>,
    pub meanings: Vec<Meaning>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: Option<String>,

    pub definitions: Vec<WordDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordDef {
    pub definition: Option<String>,
    pub example: Option<String>,
}

#[cfg(test)]
pub mod tests {
    use crate::definitions::get_definitions;

    #[test]
    fn test_lookup() {
        tokio_test::block_on(async move {
            let _def = get_definitions("hello").await.unwrap();
        });
    }
}

impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let word = self.word.clone().unwrap_or("Unknown...".into());
        let pronounciation = self
            .clone()
            .phonetic
            .unwrap_or("no pronounciation found...".into());
        writeln!(f, "# {word} ({pronounciation})",)?;
        if let Some(origin) = &self.origin {
            write!(f, "origin: {origin}")?;
        }
        writeln!(f, "## Meanings: ")?;
        for meaning in &self.meanings {
            writeln!(f, "{meaning}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Meaning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pos) = self.part_of_speech.as_ref() {
            writeln!(f, "(_{pos}_): ")?;
            writeln!(f)?;
        }
        for definition in &self.definitions {
            writeln!(f, "{definition}")?;
        }
        Ok(())
    }
}
impl std::fmt::Display for WordDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(def) = &self.definition {
            writeln!(f, "- `Definition`: {def}")?;
        }
        if let Some(example) = &self.example {
            writeln!(f, "- `Example`: {example}")?;
        }

        Ok(())
    }
}
