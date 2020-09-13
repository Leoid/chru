use reqwest;
use scraper::{Html, Selector};
use itertools::Itertools;
use http::{HeaderMap, HeaderValue};
use url::{Url};
use std::thread;

enum LinkOptions{
    INTERNAL,
    EXTERNAL,
}

fn main(){
    println!("Start Scrapping.......");
    let mut fetched_urls: Vec<String> = Vec::new();

    get_urls(LinkOptions::INTERNAL, &mut fetched_urls,"http://b1twis3.ca");
    for i in fetched_urls.clone(){
        println!("Scarping {}",i);
        get_urls(LinkOptions::INTERNAL, &mut fetched_urls,&i);
    }
    for i in fetched_urls.clone(){
        println!("url: {}",i);
    }


    //get_urls("http://b1twis3.ca/burpsuite-30-pro-tips/");
}

#[tokio::main]
async fn get_urls(option: LinkOptions,fetched_urls: &mut Vec<String>,_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Start Scraping
    let target_url = _url;
    let mut headers = HeaderMap::new();
    let client = reqwest::Client::builder().build()?;
    headers.insert(reqwest::header::USER_AGENT,HeaderValue::from_str("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:80.0) Gecko/20100101 Firefox/80.0").unwrap());
    let resp = client
        .get(target_url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let body = resp;
    let fragment = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    let mut urls = Vec::new();
    for element in fragment.select(&selector){
        //println!("[ULR]: {}",element.value().attr("href").unwrap());
        urls.push(element.value().attr("href").unwrap());
    }

    // Cleaning the URLs vector
    let mut urls: Vec<_> = urls.into_iter().unique().collect();
    for i in urls{


        // Filtering Internal and External URLs
        let mut parsed_target = Url::parse(target_url)?;
        if(parsed_target.join(i)?.path() != "/"){
            let mut check_url = parsed_target.join(i)?;

        match option{
                LinkOptions::INTERNAL => {
                    // Internal Links
                    if parsed_target.host_str().unwrap() == check_url.host_str().unwrap() {
                        if !check_url.path().contains("ailto"){
                            fetched_urls.push(check_url.as_str().to_string());
                        }

                    // Relative Path
                    if None == parsed_target.join(i)?.host_str(){
                        let jurl = parsed_target.join(i)?;
                        fetched_urls.push(jurl.as_str().to_string());
                        }
                    }
                }

               LinkOptions::EXTERNAL => {

                // External Links
                if parsed_target.host_str().unwrap() != parsed_target.join(i)?.host_str().unwrap(){
                    //println!("external: {}",i);
                    fetched_urls.push(i.to_string());
                    // Ingore (for now)
                    }
               }

            }
        }
    }

    Ok(())

}

