use tokio::sync::mpsc;

use crate::app::App;
use crate::pagerduty::{acknowledge_async, get_items_async, PAGER_DUTY_INCIDENT_URL};
use crate::utils::open_in_browser;

#[derive(PartialEq)]
pub enum Action {
  UpdateIncidents,
  Increment,
  Decrement,
  Top,
  Open,
  Acknowledge,
  AcknowledgeAllService,
  HideAck,
  AllIncidents,
  Quit,
  None,
}

// Tick rate
pub const REFRESH_RATE:i64 = 250;

pub async fn update(app: &mut App, msg: Action) -> Action {
  match msg {
    Action::UpdateIncidents => {
      app.refreshing = true;
      let _res = get_items_async(app.pager_duty.get_pagerduty_domain(),app.pager_duty.get_pagerduty_api_key(), app.all_incidents, app.items_tx.clone()).await;
    },
    Action::Increment => {
      app.next();
    },
    Action::Decrement => {
      app.previous();
    },
    Action::Top => {
      app.top();
    }
    Action::Open => {
      let selected_id = app.state.selected().unwrap();
      let url = format!("https://{}{}{}", app.pager_duty.get_pagerduty_domain(), PAGER_DUTY_INCIDENT_URL, app.items[selected_id].id());
      open_in_browser(&url);
    },
    Action::Acknowledge => {
      let selected_id = app.state.selected().unwrap();
      if app.items[selected_id].triggered {
        let selected_item:&str = app.items[selected_id].id();
        acknowledge_async(&app.pager_duty.get_pagerduty_api_key(), selected_item).await.expect("Error during aknowledge");
        app.items[selected_id].status = format!("{}\nSending Ack", app.items[selected_id].status);
        app.items[selected_id].triggered = false;
      }
    },
    Action::AcknowledgeAllService => {
      // Loop on all incidents
      for i in 0..app.items.len() {
        if app.items[i].service == app.items[app.state.selected().unwrap()].service && app.items[i].triggered {
          let item_to_ack: &str = app.items[i].id();
          acknowledge_async(&app.pager_duty.get_pagerduty_api_key(), item_to_ack).await.expect("Error during aknowledge");
          app.items[i].status = format!("{}\nSending Ack", app.items[i].status);
          app.items[i].triggered = false;
        }
      }
    },
    Action::HideAck => {
      if app.hide_ack {
        app.hide_ack = false;
      } else {
        app.hide_ack = true;
      }
    },
    Action::AllIncidents => {
      if app.all_incidents {
        app.all_incidents = false;
      } else {
        app.all_incidents = true;
      }
      let _res = get_items_async(app.pager_duty.get_pagerduty_domain(),app.pager_duty.get_pagerduty_api_key(), app.all_incidents, app.items_tx.clone()).await;
      app.refreshing = true;
    },
    Action::Quit => app.should_quit = true, // You can handle cleanup and exit here
    _ => {},
  };
  Action::None
}

pub fn handle_event(_app: &App, tx: mpsc::UnboundedSender<Action>) -> tokio::task::JoinHandle<()> {
  let tick_rate = std::time::Duration::from_millis(u64::try_from(REFRESH_RATE).expect("Refresh rate not valid"));

  tokio::spawn(async move {
    loop {
      let action = if crossterm::event::poll(tick_rate).unwrap() {
        if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
          if key.kind == crossterm::event::KeyEventKind::Press {
            match key.code {
              crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => Action::Increment,
              crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => Action::Decrement,
              crossterm::event::KeyCode::Home => Action::Top,
              crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::F(5) => Action::UpdateIncidents,
              crossterm::event::KeyCode::Char('o') | crossterm::event::KeyCode::Enter => Action::Open,
              crossterm::event::KeyCode::Char(' ') => Action::Acknowledge,
              crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => Action::AcknowledgeAllService,
              crossterm::event::KeyCode::Char('h') => Action::HideAck,
              crossterm::event::KeyCode::Char('g') => Action::AllIncidents,
              crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => Action::Quit,

              _ => Action::None,
            }
          } else {
            Action::None
          }
        } else {
          Action::None
        }
      } else {
        Action::None
      };
      if let Err(_) = tx.send(action) {
        break;
      }
    }
  })
}
