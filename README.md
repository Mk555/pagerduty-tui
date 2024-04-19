# PagerDuty TUI

PagerDuty TUI is a minimalist terminal user interface developed in Rust for managing incidents in PagerDuty. It provides a simple interface to list triggered incidents, acknowledge them, and open them in your default browser.

![Screenshot](DOC/pagerduty-tui-screenshot.png)

## Installation

Download the last release on [Gihtub Releases](https://github.com/Mk555/pagerduty-tui/releases/latest)

## Update

To update, run the self update : 

`pagerduty-tui update`


## Build

To install Cargo, follow the steps here : 

`https://doc.rust-lang.org/cargo/getting-started/installation.html`

Then compile the app :

`cargo build`

The binary will be stored here : `./target/debug/pagerduty-tui`

## Configuration

PagerDuty TUI requires a configuration file located at `~/.config/pagerduty_tui.yaml`. This configuration file is used to store your PagerDuty API key, which is necessary for accessing PagerDuty incident data.

### Create Pager Duty API Key

![Create PagerDuty API Key](DOC/PagerDutyApiKey.png "PagerDutyApiKey")

### Configuration File Format

The configuration file should be in YAML format and be stored in `~/.config/pagerduty_tui.yaml` 

You need to put you PagerDuty API Key in this value :

`pagerduty_api_key: <PagerDuty API Key>`

Replace `<PagerDuty API Key>` with your actual PagerDuty API key. This key is required for authenticating requests to the PagerDuty API and accessing incident data.

By default the refresh time is 30 seconds, but you can change it with the following configuration :
`refresh_rate: <seconds>`

Replace `<seconds>` by the number of seconds between 2 refresh.

## Usage

After installation, you can run PagerDuty TUI by executing the following command in your terminal:

`pagerduty-tui`

## Key Bindings

 - `Arrow Up/Down`: Navigate incidents
 - `r`: Update incident list
 - `Enter`: Open incident in default browser
 - `a`: Acknowledge incident
 - `h`: Hide Acknowledged incidents
 - `q`: Quit PagerDuty TUI

### Contributions

Contributions to PagerDuty TUI are welcome! If you find any issues or have suggestions for improvements, feel free to open an issue or submit a pull request.
License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
