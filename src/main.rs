use reqwest::Error;
use rust_decimal::Decimal;
use select::document::Document;
use select::predicate::{Attr, Name};

fn main() {
    let ean = String::from("3162420674272");
    let url = format!("https://www.google.com/search?q={}&tbm=shop", ean);
    let compare_page_url = extract_compare_page_url(get_html(url).unwrap(), ean.clone()).unwrap();
    let prices = extract_prices(get_html(compare_page_url).unwrap());
    let price_calculation = calculate_price_calculation(prices);

    println!(
        "{},{},{},{}",
        ean, price_calculation.min, price_calculation.max, price_calculation.average
    );
}

fn get_html(url: String) -> Result<String, Error> {
    return reqwest::blocking::get(url)?.text();
}

fn extract_compare_page_url(html: String, ean: String) -> Result<String, String> {
    let doc = Document::from(&html[..]);
    for tbody in doc.find(Name("a")).filter_map(|a| a.attr("href")) {
        if tbody.contains("product/") && tbody.contains(&ean) {
            return Ok(format!(
                "{}{}",
                "https://www.google.com/",
                String::from(tbody)
            ));
        }
    }
    return Err("no `compare_page_url` found".to_string());
}

fn extract_prices(html: String) -> Vec<Price> {
    let mut prices: Vec<Price> = Vec::new();
    let doc = Document::from(&html[..]);
    for node in doc.find(Attr("id", "online")) {
        for b_elem in node.find(Name("b")) {
            prices.push(Price {
                item: item_price(b_elem),
                shipping: extract_shipping_price(b_elem),
            });
        }
    }

    return prices;
}

fn extract_shipping_price(b_elem: select::node::Node) -> Decimal {
    let value = b_elem
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .last_child()
        .unwrap()
        .text()
        .chars()
        .skip(1) // skip the "+" sign
        .take_while(|&c| c != ' ') // take characters until a space is encountered
        .collect::<String>()
        .replace(",", ".")
        .replace("€", "");

    return Decimal::from_str_exact(value.trim()).unwrap();
}

fn item_price(b_elem: select::node::Node) -> Decimal {
    let val = b_elem.text().replace(",", ".").replace("€", "");
    return Decimal::from_str_exact(val.trim()).unwrap();
}

fn calculate_price_calculation(prices: Vec<Price>) -> PriceCalculation {
    let mut items: Vec<Decimal> = prices.iter().map(|price| price.item).collect();
    let _shippings: Vec<Decimal> = prices.iter().map(|price| price.shipping).collect();

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

struct Price {
    item: Decimal,
    shipping: Decimal,
}

struct PriceCalculation {
    min: Decimal,
    max: Decimal,
    average: Decimal,
}
