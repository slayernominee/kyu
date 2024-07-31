#![allow(dead_code)]

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
        let workdir = match path {
            Some(p) => p,
            None => env::current_dir().unwrap().to_str().unwrap().to_string(),
        };

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

    pub fn get_refs(&self) -> Vec<(String, String)> {
        let mut refs = vec![];

        let heads = std::fs::read_dir(self.gitdir.clone() + "/refs/heads").unwrap();
        for head in heads {
            let head = head.unwrap();
            let head = head.file_name().into_string().unwrap();
            let head = head.trim().to_string();
            let hash =
                std::fs::read_to_string(self.gitdir.clone() + "/refs/heads/" + &head).unwrap();
            let hash = hash.trim().to_string();
            refs.push((format!("refs/heads/{}", head), hash));
        }

        let tags = std::fs::read_dir(self.gitdir.clone() + "/refs/tags").unwrap();
        for tag in tags {
            let tag = tag.unwrap();
            let tag = tag.file_name().into_string().unwrap();
            let tag = tag.trim().to_string();
            let hash = std::fs::read_to_string(self.gitdir.clone() + "/refs/tags/" + &tag).unwrap();
            let hash = hash.trim().to_string();
            refs.push((format!("refs/tags/{}", tag), hash));
        }

        refs
    }

    fn get_last_commit_hash(&self) -> Result<String, RepError> {
        let head_path = self.gitdir.clone() + "/HEAD";
        let head = std::fs::read_to_string(&head_path).unwrap();
        let head = head.trim().split(':').collect::<Vec<&str>>()[1].trim();

        let head_path = self.gitdir.clone() + "/" + head;
        let head = std::fs::read_to_string(&head_path);

        if head.is_err() {
            let branch = head_path.split('/').collect::<Vec<&str>>();
            let branch = branch.last().unwrap();
            return Err(RepError::NoCommitsInBranch(branch.to_string()));
        }
        let head = head.unwrap();

        let head = head.trim();

        Ok(head.to_string())
    }

    pub fn ref_resolve(&self, reference: &str) -> Result<String, RepError> {
        if reference == "HEAD" || reference == "HEAD~" {
            let c = self.get_last_commit_hash();
            if c.is_err() {
                return Err(c.err().unwrap());
            }
            let c = c.unwrap();
            if c.starts_with("ref: ") {
                let c = c.split(':').collect::<Vec<&str>>()[1].trim();
                return self.ref_resolve(c);
            } else {
                return Ok(c);
            }
        }

        let reference = reference.to_string();

        if reference.starts_with("refs/") || reference.len() != 40 {
            let mut head_path = self.gitdir.clone() + "/" + &reference;

            if !std::path::Path::new(&head_path).exists() {
                // check if the path exists with a refs/ before
                // a refs/heads/ or a refs/tags/
                let t_head = self.gitdir.clone() + "/refs/" + &reference;
                let t2_head = self.gitdir.clone() + "/refs/heads/" + &reference;
                let t3_head = self.gitdir.clone() + "/refs/tags/" + &reference;
                if std::path::Path::new(&t_head).exists() {
                    head_path = t_head;
                } else if std::path::Path::new(&t2_head).exists() {
                    head_path = t2_head;
                } else if std::path::Path::new(&t3_head).exists() {
                    head_path = t3_head;
                } else {
                    return Err(RepError::InvalidReference(reference));
                }
            }

            let head = std::fs::read_to_string(&head_path);

            if head.is_err() {
                return Err(RepError::InvalidReference(reference));
            }

            let head = head.unwrap();
            let head = head.trim();

            if head.starts_with("ref: ") {
                let head = head.split(':').collect::<Vec<&str>>()[1].trim();
                return self.ref_resolve(head);
            } else {
                return Ok(head.to_string());
            }
        } else {
            Ok(reference)
        }
    }

    pub fn get_object_path(&self, sha: &str) -> String {
        let mut path = self.gitdir.clone() + "/objects/";
        path.push_str(&sha[..2]);
        path.push('/');
        path.push_str(&sha[2..]);
        path
    }

    pub fn load(path: Option<String>) -> Result<Self, RepError> {
        let workdir = match path {
            Some(p) => p,
            None => {
                // check if we are in a git repository / subdirectory
                let mut pwd = env::current_dir().unwrap().to_str().unwrap().to_string();

                let mut gitdir = pwd.clone() + "/.git";

                while pwd != "/" && !std::path::Path::new(&gitdir).exists() {
                    let p = std::path::Path::new(&pwd);
                    let parent = p.parent().unwrap();
                    pwd = parent.to_str().unwrap().to_string();
                    gitdir = pwd.clone() + "/.git";
                }

                pwd
            }
        };

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
    NoCommitsInBranch(String),
    InvalidReference(String),
}

#[derive(Debug)]
pub enum ConfigError {
    UnsupportedRepositoryFormatVersion,
}
