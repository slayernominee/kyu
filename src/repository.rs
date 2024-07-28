use ini::Ini;
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

        std::fs::create_dir(&s.gitdir).unwrap();

        s.mkdir(vec!["objects"]);
        s.mkdir(vec!["refs", "heads"]);
        s.mkdir(vec!["refs", "tags"]);
        s.mkdir(vec!["branches"]);

        // write the description file
        let description_path = s.gitdir.clone() + "/description";
        std::fs::write(
            &description_path,
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )
        .unwrap();

        // write the HEAD file
        let head_path = s.gitdir.clone() + "/HEAD";
        std::fs::write(&head_path, "ref: refs/heads/master\n").unwrap();

        s.config.dump(&(s.gitdir.clone() + "/config"));

        Ok(s)
    }

    pub fn get_object_path(&self, sha: &str) -> String {
        let mut path = self.gitdir.clone() + "/objects/";
        path.push_str(&sha[..2]);
        path.push('/');
        path.push_str(&sha[2..]);
        path
    }

    pub fn load(path: Option<String>) -> Result<Self, RepError> {
        let workdir = path.unwrap_or(env::current_dir().unwrap().to_str().unwrap().to_string());
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
            let r = std::fs::create_dir(&dir);
            if r.is_err() {
                if r.err().unwrap().kind() == std::io::ErrorKind::AlreadyExists {
                    continue;
                }
                println!("Failed to create directory: {:?}", dir);
            }
        }
    }

    pub fn get_workdir(&self) -> &String {
        &self.workdir
    }
}

// Config
#[derive(Debug)]
struct Config {
    bare: bool,
    repository_format_version: i32,
    file_mode: bool,
    /*ignore_case: bool,
    precompose_unicode: bool,
    logal_lref_updates: bool,*/
}

impl Config {
    fn default() -> Self {
        Self {
            bare: false,                  // bare -> no working directory, only the .git directory
            repository_format_version: 0, // 0 -> without extensions in the git directory, 1 -> with extensions
            file_mode: false,             // tracking file mode changes (permissions)
                                          //ignore_case: false,
                                          //precompose_unicode: false,
                                          //logal_lref_updates: false,
        }
    }

    fn load(config_path: &str) -> Result<Self, ConfigError> {
        // TODO: make the config loadable

        let config = Ini::load_from_file(config_path).unwrap();
        //  .get_from(Option<"core">, "repositoryformatversion");

        let repository_format_version = config
            .get_from(Some("core"), "repositoryformatversion")
            .unwrap()
            .parse::<i32>()
            .unwrap();

        if repository_format_version != 0 {
            return Err(ConfigError::UnsupportedRepositoryFormatVersion);
        }

        let bare = config
            .get_from(Some("core"), "bare")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let file_mode = config
            .get_from(Some("core"), "filemode")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        /*let ignore_case = config
            .get_from(Some("core"), "ignorecase")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let precompose_unicode = config
            .get_from(Some("core"), "precomposeunicode")
            .unwrap()
            .parse::<bool>()
            .unwrap();

        let logal_lref_updates = config
            .get_from(Some("core"), "logallrefupdates")
            .unwrap()
            .parse::<bool>()
            .unwrap();*/

        Ok(Self {
            bare,
            repository_format_version,
            file_mode,
            /*ignore_case,
            precompose_unicode,
            logal_lref_updates,*/
        })
    }

    fn dump(&self, path: &str) {
        let mut conf = Ini::new();
        conf.with_section(Some("core"))
            .set("bare", self.bare.to_string())
            .set(
                "repositoryformatversion",
                self.repository_format_version.to_string(),
            )
            .set("filemode", self.file_mode.to_string())
            /*.set("ignorecase", self.ignore_case.to_string())
            .set("precomposeunicode", self.precompose_unicode.to_string())
            .set("logallrefupdates", self.logal_lref_updates.to_string())*/
            ;
        conf.write_to_file(path).unwrap();
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
