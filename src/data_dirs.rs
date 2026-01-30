use anyhow::anyhow;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use dotenv::dotenv;

pub struct PublicDirectories {
    #[allow(unused)]
    pub cache: PathBuf,
    pub data: PathBuf,
    #[allow(unused)]
    pub config: PathBuf,
}

pub static DATA_DIRECTORIES: LazyLock<PublicDirectories> = LazyLock::new(|| {
    let cache = resolve_cache_dir().to_spellsp_dir();
    let config = resolve_config_dir().to_spellsp_dir();
    let data = resolve_data_dir().to_spellsp_dir();
    PublicDirectories {
        cache,
        data,
        config,
    }
});

pub fn resolve_cache_dir() -> PathBuf {
    dotenv().ok();

    if let Ok(xdg_cache) = dotenv::var("XDG_CACHE_HOME") {
        eprintln!("Got xdg cache directory");
        return PathBuf::from(xdg_cache);
    }
    if let Ok(home) = dotenv::var("HOME") {
        eprintln!("Got Home directory for cache");
        let cache_path = PathBuf::from(home);
        return cache_path.join(".cache");
    }

    PathBuf::from("./.cache")
}
pub fn resolve_config_dir() -> PathBuf {
    dotenv().ok();

    if let Ok(xdg_cache) = dotenv::var("XDG_CONFIG_HOME") {
        eprintln!("Got xdg cache directory");
        return PathBuf::from(xdg_cache);
    }
    if let Ok(home) = dotenv::var("HOME") {
        eprintln!("Got Home directory for cache");
        let cache_path = PathBuf::from(home);
        return cache_path.join(".config");
    }

    PathBuf::from("./.config")
}
pub fn resolve_data_dir() -> PathBuf {
    dotenv().ok();

    if let Ok(xdg_cache) = dotenv::var("XDG_DATA_HOME") {
        eprintln!("Got xdg cache directory");
        return PathBuf::from(xdg_cache);
    }
    if let Ok(home) = dotenv::var("HOME") {
        eprintln!("Got Home directory for cache");
        let cache_path = PathBuf::from(home);
        return cache_path.join(".local/share");
    }

    PathBuf::from("./.local/share")
}

trait ToSpellspDir {
    fn to_spellsp_dir(&self) -> Self;
}

impl ToSpellspDir for PathBuf {
    fn to_spellsp_dir(&self) -> Self {
        self.join("spellsp")
    }
}
pub async fn read_or_create<P, F>(path: P, f: F) -> anyhow::Result<String>
where
    F: Future<Output = anyhow::Result<String>>,
    P: AsRef<Path>,
{
    let file = std::fs::OpenOptions::new().read(true).open(&path);
    if let Ok(mut f) = file {
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(s)
    } else {
        let p = path.as_ref();
        let parent_dir = p.parent().ok_or(anyhow!("failed getting parent dir"))?;
        std::fs::create_dir_all(parent_dir)?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;
        let res = f.await?;
        file.write_all(res.as_bytes())?;

        Ok(res)
    }
}
