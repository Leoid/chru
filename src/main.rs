use reqwest;
use scraper::{Html, Selector};
use std::io::prelude::*;
use itertools::Itertools;
use http::{HeaderMap, HeaderValue};
use url::{Url, ParseError};
//use std::thread;
use std::io::BufReader;
use std::fs::File;
//use std::io;
//use std::path::Path;
use regex::Regex;
use std::collections::HashSet;




/// Link Options Enum
enum LinkOptions{
    INTERNAL,
    EXTERNAL,
    ALL,
}


/// Root route
const ROOT: usize = 3;

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
fn build_segmented_sitemap(_index: usize, _urls: &mut Vec<String>, _sitemap: &mut Vec<Vec<String>>){
    let mut v: Vec<String> = Vec::new();
        for url in _urls{
            let item = url.split("/").collect::<Vec<&str>>();
            if _index <= item.len() -1 {
                // Skipping the empty or the "/" at the end of each vector
                if item[_index] != ""{
                if _index == ROOT{
                        if item[_index].to_string().contains(".") || item[_index].to_string().contains("#")
                        || item[_index].to_string().contains("?"){
                            // Skip a file
                            continue;
                        }
                    _sitemap.push(vec!(item[_index].to_string()));
                }
                if _index > ROOT {
                    for i in ROOT.._index+1{
                        // Skipping the filename
                        if item[i].to_string().contains(".") || item[i].to_string().contains("#")
                        || item[_index].to_string().contains("?"){
                            // Skip a file
                            continue;
                        }
                        v.push(item[i].to_string());
                    }
                    //println!("v: {:?}",v.clone());
                    //let item: Vec<_> = v.clone().into_iter().unique().collect();
                    _sitemap.push(v.clone());
                    v.clear();
                }
                }

            }
       }
}

/// Add endpoints to the site map
fn add_endpoints(_sitemap: &mut Vec<Vec<String>>, _endpoints: Vec<String>) -> Vec<Vec<String>> {


    // Get the cleaned site map and append the endpoints from `endpoints`
    let clean_sitemap: Vec<Vec<String>> = _sitemap.clone().into_iter().unique().collect();
    let mut endpoints_vec2: Vec<Vec<String>> = Vec::new();
    //println!("sitemap: {:?}",clean_sitemap);
    for i in clean_sitemap{
        for endpoint in &_endpoints {

            let mut endpoints_vec: Vec<String> = Vec::new();
            for ii in &i {
                if ii != ""{
                  //print!("{}/",ii);
                  endpoints_vec.push(format!("{}/",ii));
                }
            }
            endpoints_vec.push(format!("{}",endpoint).to_string());
            //print!("{}\n",endpoint);
            endpoints_vec2.push(endpoints_vec);
        }
    }
    endpoints_vec2

}
/// Extract URLs from JS file
#[tokio::main]
async fn extract_urls(target_url: &str,extracted: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let re: Regex = Regex::new(r"[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();


    // Make a request
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
    Ok(
        extracted.push(re.find_iter(&resp).map(|u| u.as_str()).collect())
      )

}
fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("Start Scrapping.......");

    // Arguments
    let mut fetched_urls: Vec<String> = Vec::new();
    let mut sitemap: Vec<Vec<String>> = Vec::new();
    //let target = "https://pwm.oddo-bhf.com";
    //let target = "http://b1twis3.ca";
    let target = "http://216.177.93.235";
    let depth = 10;
    //let tweet = "https://google.com hello /test/test.php /api/v1/ /index.html";
    //let tag = extract_urls(tweet);
    //println!("tags = {:?}",tag);


    // Start Scarping
    get_urls(LinkOptions::ALL, &mut fetched_urls,target);
    //get_urls(LinkOptions::INTERNAL, &mut fetched_urls,"http://b1twis3.ca/wp-includes/css/dist/block-library/style.min.css?ver=5.4.2");
    //println!("fetched: {:?}",fetched_urls);

    // Getting Endpoints/Wordlist froma file
    let endpoints: Vec<String> = read_lines("test.txt").unwrap();

    // Do segmentation
    build_segmented_sitemap(depth,&mut fetched_urls,&mut sitemap);


    let mut test_url = fetched_urls.clone();
    //println!("test_url: {:?}",test_url);
    let url1 = &test_url[..];

    for i in ROOT..depth{
        // Build Site map from `fetched_urls` and add for each route a line from `endpoints` file.
        // Starting from 3 because we're splitting the URL, `10` is the Depth, which can be changed
        // later on
        build_segmented_sitemap(i,&mut fetched_urls,&mut sitemap);
   }

    let mut new_sitemap: Vec<Vec<String>> = add_endpoints(&mut sitemap, endpoints.clone());

    // Dogin Segmentation and adding endpoints to the inner URLs
    for i in url1 {
        if i != ""{

            // Fetching JS files
            let js_path = Url::parse(i)?;
            if js_path.path().contains(".js"){
                println!("path = {}{}",target,js_path.path());
                //let mut extracted = Vec::new();
                //extract_urls(&format!("{}{}",target,js_path.path()),&mut extracted);

                //println!("Extracted: {:?}",extracted);
            }

        get_urls(LinkOptions::ALL, &mut fetched_urls,i);
        for i in ROOT..depth{
            // Build Site map from `fetched_urls` and add for each route a line from `endpoints` file.
            // Starting from 3 because we're splitting the URL, `10` is the Depth, which can be changed
            // later on

            build_segmented_sitemap(i,&mut fetched_urls,&mut sitemap);
        }
        new_sitemap.append(&mut add_endpoints( &mut sitemap, endpoints.clone()));
        }
    }

    //println!("New endpoints:::::: {:?}",new_endpoints);
    println!("New Sitemap Len: {}",new_sitemap.len());
    let unique_sitemap: Vec<Vec<String>> = new_sitemap.clone().into_iter().unique().collect();
    println!("Unique Sitemap Len: {}",unique_sitemap.len());
    for nn in unique_sitemap{
        print!("{}/",target);
        for ii in nn{
            print!("{}",ii);
        }
        println!("");
    }



    //println!("sitemap: {:?}", _sitemap);


    //println!("sitemap: {:?}",&sitemap);

    //get_urls(LinkOptions::INTERNAL, &mut fetched_urls.clone(), &fetched_urls[0]);
    //println!("fetched_url[0] = {}",&fetched_urls[5]);
    //println!("{:?}",sitemap);


    Ok(())
}

/// Fetch URLs based on `LinkOptions` and save them into `fetched_urls` vector
#[tokio::main]
async fn get_urls(option: LinkOptions,fetched_urls: &mut Vec<String>,_url: &str) -> Result<(), Box<dyn std::error::Error>> {

    // Start Scraping
    let target_url = _url;
    //println!("target____url: {}",target_url);
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
    //println!("{:?}",&fragment.errors);
    //println!("{:?}",body);
    // Selector & Element
    let target_tags = vec!["a","link","script","img","form"];
    let mut urls: Vec<String> = Vec::new();
    // Check out this later ---------------
    //if fragment.clone().errors.len()  == 99999999 as usize {
    if fragment.clone().errors.len() <= 3 {
     //   assert!(true);
    //}
    target_tags.iter().map( |tag| {
        let selector = Selector::parse(tag).unwrap();
        for element in fragment.select(&selector){
            match tag {
                &"form" => {
                    match element.value().attr("action") {
                        Some(u) => {
                                urls.push(element.value().attr("action").unwrap().to_string());
                                //println!("[form]: {}",element.value().attr("action").unwrap().to_string());

                        }
                        _ => {}
                    }
                 }


               &"a" => {
                    urls.push(element.value().attr("href").unwrap().to_string());
                    //println!("[a]: {}",element.value().attr("href").unwrap());
                }

               &"link" => {

                   urls.push(element.value().attr("href").unwrap().to_string());
                   //println!("[href]: {}",element.value().attr("href").unwrap());
               }

               &"script" => {
                   match element.value().attr("src"){
                       None => {

                       }
                       _ => {
                            //println!("[SRC]: {}",element.value().attr("src").unwrap());
                            urls.push(element.value().attr("src").unwrap().to_string());
                       }
                   }
               }
               &"img" => {
                   //println!("[IMG]: {}",element.value().attr("src").unwrap());
                   urls.push(element.value().attr("src").unwrap().to_string());
               }
               _ => {

               }
            }

        }

    }).unique().collect::<()>();


    //-- End of Selector

    // Cleaning the URLs vector
    //println!("lennn: {}",urls.len());
    let _urls: Vec<String> = urls.clone().into_iter().unique().collect();
    //println!("urls: {:?}",urls);
    for i in _urls{

        // Filtering Internal and External URLs
        let parsed_target = Url::parse(target_url)?;
        //println!("host {:?}",target_url);
        if parsed_target.join(&i)?.path() != "/" {
            let check_url = parsed_target.join(&i)?;
        match check_url.host_str(){
        Some(ok_url) => {
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

           None => { }
        }
        }
    }
    }




    Ok(())

}

