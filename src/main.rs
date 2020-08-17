mod database;
mod nytimes;
mod tracker;

use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

//use std::error::Error;
use failure::Error;
use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio;
use yaml_rust::{Yaml, YamlLoader};

use tracker::Tracker;

struct Xword {

}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let feeds = (0..10).collect::<Vec<_>>();
//     let res = read_feeds(feeds).await;

//     Ok(())
// }

// async fn read_feeds(feeds: Vec<u32>) -> Vec<u32> {
//     feeds.iter()
//         .map(read_feed)
//         .collect::<FuturesUnordered<_>>()
//         .collect::<Vec<_>>()
//         .await
// }

// async fn read_feed(feed: &u32) -> u32 {
//     println!("start {}", feed);
//     tokio::time::delay_for(Duration::from_millis((feed * 100) as u64)).await;
//     println!("end {}", feed);
//     feed * 2
// }

async fn foo(i: &u32) -> String {
    println!("{} start", i);
    tokio::time::delay_for(Duration::from_secs(5-*i as u64)).await;
    println!("{} end", i);
    return i.to_string();
}

#[tokio::main(core_threads=4, max_threads=8)]
//#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    //let config = config()?;



    // let path = PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe");
    // let path = PathBuf::from(r"/mnt/c/Program Files (x86)/Google/Chrome/Application/chrome.exe");
    // let browser = Browser::new(LaunchOptionsBuilder::default().path(Some(path)).build().unwrap())?;
    // let tab = browser.wait_for_initial_tab()?;
    // login(&tab, &config)?;

    // let f = (1..5).collect::<Vec<u32>>();
    // let fi = f.iter();
    // println!("====1");
    // let g = fi.map(foo);
    // println!("====2");
    // let h = g.collect::<FuturesUnordered<_>>();
    // println!("====3");
    // let i = h.collect::<Vec<String>>().await;
    // println!("{:?}", i);

    let mut tracker = Tracker::new("2UpqIGuS2G/lJhJrtcnpckl0t.SrDv0OhLqm4VdjInRdqG1Mf3LYxupuTwqL7IkUOwjOoea6bgYnT6Q1KiaXrnBNNELieBARpvzmk2XaDneSmOAwdrWzeNm/tlGoddbcewMaeJYR/IBXdPrTGW6xmzCutvM.KilrchgnOxRoBoMsMCN/xm6fhVXSlVACaiQLsJ5i8MdzZHoVL1lE5/cAB4ERur5n7S6iGE/8yjeb7W1LYVfqHf0Nn4EM00".to_string())?;
    tracker.foo().await?;
    //tracker.get_xwords().await?;
    //tracker.update_times().await?;
    //tracker.update_times()?;

    Ok(())
}

// fn config() -> Result<Yaml, Error> {
//     let config_str = read_to_string("config.yaml")?;
//     Ok(YamlLoader::load_from_str(&config_str)?[0].clone())
// }

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