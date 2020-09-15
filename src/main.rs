use reqwest;
use scraper::{Html, Selector};
use std::io::prelude::*;
use itertools::Itertools;
use http::{HeaderMap, HeaderValue};
use url::{Url};
//use std::thread;
use std::io::BufReader;
use std::fs::File;
//use std::io;
//use std::path::Path;


/// Link Options Enum
enum LinkOptions{
    INTERNAL,
    EXTERNAL,
    ALL,
}

/// Fetched Url Struct
//#[derive(Debug, Copy, Clone)]
//struct FUrl{
 //url: Vec<String>,
//}

/// Reading lines from a file
fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(
        reader.lines().filter_map(Result::ok).collect()
    )
}

/// Build site map
fn build_sitemap(_index: usize, _urls: &mut Vec<String>, _sitemap: &mut Vec<Vec<String>>){
    let mut v: Vec<String> = Vec::new();
        for url in _urls{
            let item = url.split("/").collect::<Vec<&str>>();
            if _index <= item.len() -1 {
                if _index == 3{
                    _sitemap.push(vec!(item[_index].to_string()));
                }
                if _index > 3 {
                    for i in 3.._index+1{
                        v.push(item[i].to_string());
                    }
                    println!("v: {:?}",v.clone());
                    _sitemap.push((v.clone()));    
                    v.clear();
                }
            }
       }
}

/// Add endpoints to the site map
fn add_endpoints(_index: usize, _urls: &mut Vec<String>,_sitemap: &mut Vec<Vec<String>>, _endpoints: Vec<String>){
    
        //println!("{:?}",endpoints);
  
    // Clean Duplicates in Site map and fetch route levels
    let mut sitemap: Vec<Vec<String>> = Vec::new();
    // Fetch the routes form `3` to `_index`  
    build_sitemap(_index, _urls,_sitemap);
    let clean_sitemap: Vec<Vec<String>> = _sitemap.clone().into_iter().unique().collect();
    //println!("clean_sitemap: {:?}",clean_sitemap.clone());

    for i in clean_sitemap{
        println!("sitemap: {}",i[1]);
    }
    //

    //for i in clean_sitemap {
     //   for endpoint in &_endpoints {
      //      if i[0] != ""{
       //     println!("Path: {}/{}",i[0],endpoint);
        //    }
        //}

    //}
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Start Scrapping.......");
    let mut fetched_urls: Vec<String> = Vec::new();
    let mut sitemap: Vec<Vec<String>> = Vec::new();
   
    // Start Scarping
    get_urls(LinkOptions::INTERNAL, &mut fetched_urls,"http://b1twis3.ca");
    //println!("fetched: {:?}",fetched_urls);
    
    // Getting Endpoints/Wordlist froma file
    let endpoints: Vec<String> = read_lines("test.txt").unwrap();


    //for i in fetched_urls.clone(){
     //   println!("Scarping {}",i);
      //  get_urls(LinkOptions::INTERNAL, &mut fetched_urls,&i);
    //}
    for i in 3..5{
        println!(":::::::::: Depth: {} ::::::::::::::",i-3);
        add_endpoints(i,&mut fetched_urls, &mut sitemap, endpoints.clone());
        sitemap.clear();
    }
    //println!("{:?}",sitemap);


    Ok(())
}

/// Fetch URLs based on `LinkOptions` and save them into `fetched_urls` vector
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

    // Selector & Element
    let target_tags = vec!["a","link","script","img"];
    let mut urls = Vec::new();

    target_tags.iter().map( |tag| {
        let selector = Selector::parse(tag).unwrap();

        for element in fragment.select(&selector){
            match tag {
               &"a" => {
                    urls.push(element.value().attr("href").unwrap());
                    //println!("[a]: {}",element.value().attr("href").unwrap());
                }

               &"link" => {

                   urls.push(element.value().attr("href").unwrap());
                   //println!("[href]: {}",element.value().attr("href").unwrap());
               }

               &"script" => {
                   match element.value().attr("src"){
                       None => {

                       }
                       _ => {
                            //println!("[SRC]: {}",element.value().attr("src").unwrap());
                            urls.push(element.value().attr("src").unwrap());
                       }
                   }
               }
               &"img" => {
                   //println!("[IMG]: {}",element.value().attr("src").unwrap());
                   urls.push(element.value().attr("src").unwrap());
               }
               _ => {

               }
            }

        }

    }).unique().collect::<()>();


    //-- End of Selector

    // Cleaning the URLs vector
    //let mut urls: Vec<String> = urls.iter().unique().collect::<()>();
    //println!("urls: {:?}",urls);

    for i in urls{


        // Filtering Internal and External URLs
        let parsed_target = Url::parse(target_url)?;
        if parsed_target.join(&i)?.path() != "/" {
            let check_url = parsed_target.join(&i)?;

        match option{

               LinkOptions::INTERNAL => {
                    // Internal Links
                    if parsed_target.host_str().unwrap() == check_url.host_str().unwrap() {
                        if !check_url.path().contains("ailto"){
                            fetched_urls.push(check_url.as_str().to_string());
                        }

                    // Relative Path
                    if None == parsed_target.join(&i)?.host_str(){
                        let jurl = parsed_target.join(&i)?;
                        fetched_urls.push(jurl.as_str().to_string());
                        }
                    }
                }


               LinkOptions::EXTERNAL => {
                    // External Links
                    if parsed_target.host_str().unwrap() != parsed_target.join(&i)?.host_str().unwrap(){
                        //println!("external: {}",i);
                        fetched_urls.push(i.to_string());
                        // Ingore (for now)
                        }
               }

               LinkOptions::ALL => {
                   // External and Internal Links

                   // Internal Links
                    if parsed_target.host_str().unwrap() == check_url.host_str().unwrap() {
                        if !check_url.path().contains("ailto"){
                            fetched_urls.push(check_url.as_str().to_string());
                        }

                    // Relative Path
                    if None == parsed_target.join(&i)?.host_str(){
                        let jurl = parsed_target.join(&i)?;
                        fetched_urls.push(jurl.as_str().to_string());
                        }
                    }

                  // External Links
                  if parsed_target.host_str().unwrap() != parsed_target.join(&i)?.host_str().unwrap(){
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

