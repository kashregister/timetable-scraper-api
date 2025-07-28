use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use regex::Regex;
use soup::prelude::*;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlock {
    pub dan: isize,
    pub predmet: Predmet,
    pub profesor: String,
    pub tip: String,
    pub trajanje: isize,
    pub ucilnica: String,
    pub ura: isize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Predmet {
    pub abbr: String,
    pub location: String,
    pub name: String,
}

fn map_day(day: String) -> isize {
    let conv = day.as_str();
    match conv {
        "MON" => 0,
        "TUE" => 1,
        "WED" => 2,
        "THU" => 3,
        "FRI" => 4,
        _ => -1,
    }
}

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Hello\nCurrently supported timetables:\nFRI")
}

#[get("/timetable/fri/{id}")]
async fn timetable(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let url = format!(
        "https://urnik.fri.uni-lj.si/timetable/fri-2024_2025-letni/allocations?group={}",
        id
    );
    let body: String = ureq::get(url)
        .call()
        .unwrap()
        .body_mut()
        .read_to_string()
        .unwrap();
    let soup = Soup::new(&body);

    let results: Vec<_> = soup.attr("class", "grid-entry").find_all().collect();
    let rgx_time_duration = Regex::new(r"grid-row: (\d?\d) / span (\d?\d);").unwrap();
    let rgx_day = Regex::new(r"grid-area: day(...)").unwrap();
    let mut json = Vec::new();
    for grid_entry in &results {
        // Declare the object for storing temporary data
        let mut temp_block: TimeBlock = TimeBlock {
            ura: -1,
            profesor: "N/A".to_string(),
            dan: -1,
            tip: "N/A".to_string(),
            trajanje: -1,
            ucilnica: "N/A".to_string(),
            predmet: Predmet {
                name: "N/A".to_string(),
                abbr: "N/A".to_string(),
                location: "N/A".to_string(),
            },
        };
        // Find the style attribute for getting the row and span values
        let style = grid_entry.attr_name("style").find().expect("N/A").display();
        let time_duration = rgx_time_duration.captures(&style);
        if let Some(exists) = time_duration {
            temp_block.ura = 6 + exists[1].parse::<isize>().unwrap();
            temp_block.trajanje = exists[2].parse::<isize>().unwrap();
        } else {
            temp_block.ura = -1;
            temp_block.trajanje = -1;
        }
        // Find the day attribute
        let dan = grid_entry
            .parent()
            .unwrap()
            .attr_name("style")
            .find()
            .expect("N/A")
            .display()
            .to_string();

        if let Some(exists) = rgx_day.captures(&dan) {
            temp_block.dan = map_day(exists[1].to_string());
        } else {
            temp_block.dan = -1;
        }
        // Subject attributes
        let subject = grid_entry
            .tag("a")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();
        // Subject type
        //		tip = entry.find(class_='entry-type').text[1:].strip();
        let subject_type = grid_entry
            .attr("class", "entry-type")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();
        // Professor
        // profesor = entry.find(class_='link-teacher').text.title();
        let temp_profesor = grid_entry
            .attr("class", "link-teacher")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();
        // Classroom
        // ucilnica = entry.find(class_='link-classroom').text;

        let temp_classroom = grid_entry
            .attr("class", "link-classroom")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();
        // Init the object
        let temp_predmet: Predmet = Predmet {
            // Name is the same as the abbreviation for now
            name: subject.to_string(),
            abbr: subject.to_string(),
            location: "FRI".to_string(),
        };
        temp_block.profesor = temp_profesor.to_string();
        temp_block.predmet = temp_predmet;
        temp_block.tip = subject_type.to_string();
        temp_block.ucilnica = temp_classroom.to_string();
        // Push the time block to the list
        json.push(temp_block);
    }
    HttpResponse::Ok().json(json)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(root).service(timetable))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
