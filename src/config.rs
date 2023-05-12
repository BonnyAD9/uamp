pub struct Config {
    library_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            library_path: "/home/kubas/Music/".to_owned(),
        }
    }
}

impl Config {
    pub fn library_path(&self) -> &str {
        &self.library_path
    }
}
