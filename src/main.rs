use reqwest::Error;
use select::document::Document;
use select::predicate::{Attr, Name};

fn main() {
    let ean = String::from("3162420172969");
    let url = format!("https://www.google.com/search?q={}&tbm=shop", ean);
    println!("{}",url);
    let compare_page_url = extract_compare_page_url(get_html(url).unwrap(), ean).unwrap();
    let _prices = extract_prices(get_html(compare_page_url).unwrap());
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

fn extract_prices(html: String) -> Prices {
    let mut prices = Prices {
        items: Vec::new(),
        shippings: Vec::new(),
    };
    let doc = Document::from(&html[..]);
    for node in doc.find(Attr("id", "online")) {
        for b_elem in node.find(Name("b")) {
            let price = item_price(b_elem);
            let shipping_price = extract_shipping_price(b_elem);

            prices.items.push(price);
            prices.shippings.push(shipping_price);

            println!("Price: {} Shipping Price: {}", price, shipping_price);
        }
    }
    return prices;
}

fn extract_shipping_price(b_elem: select::node::Node) -> f32 {
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

    return value.trim().parse::<f32>().unwrap();
}

fn item_price(b_elem: select::node::Node) -> f32 {
    return b_elem
        .text()
        .replace(",", ".")
        .replace("€", "")
        .trim()
        .parse::<f32>()
        .unwrap();
}

struct Prices {
    items: Vec<f32>,
    shippings: Vec<f32>,
}
