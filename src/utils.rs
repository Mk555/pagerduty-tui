use std::process::Command;
use  std::env;

pub fn split_str(text:String, lenght:u16) -> String{
  let mut result:String = String::from("");
  let buf:String = text;

  for i in 0..lenght {
    result = format!("{}{}", result, buf.chars().nth(i.into()).unwrap());
  }

  format!("{}...",result)
}

pub fn open_in_browser(url:&str){
  let os = env::consts::OS;
  
  if os == "linux" {
    let _ = Command::new("/usr/bin/xdg-open")
      .arg(url)
      .output();
  } else if os == "macos" {
    let _ = Command::new("/usr/bin/open")
      .arg(url)
      .output();
  } else {
    ()
  }
}
