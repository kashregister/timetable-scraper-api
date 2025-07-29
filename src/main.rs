use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use regex::Regex;
use soup::prelude::*;

use actix_governor::*;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlock {
    pub day: usize,
    pub time: usize,
    pub duration: usize,
    pub professor: String,
    pub classroom: String,
    pub subject: Subject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub name: String,
    pub abbreviation: String,
    pub location: String,
    pub r#type: String,
}

fn map_day(day: String) -> usize {
    let conv = day.as_str();
    match conv {
        "MON" => 0,
        "TUE" => 1,
        "WED" => 2,
        "THU" => 3,
        "FRI" => 4,
        _ => 0,
    }
}
impl Default for TimeBlock {
    fn default() -> Self {
        TimeBlock {
            day: 0,
            time: 0,
            professor: "N/A".to_string(),
            classroom: "N/A".to_string(),
            duration: 0,
            subject: Subject::default(),
        }
    }
}
impl Default for Subject {
    fn default() -> Self {
        Subject {
            r#type: "N/A".to_string(),
            name: "N/A".to_string(),
            abbreviation: "N/A".to_string(),
            location: "N/A".to_string(),
        }
    }
}

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body(
        "Hello!\n\n\
        Endpoints:\n\
        / - current page\n\
        /timetable/{uni}/{group}\n\
        \n\
        Currently supported unis:\n\
        - fri",
    )
}

#[get("/timetable/fri/{group}")]
async fn timetable(path: web::Path<String>) -> impl Responder {
    let group = path.into_inner();
    let url = format!(
        "https://urnik.fri.uni-lj.si/timetable/fri-2024_2025-letni/allocations?group={}",
        group
    );
    let body: String = ureq::get(url)
        .call()
        .unwrap()
        .body_mut()
        .read_to_string()
        .unwrap_or("Error fetching".to_string());
    if body == "Error fetching" {
        HttpResponse::Ok().body("Error fetching");
    }
    let soup = Soup::new(&body);

    let results: Vec<_> = soup.attr("class", "grid-entry").find_all().collect();
    let rgx_time_duration = Regex::new(r"grid-row: (\d?\d) / span (\d?\d);").unwrap();
    let rgx_day = Regex::new(r"grid-area: day(...)").unwrap();
    let mut json = Vec::new();
    for grid_entry in &results {
        // Declare the object for storing temporary data
        let mut temp_block = TimeBlock::default();
        // Find the style attribute for getting the row and span values
        let style = grid_entry.attr_name("style").find().expect("N/A").display();
        let time_duration = rgx_time_duration.captures(&style);
        if let Some(exists) = time_duration {
            temp_block.time = 6 + exists[1].parse::<usize>().unwrap();
            temp_block.duration = exists[2].parse::<usize>().unwrap();
        } else {
            temp_block.time = 0;
            temp_block.duration = 0;
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
            temp_block.day = map_day(exists[1].to_string());
        } else {
            temp_block.day = 0;
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
        let mut subject_type = grid_entry
            .attr("class", "entry-type")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();

        subject_type.retain(|c| c != '|');
        _ = subject_type.trim();

        // Professor
        let temp_professor = grid_entry
            .attr("class", "link-teacher")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();
        // Classroom
        let temp_classroom = grid_entry
            .attr("class", "link-classroom")
            .find()
            .expect("N/A")
            .text()
            .trim()
            .to_string();
        // Init the object
        let temp_predmet: Subject = Subject {
            // Name is the same as the abbreviation for now
            name: subject.to_string(),
            abbreviation: subject.to_string(),
            location: "FRI".to_string(),
            r#type: subject_type.to_string(),
        };

        temp_block.professor = temp_professor.to_string();
        temp_block.subject = temp_predmet;
        temp_block.classroom = temp_classroom.to_string();
        // Push the time block to the list
        json.push(temp_block);
    }
    HttpResponse::Ok().json(json)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(60)
        .burst_size(3)
        .finish()
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .wrap(Governor::new(&governor_conf))
            .service(root)
            .service(timetable)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
