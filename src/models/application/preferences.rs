use errors::*;
use app_dirs::{app_root, AppDataType, AppInfo};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use yaml::yaml::{Yaml, YamlLoader};

const FILE_NAME: &'static str = "config.yml";
const APP_INFO: AppInfo = AppInfo {
    name: "amp",
    author: "Jordan MacDonald",
};
const TYPES_KEY: &'static str = "types";
const THEME_KEY: &'static str = "theme";
const TAB_WIDTH_KEY: &'static str = "tab_width";
const LINE_LENGTH_GUIDE_KEY: &'static str = "line_length_guide";
const LINE_WRAPPING_KEY: &'static str = "line_wrapping";
const SOFT_TABS_KEY: &'static str = "soft_tabs";

const THEME_DEFAULT: &'static str = "solarized_dark";
const TAB_WIDTH_DEFAULT: usize = 2;
const LINE_LENGTH_GUIDE_DEFAULT: usize = 80;
const LINE_WRAPPING_DEFAULT: bool = true;
const SOFT_TABS_DEFAULT: bool = true;

pub struct Preferences {
    data: Option<Yaml>,
}

impl Preferences {
    pub fn new(data: Option<Yaml>) -> Preferences {
        Preferences { data: data }
    }

    pub fn load() -> Result<Preferences> {
        // Build a path to the config file.
        let mut config_path =
            app_root(AppDataType::UserConfig, &APP_INFO)
                .chain_err(|| "Couldn't create or open application config directory")?;
        config_path.push(FILE_NAME);

        // Open (or create) the config file.
        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_path)
            .chain_err(|| "Couldn't create or open config file")?;

        // Read the config file's contents.
        let mut data = String::new();
        config_file
            .read_to_string(&mut data)
            .chain_err(|| "Couldn't read config file")?;

        // Parse the config file's contents and get the first YAML document inside.
        let parsed_data = YamlLoader::load_from_str(&data)
            .chain_err(|| "Couldn't parse config file")?;
        let document = parsed_data.into_iter().nth(0);

        Ok(Preferences { data: document })
    }

    pub fn theme(&self) -> &str {
        self.data
            .as_ref()
            .and_then(|data| if let Yaml::String(ref theme) = data[THEME_KEY] {
                          Some(theme.as_str())
                      } else {
                          None
                      })
            .unwrap_or(THEME_DEFAULT)
    }

    pub fn tab_width(&self, path: Option<&PathBuf>) -> usize {
        self.data
            .as_ref()
            .and_then(|data| {
                if let Some(extension) = path.and_then(|p| p.extension()).and_then(|e| e.to_str()) {
                    if let Yaml::Integer(tab_width) = data[TYPES_KEY][extension][TAB_WIDTH_KEY] {
                        return Some(tab_width as usize);
                    } else if let Yaml::Integer(tab_width) = data[TAB_WIDTH_KEY] {
                        return Some(tab_width as usize);
                    }
                } else if let Yaml::Integer(tab_width) = data[TAB_WIDTH_KEY] {
                    return Some(tab_width as usize);
                }

                None
            })
            .unwrap_or(TAB_WIDTH_DEFAULT)
    }

    pub fn line_length_guide(&self) -> Option<usize> {
        self.data
            .as_ref()
            .and_then(|data| match data[LINE_LENGTH_GUIDE_KEY] {
                          Yaml::Integer(line_length) => Some(line_length as usize),
                          Yaml::Boolean(line_length_guide) => {
                              if line_length_guide {
                                  Some(LINE_LENGTH_GUIDE_DEFAULT)
                              } else {
                                  None
                              }
                          }
                          _ => None,
                      })
    }

    pub fn line_wrapping(&self) -> bool {
        self.data
            .as_ref()
            .and_then(|data| if let Yaml::Boolean(wrapping) = data[LINE_WRAPPING_KEY] {
                          Some(wrapping)
                      } else {
                          None
                      })
            .unwrap_or(LINE_WRAPPING_DEFAULT)
    }

    pub fn soft_tabs(&self) -> bool {
        self.data
            .as_ref()
            .and_then(|data| if let Yaml::Boolean(soft_tabs) = data[SOFT_TABS_KEY] {
                          Some(soft_tabs)
                      } else {
                          None
                      })
            .unwrap_or(SOFT_TABS_DEFAULT)
    }

    pub fn tab_content(&self) -> String {
        if self.soft_tabs() {
            format!("{:1$}", "", self.tab_width(None))
        } else {
            String::from("\t")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Preferences, YamlLoader};
    use std::path::PathBuf;

    #[test]
    fn preferences_returns_user_defined_theme_name() {
        let data = YamlLoader::load_from_str("theme: \"my_theme\"").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.theme(), "my_theme");
    }

    #[test]
    fn tab_width_returns_user_defined_data() {
        let data = YamlLoader::load_from_str("tab_width: 12").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_width(None), 12);
    }

    #[test]
    fn tab_width_returns_user_defined_type_specific_data() {
        let data = YamlLoader::load_from_str("tab_width: 12\ntypes:\n  rs:\n    tab_width: 24")
            .unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_width(Some(PathBuf::from("preferences.rs")).as_ref()),
                   24);
    }

    #[test]
    fn tab_width_returns_default_when_user_defined_type_specific_data_not_found() {
        let data = YamlLoader::load_from_str("tab_width: 12").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_width(Some(PathBuf::from("preferences.rs")).as_ref()),
                   12);
    }

    #[test]
    fn preferences_returns_user_defined_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: 100").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(), Some(100));
    }

    #[test]
    fn preferences_returns_user_disabled_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(), None);
    }

    #[test]
    fn preferences_returns_user_default_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: true").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(), Some(80));
    }

    #[test]
    fn preferences_returns_user_defined_line_wrapping() {
        let data = YamlLoader::load_from_str("line_wrapping: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_wrapping(), false);
    }

    #[test]
    fn preferences_returns_user_defined_soft_tabs() {
        let data = YamlLoader::load_from_str("soft_tabs: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.soft_tabs(), false);
    }

    #[test]
    fn tab_content_uses_tab_width_spaces_when_soft_tabs_are_enabled() {
        let data = YamlLoader::load_from_str("soft_tabs: true\ntab_width: 5").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_content(), "     ");
    }

    #[test]
    fn tab_content_returns_tab_character_when_soft_tabs_are_disabled() {
        let data = YamlLoader::load_from_str("soft_tabs: false\ntab_width: 5").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_content(), "\t");
    }
}
