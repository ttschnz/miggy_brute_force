#![deny(warnings)]

extern crate termsize;

use serde_json::Value;
use colored::Colorize;
use std::io;
use std::io::Write;

#[cfg(target_os = "windows")]
extern crate ansi_term;
// This is using the `tokio` runtime. You'll need the following dependency:
//
// `tokio = { version = "1", features = ["full"] }`
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    use std::{time::Duration};
    // only run ansi_term::enable_ansi_support if the platform is windows
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support().expect("Failed to enable ANSI support");

    let mut cookies = String::new();
    println!("Head over to (https://win.migros.ch/promos/) and paste the cookie data from your browser: ");
    std::io::stdin().read_line(&mut cookies).unwrap();
    let cookies = cookies.trim().to_string();
    if cookies.is_empty() {
        // return with an error if the cookie is empty
        println!("cookie is empty, exiting\n\n");
    }else{
        // Get some attributes from the user.
        let mut filiale = String::new();
        println!("Enter the filiale (e.g. \"0033810\"): ");
        std::io::stdin().read_line(&mut filiale).unwrap();
        filiale = filiale.trim().to_string();
        if filiale.is_empty(){
            filiale = "0033810".to_string();
        }
        let filiale = filiale.trim();

        let mut date = String::new();
        println!("Enter the date (e.g. 13.06.22): ");
        std::io::stdin().read_line(&mut date).unwrap();
        date = date.trim().to_string();
        if date.is_empty(){
            date = "13.06.22".to_string();
        }
        // remove all dots from the date and make it immutable
        date = date.replace(".", "");
        let date = date.trim();


        let mut kasse = String::new();
        println!("Enter the kasse (e.g. \"254\"): ");
        std::io::stdin().read_line(&mut kasse).unwrap();
        kasse = kasse.trim().to_string();
        // if the value is empty, use the default value
        if kasse.is_empty() {
            kasse = "254".to_string();
        }
        let kasse = kasse.trim();

        let mut bon_lower_bound = String::new();
        println!("Enter the lower bound of the bon (e.g. \"0\"): ");
        std::io::stdin().read_line(&mut bon_lower_bound).unwrap();
        // Convert the string to a number.
        let bon_lower_bound = bon_lower_bound.trim().parse::<i64>().unwrap_or(0);

        let mut bon_upper_bound = String::new();
        println!("Enter the upper bound of the bon (e.g. \"200\"): ");
        std::io::stdin().read_line(&mut bon_upper_bound).unwrap();
        // Convert the string to a number, use "30.95" as default value.
        let mut bon_upper_bound = bon_upper_bound.trim().parse::<i64>().unwrap_or(200);
        // check if upper_bound is greater than lower_bound
        if bon_upper_bound < bon_lower_bound {
            println!("upper bound is smaller than lower bound, using default value: \"+200\"\n\n");
            bon_upper_bound = bon_lower_bound + 200;
        }
        let bon_upper_bound = bon_upper_bound;
        
        let mut price_lower_bound = String::new();
        println!("Enter the lower bound of the price (e.g. \"0\"): ");
        std::io::stdin().read_line(&mut price_lower_bound).unwrap();
        // Convert the string to a number.
        let price_lower_bound = price_lower_bound.trim().parse::<f64>().unwrap_or(0.0);

        let mut price_upper_bound = String::new();
        println!("Enter the upper bound of the price (e.g. \"200\"): ");
        std::io::stdin().read_line(&mut price_upper_bound).unwrap();
        // Convert the string to a number, use "30.95" as default value.
        let mut price_upper_bound = price_upper_bound.trim().parse::<f64>().unwrap_or(200.0);
        // check if upper_bound is greater than lower_bound
        if price_upper_bound < price_lower_bound {
            println!("upper bound is smaller than lower bound, using default value: \"+200\"\n\n");
            price_upper_bound = price_lower_bound + 200.0;
        }
        let price_upper_bound = price_upper_bound;

        let mut price_stepping = String::new();
        println!("Enter the price stepping (e.g. \"0.05\"): ");
        std::io::stdin().read_line(&mut price_stepping).unwrap();
        // Convert the string to a float
        let mut price_stepping = price_stepping.trim().parse::<f64>().unwrap_or(0.05);
        // check if the price stepping is greater than 0.01
        if price_stepping < 0.01 {
            println!("price stepping is smaller than 0.01, using default value: \"0.05\"\n\n");
            price_stepping = 0.05 as f64;
        }
        let price_stepping = price_stepping;


        // generate the code from this scheme: "{}{}{}{}<preis>", filiale, kasse, date, bon
        let mut code_map = String::new();
        code_map.push_str("0101");
        code_map.push_str(&filiale);
        code_map.push_str(&kasse);
        code_map.push_str(&date);
        // code_map.push_str(&bon);
        
        println!("generating {} codes...", get_code_count(&bon_lower_bound, &bon_upper_bound, &price_lower_bound, &price_upper_bound, &price_stepping));

        let codes = generate_codes(&bon_lower_bound, &bon_upper_bound, &price_lower_bound, &price_upper_bound, &price_stepping ,&code_map);
        
        println!("will check {} codes", codes.len());    
        
        let url = "https://win.migros.ch/promos/skins/fruehlingspromo/services/play.php";
        
        
        
        // iterate over all codes
        for i in 0..codes.len() {
            let code = &codes[i];
            if i>0 && i % 95 == 0 {
                // sleep 10s
                for j in 0..60{
                    update_line(&format!("waiting for timeout to prevent blocking: {}s left...", 60-j), false);
                    std::thread::sleep(Duration::from_secs(1));
                }
                update_line("timeout passed, continuing.", true);

            }
            let mut current_line = format!("checking code: \"{}\"", code);

            update_line(&current_line, false);

            let result = make_request(&url, code, &cookies).await;

            if result["errors"] == "" {
                if result["win"]==true{
                    current_line = format!("{} => {}", current_line, (format!("won!").green()));
                }else{
                    current_line = format!("{} => {}", current_line, (format!("redeemed code, no win.").blue()));
                }
            } else {
                if result["loggedIn"] == false{
                    current_line = format!("{} => {}", current_line, (format!("not logged in.").red()));
                }else if result["errors"] == "Dieser Kassenbon hat bereits teilgenommen." {
                    current_line = format!("{} => {}", current_line, (format!("code already used.").red()));
                } else if result["errors"] == "Diese Nummer ist leider ungültig."{
                    current_line = format!("{} => {}", current_line, (format!("code is invalid! Please check configuration.").red()));
                }else{
                    let errmsg = result["errors"].to_string().replace("\"", "");
                    let errmsg = errmsg.split(".");
                    let errmsg = errmsg.collect::<Vec<&str>>();
                    let errmsg = errmsg[0].to_string();
                    current_line = format!("{} => {}", current_line, (format!("{} {}", format!("error:").red(), errmsg)));
                }
            }
            update_line(&current_line, true);
        }
    }
    Ok(())
}

fn update_line(content: &str, last:bool) {
    termsize::get().map(|size| {
        let mut size_to_fill = 0;
        if size.cols > content.len() as u16{
            size_to_fill = size.cols - content.len() as u16;
        }
        // let size_to_fill = 4;
        // print the content and fill with spaces
        print!("\r{}", content);
        for _ in 0..size_to_fill {
            print!(" ");
        }
        io::stdout().flush().unwrap();
        if last {
            println!();
        }
    });
}


fn get_code_count(bon_lower_bound: &i64, bon_upper_bound: &i64, price_lower_bound: &f64, price_upper_bound: &f64, price_stepping: &f64) -> i64 {
    return (bon_upper_bound - bon_lower_bound) * ((price_upper_bound - price_lower_bound) / price_stepping) as i64;
}

async fn make_request(url: &str, code: &String, cookies: &String) -> Value {
    // create a client
    let client = reqwest::Client::new();
    // create a request
    let req = client.get(url);
    
    // add cookie headers
    let mut headers = reqwest::header::HeaderMap::new();
    let cookie_value = reqwest::header::HeaderValue::from_str(cookies).expect("Cookie header is invalid");
    headers.insert(reqwest::header::COOKIE, cookie_value);
    // add headers to the request
    let req = req.headers(headers);

    // add the code to the request
    let req = req.query(&[("code", code)]);

    // send the request
    let res = req.send().await.unwrap();
    // get the body of the response
    let body = res.text().await.unwrap_or("{}".to_string());
    
    // parse the body to get the price
    let json: Value = serde_json::from_str(&body).expect("Failed to read response");
    // return the json
    return json;
}

/// # Generate a list of codes for the given parameters.
/// ## The codes are generated in the following way:
/// 1. Iterate over all possible bons in the range between lower_bound and upper_bound.
/// 2. Iterate over all possible prices in the range between lower_bound and upper_bound (using the price_stepping).
/// 3. Generate a code for each combination of bon and price.
/// 4. Return the list of codes.
/// ## Parameters:
/// - bon_lower_bound: The lower bound of the bon range.
/// - bon_upper_bound: The upper bound of the bon range.
/// - price_lower_bound: The lower bound of the price range.
/// - price_upper_bound: The upper bound of the price range.
/// - price_stepping: The price stepping.
/// - code_map: The code map.
/// ## Returns:
/// - The list of codes.
/// 
fn generate_codes(bon_lower_bound: &i64, bon_upper_bound: &i64, price_lower_bound: &f64, price_upper_bound: &f64, price_stepping: &f64, code_map: &String) -> Vec<String> {
    let mut codes = Vec::new();
    let code_count = get_code_count(&bon_lower_bound, &bon_upper_bound, &price_lower_bound, &price_upper_bound, &price_stepping);
    let mut codes_left = code_count.clone();
    // get the count of prices to check (ceiling of the division)
    let price_count= ((price_upper_bound - price_lower_bound) / price_stepping).ceil() as i64;
    for i in bon_lower_bound.clone()..bon_upper_bound.clone() {
        for j in 0..price_count {
            if codes_left % 50000 == 0 {
                update_line(&format!("{}%", (100.0 * (code_count - codes_left) as f64 / code_count as f64)), false);
            }
            let current_price = price_lower_bound.clone() + (j as f64 * price_stepping.clone());
            let bon = i;
            let code = generate_code(bon,  current_price, code_map);
            codes.push(code);
            codes_left -= 1;
        }
    }
    update_line("done.", true);
    return codes;
}

fn generate_code(bon_code: i64, price: f64, code_map: &String ) -> String {
  // generate the code from this scheme: "{}{}{}{}<preis>", filiale, kasse, date, bon
  let bon = format!("{:0>4}", bon_code);
  //   println!("price: {}", price);
  let price = price.to_string();
  // split the price into a vec with two parts: the first part is the whole number part, the second part is the fractional part
  let mut price_parts: Vec<&str> = price.split(".").collect();
  if price_parts.len() == 1 {
      price_parts.push("00");
  }

  // let price_parts = price.to_string().split(".").collect::<Vec<&str>>
  let decimal = format!("{:0<2}", format!("{:.2}", price_parts[1]));
//   println!("decimal: {}", decimal);
  let price = price_parts[0].to_string() + "." + &decimal;
  let price = price.replace(".", "");
  // pad the price with zeros to the left
  let price = format!("{:0>9}", price);

  let code = code_map.clone() + &bon +  &price;
  return code;
}
// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}