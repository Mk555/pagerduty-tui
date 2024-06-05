use std::fs;
use serde::{Deserialize,Serialize};
use serde_yaml;
use homedir::get_my_home;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
  pagerduty_domain: String,
  pagerduty_api_key: String,
  refresh_rate: Option<i64>,
}

impl AppConfig {
  pub fn new() -> Self {
    let dir_home:String = String::from(get_my_home().expect("msg").unwrap().to_str().unwrap());
    let config_path = format!("{}/.config/pagerduty_tui.yaml",dir_home);

    let str_config:String = fs::read_to_string(config_path)
      .expect("Unable to read YAML config file");
    let config:AppConfig = serde_yaml::from_str(&str_config)
      .expect("Error while parsing YAML config");

    Self {
      pagerduty_domain: config.pagerduty_domain,
      pagerduty_api_key: config.pagerduty_api_key,
      refresh_rate: config.refresh_rate,
    }
  }

  pub fn get_pagerduty_api_key(&self) -> &str {
    &self.pagerduty_api_key
  }
  pub fn get_pagerduty_domain(&self) -> &str {
    &self.pagerduty_domain
  }
  pub fn get_refresh_rate(&self) -> &Option<i64> {
    &self.refresh_rate
  }
}
