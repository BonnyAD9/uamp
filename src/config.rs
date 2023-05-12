pub struct Config {
    pub library_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            library_path: "/home/kubas/Music/".to_owned(),
        }
    }
}
