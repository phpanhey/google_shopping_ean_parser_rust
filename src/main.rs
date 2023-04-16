use reqwest::Error;
use rust_decimal::Decimal;
use select::document::Document;
use select::predicate::{Attr, Name};
use std::env;
fn main() {
    if ean_incorrect() {
        std::process::exit(1);
    }
    let ean = get_ean();
    let url = format!("https://www.google.com/search?q={}&tbm=shop", ean);
    let compare_page_url = extract_compare_page_url(get_html(url).unwrap(), ean.clone()).unwrap();
    let data_item = extract_data(get_html(compare_page_url).unwrap());
    println!("{},{}", ean, data_item.to_string());
}

fn get_html(url: String) -> Result<String, Error> {
    return reqwest::blocking::get(url)?.text();
}

fn extract_compare_page_url(html: String, ean: String) -> Result<String, String> {
    let doc = Document::from(&html[..]);
    for anchor in doc.find(Name("a")).filter_map(|a| a.attr("href")) {
        if anchor.contains("product/") && anchor.contains(&ean) {
            return Ok(format!(
                "{}{}",
                "https://www.google.com/",
                String::from(anchor)
            ));
        }
    }
    return Err("no `compare_page_url` found".to_string());
}

fn extract_data(html: String) -> DataItem {
    let mut prices: Vec<Decimal> = Vec::new();
    let doc = Document::from(&html[..]);
    for node in doc.find(Attr("id", "online")) {
        for b_elem in node.find(Name("b")) {
            prices.push(item_price(b_elem));
        }
    }
    let calculation = calculate_price_calculation(prices.clone());
    return DataItem {
        price_min: calculation.min,
        price_max: calculation.max,
        price_average: calculation.average,
        prices: prices,
    };
}

fn item_price(b_elem: select::node::Node) -> Decimal {
    let val = b_elem
        .text()
        .replace(".", "")
        .replace(",", ".")
        .replace("€", "");
    return Decimal::from_str_exact(val.trim()).unwrap();
}

fn calculate_price_calculation(prices: Vec<Decimal>) -> PriceCalculation {
    let mut items = prices;
    items.sort();

    let first = items.first().unwrap();
    let last = items.last().unwrap();

    let sum: Decimal = items.iter().sum();
    let average = sum / Decimal::new(items.len() as i64, 0);

    return PriceCalculation {
        min: *first,
        max: *last,
        average: average,
    };
}
struct PriceCalculation {
    min: Decimal,
    max: Decimal,
    average: Decimal,
}
struct DataItem {
    price_min: Decimal,
    price_max: Decimal,
    price_average: Decimal,
    prices: Vec<Decimal>,
}

impl DataItem {
    fn to_string(&self) -> String {
        format!(
            "{},{},{},{}",
            self.price_min,
            self.price_max,
            self.price_average,
            self.prices
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

fn ean_incorrect() -> bool {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        return true;
    }
    let ean = &args[1];
    return ean == "ean" || ean == "";
}

fn get_ean() -> String {
    return env::args().nth(1).expect("Missing argument");
}
