use self_update::cargo_crate_version;

pub fn update_bin() -> Result<(), Box<dyn ::std::error::Error>> {
  let status = self_update::backends::github::Update::configure()
      .repo_owner("Mk555")
      .repo_name("pagerduty-tui")
      .bin_name("pagerduty-tui-linux-amd64")
      .target("linux-amd64")
      .show_download_progress(true)
      .current_version(cargo_crate_version!())
      .build()?
      .update()?;
  println!("Update status: `{}`!", status.version());
  Ok(())
}
