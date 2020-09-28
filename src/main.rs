use reqwest;
use scraper::{Html, Selector};
use std::io::prelude::*;
use itertools::Itertools;
use http::{HeaderMap, HeaderValue};
use url::{Url};
use std::io::BufReader;
use std::fs::File;
use regex::Regex;
use std::iter::FromIterator;
use futures;
use futures::stream::{StreamExt};
use structopt::StructOpt;
use std::str::FromStr;
use std::string::ParseError;





/// Link Options Enum
#[derive(Copy, Clone, Debug)]
enum LinkOptions{
    INTERNAL,
    EXTERNAL,
    ALL,
}

impl FromStr for LinkOptions{
    type Err = ParseError;
    fn from_str(link: &str) -> Result<Self, Self::Err>{
        match link {
            "I" => Ok(LinkOptions::INTERNAL),
            "E" => Ok(LinkOptions::EXTERNAL),
            "A" => Ok(LinkOptions::ALL),
            _ => Ok(LinkOptions::INTERNAL),
        }
    }
}

#[derive(Debug)]
pub enum Error {
       Request(reqwest::Error),
       Status(reqwest::StatusCode)
}

/// Root route
const ROOT: usize = 3;

/// Filter Text Struct
#[derive(Debug, Copy, Clone)]
struct Ftext<'a>{
    /// Filter responses based on this string
    filter: &'a str,
}

/// Fetch URLs in a target and append endpoints for each path/route
#[derive(Debug, StructOpt)]
struct Cli{
    /// Target URL
    #[structopt(short="h",long="host")]
    host: String,
    /// Endpoints file path
    #[structopt(short="w",long="wordlist")]
    path: String,
    /// URLs Options to Fetch [Interal=I, External=E or ALL=A]
    #[structopt(short="l",long="link-option", default_value="I")]
    link: LinkOptions,
    /// Segmentation Depth
    #[structopt(short="d", long="depth",default_value="10")]
    depth: usize,
    /// Number of Threads
    #[structopt(short="t",long="threads",default_value="50")]
    nthreads: usize,
    /// Status Code to print
    #[structopt(short="s",long="status-code",default_value="0")]
    status_code: usize,
    /// Text/word in Response
    #[structopt(short="T",long="text",default_value="")]
    filter_text: String,
    /// List of common extensions, such as .js,.txt,.asp.net
    #[structopt(short="e",long="extensions",use_delimiter = true, default_value=" ")]
    ext: Vec<String>,

}

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
            //println!("item: {:?} {}",item,_index);
            if _index <= item.len() -1 {
                // Skipping the empty or the "/" at the end of each vector
                if item[_index] != ""{
                if _index == ROOT{
                        if item[_index].to_string().contains(".") || item[_index].to_string().contains("#")
                        || item[_index].to_string().contains("?") || item[_index].to_string().contains("&"){
                            // Skip a file
                            continue;
                        }
                    _sitemap.push(vec!(item[_index].to_string()));
                }
                if _index > ROOT {
                    for i in ROOT.._index+1{
                        // Skipping the filename
                        if item[i].to_string().contains(".") || item[i].to_string().contains("#")
                        || item[i].to_string().contains("?") || item[i].to_string().contains("&"){
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
                //if _ursl.len() == 0{
                 //   v.push("/".to_string());
                  ////  _sitemap.push(v.clone());
                    //v.clear();
                //}

            }
       }
}

/// Add endpoints to the site map
fn add_endpoints(ext: Vec<String>, _sitemap: &mut Vec<Vec<String>>, _endpoints: Vec<String>) -> Vec<Vec<String>> {


    // Get the cleaned site map and append the endpoints from `endpoints`
    let clean_sitemap: Vec<Vec<String>> = _sitemap.clone().into_iter().unique().collect();
    let mut endpoints_vec2: Vec<Vec<String>> = Vec::new();

    //println!("sitemap: {:?}",clean_sitemap);

    // Add endpoints at least once
    if clean_sitemap.len() == 0{
        for endpoint in &_endpoints {
            for e in &ext{
                let mut endpoints_vec: Vec<String> = Vec::new();
                endpoints_vec.push(format!("{}{}",endpoint,e).to_string());
                endpoints_vec2.push(endpoints_vec);
            }
        }

    }

    for i in clean_sitemap{
        for endpoint in &_endpoints {
            for e in &ext{

                let mut endpoints_vec: Vec<String> = Vec::new();
                for ii in &i {
                    if ii != ""{
                      //print!("{}/",ii);
                      endpoints_vec.push(format!("{}/",ii));
                    }
                }
                endpoints_vec.push(format!("{}{}",endpoint,e).to_string());
                //print!("{}\n",endpoint);
                endpoints_vec2.push(endpoints_vec);
            }
        }
    }
    endpoints_vec2

}
/// Extract URLs from JS file
#[tokio::main]
async fn extract_urls(target_url: &str,extracted: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Got this regex from /gospider
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

/// Check the HTTP Request
#[tokio::main]
async fn check_request(filter_text: Ftext,s_code: u16,nthreads: usize,target: &str,sitemap: Vec<Vec<String>>) -> Result<(), Box<dyn std::error::Error>> {
        let fetches = futures::stream::iter(
            sitemap.into_iter().map(|ii| {
            //for ii in sitemap{
                async move {

                    // Make a request
                    //let mut headers = HeaderMap::new();
                    //Redirect Policy
				/*	let custom = reqwest::redirect::Policy::custom(|attempt| {
						if attempt.previous().len() > 5 {
					  	attempt.error("too many redirects")
						} //else if attempt.url().host_str() == Some("example.domain") {
							// prevent redirects to 'example.domain'
						//	attempt.stop()
						//}
						else {
							attempt.follow()
						}
					});
                    let client = reqwest::Client::builder()
								.redirect(custom)
					  		.build()
								.unwrap();
                                */
                    let path = String::from_iter(ii.clone());
                    let url = format!("{}/{}",target,String::from_iter(ii));
                    //let url = format!("/{}",String::from_iter(ii));
                    //println!("checking URL: {}",&url);

                    //headers.insert(reqwest::header::USER_AGENT,HeaderValue::from_str("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:80.0) Gecko/20100101 Firefox/80.0").unwrap());

                            let resp = match reqwest::get(&url).await{
                            //let resp = match client.get(&url).send().await{
                                Ok(resp) => {
                                    let _ss = resp.status();
                                   if resp.status().as_u16() == s_code || s_code == 0 {
                                                match resp.text().await{
                                                    Ok(text) => {
                                                            if text.contains(filter_text.filter) {
                                                                println!("[+] /{: <60} | {: <60} | {} Bytes",path,_ss,
                                                                text.len()
                                                                ); // end of println!
                                                            }
                                                    }
                                                    _ => {}
                                                }
                                    }
                                }
                                _ => {}
                             // Err(e) => {println!("error: {}",e)}
                            };

                }
            //}
        })
        ).buffer_unordered(nthreads).collect::<Vec<()>>();
        //println!("......");
        fetches.await;

        Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>>{


    //println!("Start Web Scraping.......");

    // Arguments
    let mut fetched_urls: Vec<String> = Vec::new();
    let mut sitemap: Vec<Vec<String>> = Vec::new();

    // Getting Args
    let args = Cli::from_args();
    let target = args.host.as_str();
    let path = args.path.as_str();
    let link = args.link;
    let depth = args.depth;
    let nthreads = args.nthreads;
    let status_code = args.status_code as u16;
    let filter_text = args.filter_text;
    let exts = args.ext;


    // Start Scarping
    get_urls(link, &mut fetched_urls,target);

    //println!("fetched: {:?}",fetched_urls);

    // Getting Endpoints/Wordlist froma file
    let endpoints: Vec<String> = read_lines(path).unwrap();
    let ext: Vec<String> = exts;
    // Do segmentation
    build_segmented_sitemap(depth,&mut fetched_urls,&mut sitemap);

    if fetched_urls.len() == 0{
        fetched_urls.push(format!("{}/",target.to_string()));
    }
    let mut test_url = fetched_urls.clone();
    let url1 = &test_url[..];

    let mut new_sitemap: Vec<Vec<String>> = add_endpoints(ext.clone(), &mut sitemap, endpoints.clone());
    //println!("new_sitemap: {:?}",new_sitemap);
    //println!("test_url: {:?}",url1);


    // Dogin Segmentation and adding endpoints to the inner URLs
    // This Block should be multithreaded
    for i in url1 {
        //println!("checking: {:?}",i);
        if i != ""{

            // Fetching JS files
            //let js_path = Url::parse(i)?;
            //if js_path.path().contains(".js"){
             //   println!("path = {}{}",target,js_path.path());
                //let mut extracted = Vec::new();
                //extract_urls(&format!("{}{}",target,js_path.path()),&mut extracted);

                //println!("Extracted: {:?}",extracted);
            //}

        get_urls(link, &mut fetched_urls,i);
        for i in ROOT..depth{
            // Build Site map from `fetched_urls` and add for each route a line from `endpoints` file.
            // Starting from 3 because we're splitting the URL, `10` is the Depth, which can be changed
            // later on

            build_segmented_sitemap(i,&mut fetched_urls,&mut sitemap);
        }
        new_sitemap.append(&mut add_endpoints(ext.clone(), &mut sitemap, endpoints.clone()));
        }
    }





    //println!("New endpoints:::::: {:?}",new_endpoints);
    //println!("New Sitemap Len: {}",new_sitemap.len());
    println!("[*] Target: {} ",target);
    println!("[*] Number of Threads: {} ",nthreads);
    let unique_sitemap: Vec<Vec<String>> = new_sitemap.clone().into_iter().unique().collect();
    println!("[*] Number of Requests: {}\n",unique_sitemap.len());
    //println!("unique: {:?}",unique_sitemap);
    // Displaying the result
    let filter_text = Ftext{
        filter: &filter_text,
    };
    check_request(filter_text,status_code,nthreads,target,unique_sitemap);



    //println!("sitemap: {:?}", _sitemap);


    //println!("sitemap: {:?}",&sitemap);

    //get_urls(LinkOptions::INTERNAL, &mut fetched_urls.clone(), &fetched_urls[0]);
    //println!("fetched_url[0] = {}",&fetched_urls[5]);
    //println!("{:?}",sitemap);


    Ok(())
}

/// Fetch URLs based on `LinkOptions.` and save them into `fetched_urls` vector
#[tokio::main]
async fn get_urls(option: LinkOptions,fetched_urls: &mut Vec<String>,_url: &str) -> Result<(), Box<dyn std::error::Error>> {

    // Start Scraping
    let target_url = _url;
    //println!("target____url: {}",target_url);
    //let mut headers = HeaderMap::new();
    let client = reqwest::Client::builder().build()?;
    //headers.insert(reqwest::header::USER_AGENT,HeaderValue::from_str("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:80.0) Gecko/20100101 Firefox/80.0").unwrap());
    let resp = match client.get(target_url).send().await?{
        resp => {

            match resp.text().await{
               Ok(body) => {

        //let body = resp;
        let fragment = Html::parse_document(&body);
        //println!("{:?}",&fragment.errors);
        //println!("{:?}",body);
        // Selector & Element
        let target_tags = vec!["a","link","script","img","form"];
        let mut urls: Vec<String> = Vec::new();
        // Check out this later ---------------
        if fragment.clone().errors.len() <= 3 {
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
                        match element.value().attr("href"){
                            Some(u) => {
                        urls.push(element.value().attr("href").unwrap().to_string());
                        //println!("[a]: {}",element.value().attr("href").unwrap());
                            }
                            _ => {}
                        }
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
                        match element.value().attr("src"){
                           None => {

                           }
                           _ => {
                                //println!("[IMG]: {}",element.value().attr("src").unwrap());
                                urls.push(element.value().attr("src").unwrap().to_string());
                           }
                       }
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
        if _urls.len() == 0{
            //println!("000000");
            let parsed_target = Url::parse(target_url)?;
                let check_url = parsed_target.join("/")?;

            fetched_urls.push(check_url.as_str().to_string());
        }
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
                   }
                   _ => {}
                }

        }
        // Display Reqwest error
        //Err(e) => {println!("errrrrrrrrrrr {:?}",e);}
        _ => {}
    };






    Ok(())

}

