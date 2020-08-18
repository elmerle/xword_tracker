mod database;
mod nytimes;
mod tracker;
mod util;

use std::fs::read_to_string;

use anyhow::Result;
use tokio;
use yaml_rust::{Yaml, YamlLoader};

use tracker::Tracker;

//#[tokio::main(core_threads=4, max_threads=8)]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let config = config()?;

    // let path = PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe");
    // let path = PathBuf::from(r"/mnt/c/Program Files (x86)/Google/Chrome/Application/chrome.exe");
    // let browser = Browser::new(LaunchOptionsBuilder::default().path(Some(path)).build().unwrap())?;
    // let tab = browser.wait_for_initial_tab()?;
    // login(&tab, &config)?;

    let session = config["session"].as_str().expect("Failed to get session id").to_string();
    let mut tracker = Tracker::new(session)?;
    tracker.update_times().await?;
    // tracker.foo().await?;
    //tracker.get_xwords().await?;
    //tracker.update_times().await?;
    //tracker.update_times()?;

    Ok(())
}

fn config() -> Result<Yaml> {
    let config_str = read_to_string("config.yaml")?;
    Ok(YamlLoader::load_from_str(&config_str)?[0].clone())
}

// fn login(tab: &Arc<Tab>, config: &Yaml) -> Result<(), Error> {
//     let username = config["username"].as_str().expect("Failed to find username.");
//     let password = config["password"].as_str().expect("Failed to find password");

//     tab.navigate_to("https://myaccount.nytimes.com/auth/login")?;
//     tab.wait_for_element("input#username")?.type_into(username)?;
//     tab.wait_for_element("input#password")?.type_into(password)?;
//     tab.wait_for_element("button [type=submit]")?.click()?;
//     tab.wait_until_navigated()?;

//     tab.navigate_to("https://www.nytimes.com/crosswords/archive")?;

//     let pdf = tab.print_to_pdf(None)?;
//     write("test.pdf", &pdf)?;

//     Ok(())
// }