use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

use serde_json;
use serde::Deserialize;

use crate::utils::split_str;

#[derive(Debug, Deserialize)]
struct IncidentsPagerDuty{
  incidents: Vec<IncidentPagerDuty>
}

#[derive(Debug, Deserialize)]
struct IncidentPagerDuty {
  id: String,
  summary: String,
  created_at: String,
  status: String,
  service: Service,
  priority: Option<Priority>,
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

#[derive(Debug, Deserialize)]
struct Service {
  summary: String,
}

#[derive(Debug, Deserialize)]
struct Priority {
}

pub struct PagerDuty {
  api_key: String,
}
impl PagerDuty {
  pub fn new(api_key: &str) -> Self{
    Self {
      api_key: String::from(api_key),
    }
  }

  pub async fn acknowledge(&self, id: &str) -> Result<(), ()> {
    let url_requet:String = format!("https://api.pagerduty.com/incidents/{}", id);

    let client = Client::new();
    let _response = client.put(&url_requet)
      .header(CONTENT_TYPE, "application/json")
      .header("Accept", "application/json")
      .header("Authorization", format!("Token token={}", &self.api_key))
      .body("{\n  \"incident\": {\n    \"type\": \"incident_reference\",\n    \"status\": \"acknowledged\"\n  }\n}")
      .send().await.expect("Error sending the API request to PagerDuty");

    Ok(())
  }

  pub async fn get_incidents(&self) -> Result<Vec<Incident>, String> {
    let url_requets:String = format!("https://api.pagerduty.com/incidents?statuses[]=triggered&limit=100");

    let client = Client::new();
    let response = client.get(&url_requets)
      .header(CONTENT_TYPE, "application/json")
      .header("Accept", "application/json")
      .header("Authorization", format!("Token token={}", &self.api_key))
      .send().await.expect("Error sending the API request to PagerDuty");

    if response.status().is_success() {
      let body_bytes = response.bytes().await.expect("Error while parsing PD response");
      let body = String::from_utf8_lossy(&body_bytes);

      let incidents:IncidentsPagerDuty = serde_json::from_str(&body).expect("Error parsing result");

      let mut icindents_result: Vec<Incident> = Vec::new();
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

        icindents_result.push(Incident {
          id: incident.id,
          summary: incident.summary,
          status: incident.status,
          created_at: incident.created_at,
        });
      }

      Ok(icindents_result)
    } else {
      eprint!("Error while sending request to PagerDuty : {:#?}", response);

      Err(String::from("Error while sending request"))
    }


  }

}
