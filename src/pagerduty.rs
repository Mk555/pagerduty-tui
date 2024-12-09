use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

use tokio::sync::mpsc;

use serde_json;
use serde::Deserialize;

use crate::utils::split_str;

const PAGERDUTY_URL: &str = "https://api.pagerduty.com";
const PAGERDUTY_INCIDENTS_ENDPOINT:&str = "/incidents";
const PAGERDUTY_USERS_ENDPOINT:&str = "/users";

pub const PAGER_DUTY_INCIDENT_URL: &str = ".pagerduty.com/incidents/";

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
struct PagerDutyAssignee{
  summary: String,
}
#[derive(Debug, Deserialize)]
struct PagerDutyAssignment{
  assignee: PagerDutyAssignee,
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
  assignments: Vec<PagerDutyAssignment>,
}

pub struct Incident {
  pub id: String,
  pub summary: String,
  pub service: String,
  pub status: String,
  pub created_at: String,
  pub triggered: bool,
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

  pub fn triggered(&self) -> &bool {
    &self.triggered
  }

}

pub struct PagerDuty {
  api_key: String,
  current_user_id: String,
  domain: String,
}
impl PagerDuty {
  pub async fn new(domain: &str, api_key: &str) -> Self{
    Self {
      domain: String::from(domain),
      api_key: String::from(api_key),
      current_user_id: get_current_user_id(api_key).await.expect("Error getting current user id"),
    }
  }

  pub fn get_pagerduty_api_key(&self) -> &str {
    &self.api_key
  }
  pub fn get_pagerduty_domain(&self) -> &str {
    &self.domain
  }

  pub async fn get_incidents(&self, all_incidents: bool) -> Result<Vec<Incident>, String> {
    let statuses: [&str; 2] = ["triggered","acknowledged"];
    let mut pd_incidents: Vec<PagerDutyIncident> = Vec::new();

    for status in statuses {
      let url_requets:String;
      if all_incidents {
        url_requets = format!("{}{}?statuses[]={}&limit=100",
            PAGERDUTY_URL,PAGERDUTY_INCIDENTS_ENDPOINT, status);
      } else {
        url_requets = format!("{}{}?statuses[]={}&user_ids[]={}&limit=100",
            PAGERDUTY_URL,PAGERDUTY_INCIDENTS_ENDPOINT, status, &self.current_user_id);
      }
      let client = Client::new();
      let response = client.get(&url_requets)
          .header(CONTENT_TYPE, "application/json")
          .header("Accept", "application/json")
          .header("Authorization", format!("Token token={}", &self.api_key))
          .send().await.expect("Error sending the API request to PagerDuty");
      if response.status().is_success() {
        let body_bytes = response.bytes().await.expect("Error while parsing PD response");
        let body = String::from_utf8_lossy(&body_bytes);

        let mut pd_incidents_buf:PagerDutyIncidents = serde_json::from_str(&body).expect("Error parsing result");
        pd_incidents_buf.incidents.reverse();
        pd_incidents.append(&mut pd_incidents_buf.incidents);
      }

    }

    let mut incidents_result: Vec<Incident> = Vec::new();
    for mut incident in pd_incidents {
      // Shorten summary to fit
      if incident.summary.len() > 100 {
        incident.summary = split_str(incident.summary, 100);
      }
      // Emergency
      if incident.priority.is_some() {
        incident.status = String::from("/!\\ P1 /!\\");
      }

      let mut triggered: bool = false;
      // Triggered
      if incident.status == "triggered" {
        triggered = true;
      }
      
      // Assignee
      let assignee: String;
      if incident.assignments.len() > 0 {
        assignee = incident.assignments.get(0).unwrap().assignee.summary.clone();
      }
      else {
        assignee = String::from("----------");
      }
      let created_at_str: String;
      if all_incidents {
        created_at_str = String::from(format!("{}\n{}", incident.created_at, assignee));
      }
      else {
        created_at_str = incident.created_at;
      }

      incident.service.summary = format!("{}", incident.service.summary);
      // Prepare the text to show
      incident.summary = format!("{}\n{}", incident.service.summary,incident.summary);

      incidents_result.push(Incident {
        id: incident.id,
        summary: incident.summary,
        service: incident.service.summary,
        status: incident.status,
        created_at: created_at_str,
        triggered: triggered,
      });
    }

    if incidents_result.len() == 0 {
      let empty_incident: Incident = Incident {
        id: String::from("---------"),
        summary: String::from(" - NO INCIDENTS | TIME FOR A BREAK - "),
        service: String::from(""),
        status: String::from("---------"),
        created_at: String::from("---------"),
        triggered: false,
      };
      incidents_result.push(empty_incident)
    }
    
    //incidents_result.reverse();
    Ok(incidents_result)
    
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

pub async fn get_items_async(domain: &str, api_key: &str, all_incidents: bool, tx: mpsc::UnboundedSender<Vec<Incident>>) -> Result<(), ()> {
  let pd_api_key = String::from(api_key);
  let pd_domain = String::from(domain);

  tokio::spawn(async move {
    let pd = PagerDuty::new(&pd_domain, &pd_api_key).await;
    let items_res = pd.get_incidents(all_incidents).await;
    match items_res {
      Ok(items) => {
        tx.send(items)
      }
      Err(_) => {
        tx.send(Vec::new())
      }
    }
  });

  Ok(())
}
