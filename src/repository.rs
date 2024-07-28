use std::env;

// Repository
#[derive(Debug)]
pub struct Repository {
    workdir: String,
    gitdir: String,
    config: Config,
}

impl Repository {
    pub fn init(path: Option<String>) -> Result<Self, RepError> {
        let workdir = path.unwrap_or(env::current_dir().unwrap().to_str().unwrap().to_string());
        let gitdir = workdir.clone() + "/.git";

        if std::path::Path::new(&gitdir).exists() {
            return Err(RepError::AlreadyExists);
        }

        let s = Self {
            workdir,
            gitdir,
            config: Config::default(),
        };

        // TODO: initailize git directory

        std::fs::create_dir(&s.gitdir).unwrap();
        // TODO: init

        Ok(s)
    }

    pub fn load() -> Result<Self, RepError> {
        let workdir = env::current_dir().unwrap().to_str().unwrap().to_string();
        let gitdir = workdir.clone() + "/.git";
        let config_path = gitdir.clone() + "/config";

        if !std::path::Path::new(&gitdir).exists() {
            return Err(RepError::NotARepository);
        }

        if !std::path::Path::new(&config_path).exists() {
            return Err(RepError::ConfigFileMissing);
        }

        let config = Config::load(&config_path);

        if config.is_err() {
            return Err(RepError::ConfigError(config.err().unwrap()));
        }

        let s = Self {
            workdir,
            gitdir,
            config: config.ok().unwrap(),
        };

        Ok(s)
    }

    fn mkdir(&self, path: Vec<&str>) {
        let mut dir = self.gitdir.clone();
        dir.push('/');
        for p in path {
            dir.push_str(p);
            dir.push('/');
            std::fs::create_dir(&dir).unwrap();
        }
    }
}

// Config
#[derive(Debug)]
struct Config {
    repository_format_version: i32,
}

impl Config {
    fn default() -> Self {
        Self {
            repository_format_version: 0,
        }
    }

    fn load(config_path: &str) -> Result<Self, ConfigError> {
        // TODO: make the config loadable

        let config = ini!(config_path);

        let repository_format_version = config
            .get("core")
            .expect("core section not found")
            .get("repositoryformatversion")
            .expect("repositoryformatversion not found")
            .to_owned()
            .unwrap()
            .parse::<i32>()
            .expect("repositoryformatversion is not a number");

        if repository_format_version != 0 {
            return Err(ConfigError::UnsupportedRepositoryFormatVersion);
        }

        Ok(Self {
            repository_format_version,
        })
    }
}

// Errors
#[derive(Debug)]
pub enum RepError {
    AlreadyExists,
    NotARepository,
    ConfigFileMissing,
    ConfigError(ConfigError),
}

#[derive(Debug)]
pub enum ConfigError {
    UnsupportedRepositoryFormatVersion,
}
