use std::process::Command;

pub fn split_str(text:String, lenght:u16) -> String{
  let mut result:String = String::from("");
  let buf:String = text;

  for i in 0..lenght {
    result = format!("{}{}", result, buf.chars().nth(i.into()).unwrap());
  }

  format!("{}...",result)
}

pub fn open_in_browser(url:&str){
  let _ = Command::new("/usr/bin/xdg-open")
    .arg(url)
    .output();
}
