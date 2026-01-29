use std::{path::PathBuf, sync::LazyLock};

use dotenv::dotenv;

pub struct PublicDirectories {
    #[allow(unused)]
    pub cache: PathBuf,
    pub local_share: PathBuf,
    #[allow(unused)]
    pub config: PathBuf,
}

pub static DATA_DIRECTORIES: LazyLock<PublicDirectories> = LazyLock::new(|| {
    let cache = resolve_cache_dir().to_spellsp_dir();
    let config = resolve_config_dir().to_spellsp_dir();
    let local_share = resolve_data_dir().to_spellsp_dir();
    PublicDirectories {
        cache,
        local_share,
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
