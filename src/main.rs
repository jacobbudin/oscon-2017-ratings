#[macro_use] extern crate lazy_static;
extern crate prettytable;
extern crate regex;
extern crate reqwest;
extern crate scraper;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use regex::Regex;
use std::io::Read;
use scraper::{Html, Selector};

struct Event {
    name: String,
    url: String,
    rating: f32,
    review_count: i8,
}

impl Event {
    fn fetch(&self) -> String {
        let mut body = String::new();
        let mut resp = reqwest::get(&self.url).unwrap();
        let _ = resp.read_to_string(&mut body);
        body
    }

    fn sync(&mut self) {
        lazy_static! {
            static ref RATING_RE: Regex = Regex::new(r"\(([0-9.]+), ([0-9]+) ratings\)").unwrap();
        }

        let body = self.fetch();
        let doc = Html::parse_document(&body);

        let title_selector = Selector::parse("h1").unwrap();
        self.name = doc.select(&title_selector).next().unwrap().inner_html();

        let ratings_selector = Selector::parse(".en_grade_average").unwrap();
        let ratings = doc.select(&ratings_selector).next().unwrap().inner_html();
        let caps = RATING_RE.captures(ratings.as_str()).unwrap();

        self.rating = caps.get(1).unwrap().as_str().parse::<f32>().unwrap();
        self.review_count = caps.get(2).unwrap().as_str().parse::<i8>().unwrap();
    }
}

fn output(events: &Vec<Event>) {
    let mut table = Table::new();

    // Add header
    table.add_row(Row::new(vec![Cell::new("Event"), Cell::new("Rating")]));

    for event in events {
        let rating = event.rating.to_string();
        table.add_row(Row::new(vec![Cell::new(event.name.as_str()), Cell::new(rating.as_str())]));
    }

    table.printstd();
}

fn main() {
    let mut events: Vec<Event> = vec![];

    events.push(Event {
        name: String::from(""),
        url: String::from("https://conferences.oreilly.com/oscon/oscon-tx/public/schedule/detail/57875"),
        rating: 0.0,
        review_count: 0
    });

    for event in &mut events {
        event.sync();
    }

    output(&events);
}
