use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

use serde_json;
use serde::Deserialize;

use crate::utils::split_str;

const PAGERDUTY_URL: &str = "https://api.pagerduty.com";
const PAGERDUTY_INCIDENTS_ENDPOINT:&str = "/incidents";
const PAGERDUTY_USERS_ENDPOINT:&str = "/users";

#[derive(Debug, Deserialize)]
struct PagerDutyUserResult{
  user: PagerDutyUser,
}
#[derive(Debug, Deserialize)]
struct PagerDutyUser{
  id: String,
}

#[derive(Debug, Deserialize)]
struct PagerDutyService{
  summary: String,
}
#[derive(Debug, Deserialize)]
struct PagerDutyPriority {
}

#[derive(Debug, Deserialize)]
struct PagerDutyIncidents{
  incidents: Vec<PagerDutyIncident>
}

#[derive(Debug, Deserialize)]
struct PagerDutyIncident {
  id: String,
  summary: String,
  created_at: String,
  status: String,
  service: PagerDutyService,
  priority: Option<PagerDutyPriority>,
}

pub struct Incident {
  id: String,
  summary: String,
  status: String,
  created_at: String,
}

impl Incident {
  pub const fn ref_array(&self) -> [&String; 3] {
    [&self.status, &self.summary, &self.created_at]
  }

  pub fn status(&self) -> &str {
    &self.status
  }

  pub fn summary(&self) -> &str {
    &self.summary
  }

  pub fn created_at(&self) -> &str {
    &self.created_at
  }

  pub fn id(&self) -> &str {
    &self.id
  }

}

pub struct PagerDuty {
  api_key: String,
  current_user_id: String,
}
impl PagerDuty {
  pub async fn new(api_key: &str) -> Self{
    Self {
      api_key: String::from(api_key),
      current_user_id: get_current_user_id(api_key).await.expect("Error getting current user id"),
    }
  }

  pub fn get_pagerduty_api_key(&self) -> &str {
    &self.api_key
  }

  pub async fn get_incidents(&self) -> Result<Vec<Incident>, String> {
    let url_requets:String = format!("{}{}?statuses[]=triggered&statuses[]=acknowledged&user_ids[]={}&limit=100",
      PAGERDUTY_URL,PAGERDUTY_INCIDENTS_ENDPOINT, &self.current_user_id);

    let client = Client::new();
    let response = client.get(&url_requets)
      .header(CONTENT_TYPE, "application/json")
      .header("Accept", "application/json")
      .header("Authorization", format!("Token token={}", &self.api_key))
      .send().await.expect("Error sending the API request to PagerDuty");

    if response.status().is_success() {
      let body_bytes = response.bytes().await.expect("Error while parsing PD response");
      let body = String::from_utf8_lossy(&body_bytes);

      let incidents:PagerDutyIncidents = serde_json::from_str(&body).expect("Error parsing result");

      let mut incidents_result: Vec<Incident> = Vec::new();
      for mut incident in incidents.incidents {
        // Shorten summary to fit
        if incident.summary.len() > 100 {
          incident.summary = split_str(incident.summary, 100);
        }
        // Emergency
        if incident.priority.is_some() {
          incident.status = String::from("/!\\ P1 /!\\");
        }

        // Prepare the text to show
        incident.summary = format!("{}\n{}", incident.service.summary,incident.summary);

        incidents_result.push(Incident {
          id: incident.id,
          summary: incident.summary,
          status: incident.status,
          created_at: incident.created_at,
        });
      }

      if incidents_result.len() == 0 {
        let empty_incident: Incident = Incident {
          id: String::from("---------"),
          summary: String::from(" - NO INCIDENTS | TIME FOR A BREAK - "),
          status: String::from("---------"),
          created_at: String::from("---------"),
        };
        incidents_result.push(empty_incident)
      }
      
      incidents_result.reverse();
      Ok(incidents_result)
    } else {
      eprint!("Error while sending request to PagerDuty : {:#?}", response);

      Err(String::from("Error while sending request"))
    }
  }

}

async fn get_current_user_id(api_key: &str) -> Result<String,String> {
  let current_user_requets:String = format!("{}{}/me",PAGERDUTY_URL,PAGERDUTY_USERS_ENDPOINT);

  let client = Client::new();
  let response = client.get(&current_user_requets)
    .header(CONTENT_TYPE, "application/json")
    .header("Accept", "application/json")
    .header("Authorization", format!("Token token={}", api_key))
    .send().await.expect("Error sending the API request to PagerDuty");

  if response.status().is_success() {
    let body_bytes = response.bytes().await.expect("Error while parsing PD response");
    let body = String::from_utf8_lossy(&body_bytes);

    let current_user:PagerDutyUserResult = serde_json::from_str(&body).expect("Error parsing result");
    Ok(current_user.user.id)
  } else {
    Err(response.status().to_string())
  }
}

pub async fn acknowledge_async(api_key: &str, id: &str) -> Result<(), ()> {
  let url_requet:String = format!("{}{}/{}",PAGERDUTY_URL,PAGERDUTY_INCIDENTS_ENDPOINT, id);

  let api_key_moved = String::from(api_key);

  tokio::spawn(async move {
    let client = Client::new();
    let _response = client.put(&url_requet)
      .header(CONTENT_TYPE, "application/json")
      .header("Accept", "application/json")
      .header("Authorization", format!("Token token={}", api_key_moved))
      .body("{\n  \"incident\": {\n    \"type\": \"incident_reference\",\n    \"status\": \"acknowledged\"\n  }\n}")
      .send().await.expect("Error sending the API request to PagerDuty");
  });
  
  Ok(())
}
