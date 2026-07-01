use log::info;

use crate::core::{
    UampApp,
    config::{self, Version},
};

impl UampApp {
    pub fn migrate(&mut self) {
        let version = self.config.version();
        if version == config::VERSION {
            return;
        }

        let version = version.unwrap_or(Version(0, 7, 2));

        info!("Migrating from version `{version}`.");

        if version <= Version(0, 7, 3) {
            self.migrate_add_auto_tags();
        }

        self.config.set_version(config::VERSION);
    }

    fn migrate_add_auto_tags(&mut self) {
        info!("Adding auto tags to all songs in the library.");

        if self.config.auto_tags().is_empty() {
            return;
        }

        let auto_tags = self.config.auto_tags();

        self.library.mut_tags().init_tags(auto_tags);

        for s in self.library.iter_mut() {
            s.tags.extend(auto_tags.iter().map(|a| a.name.clone()));
        }

        let songs: Vec<_> = self.library.iter().collect();
        for t in auto_tags {
            self.library.mut_tags().0.get_mut(&t.name).unwrap().songs =
                songs.clone();
        }
    }
}
