use tokio::sync::mpsc;

use crate::app::App;
use crate::utils::open_in_browser;

#[derive(PartialEq)]
pub enum Action {
  UpdateIncidents,
  Increment,
  Decrement,
  Open,
  Acknowledge,
  Quit,
  None,
}

// Tick rate
pub const REFRESH_RATE:i64 = 250;

pub async fn update(app: &mut App, msg: Action) -> Action {
  match msg {
    Action::UpdateIncidents => {
      app.items = app.pager_duty.get_incidents().await.expect("Error while retreiving incidents");
    },
    Action::Increment => {
      app.next();
    },
    Action::Decrement => {
      app.previous();
    },
    Action::Open => {
      let selected_id = app.state.selected().unwrap();
      let url = format!("https://ooyalaflex.pagerduty.com/incidents/{}", app.items[selected_id].id());
      open_in_browser(&url);
    },
    Action::Acknowledge => {
      let selected_id = app.state.selected().unwrap();
      app.pager_duty.acknowledge(app.items[selected_id].id()).await.expect("Error aknoledging");
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
              crossterm::event::KeyCode::Char('u') | crossterm::event::KeyCode::F(5) => Action::UpdateIncidents,
              crossterm::event::KeyCode::Char('o') | crossterm::event::KeyCode::Enter => Action::Open,
              crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char(' ') => Action::Acknowledge,
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
