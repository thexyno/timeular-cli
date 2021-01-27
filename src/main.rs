use argparse::{ArgumentParser, Store, Collect, Print};
use reqwest::blocking::Client;
use reqwest::Error;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::env;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;


use const_format::concatcp;

const API_URL: &str = "https://api.timeular.com/api/v3/";

#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ActivitiesResponse {
    activities: Vec<Activity>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Activity {
    id: String,
    name: String,
    color: String,
    integration: Option<String>,
    spaceId: String,
    deviceSide: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TrackingResponse {
    currentTracking: Option<CurrentTracking>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CurrentTracking {
    id: usize,
    activityId: String,
    startedAt: NaiveDateTime,
    note: Option<Note>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    text: String,
    tags: Vec<Tag>,
    mentions: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    id: usize,
    key: String,
    label: String,
    scope: String,
    spaceId: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimeularApi {
    token: String,
}

impl TimeularApi {

    fn new (key: &String, secret: &String) -> Result<TimeularApi, Error> {
        let mut map = HashMap::new();
        map.insert("apiKey", key);
        map.insert("apiSecret", secret);
        let resp = Client::new().post(concatcp!(API_URL , "developer/sign-in"))
            .json(&map)
            .send()?;
        return Ok(TimeularApi{ token: resp.json::<TokenResponse>()?.token });
    }
    
    fn new_from_token (token: &String) -> TimeularApi {
        TimeularApi{ token: token.to_string() }
    }

    fn tracking(&self) -> Result<TrackingResponse, Error> {
        Client::new()
            .get(concatcp!(API_URL , "tracking"))
            .header("Authorization", String::from("Bearer ")+&self.token).send()?.json()
    }
    
    fn activities(&self) -> Result<ActivitiesResponse, Error> {
        Client::new()
            .get(concatcp!(API_URL , "activities"))
            .header("Authorization", String::from("Bearer ")+&self.token).send()?.json::<ActivitiesResponse>()
    }

    fn cur_tracking_str(&self) -> String {
        let cur_tracking = self.tracking().unwrap().currentTracking;
        match cur_tracking {
            Some(cur) => {
                let mut activity_name: String;
                let mut note = String::new();
                let mut activities = self.activities().unwrap().activities.into_iter();
                let activity_id = cur.activityId.clone();
                activity_name = activities.find(|x| x.id == activity_id).unwrap().name;
                match cur.note {
                    Some(n) => note = n.text,
                    _ => (),
                }
                let dur = Utc::now().naive_utc() - cur.startedAt;

                format!("[{hour}:{min}] {activity}: {note}\n",
                    min=dur.num_minutes() % 60,
//                    sec=dur.num_seconds() % 60,
                    hour=dur.num_hours() % 60,
                    activity=activity_name,
                    note=note)
            },
            None => return "Nothing is being tracked\n".to_string()
    
        }
    
    }

}



fn main() {
    let mut key = "".to_string();
    let mut secret = "".to_string();
    let mut argument: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("timeular cli stuff.");
        ap.refer(&mut key)
            .add_option(&["-k", "--api-key"], Store,
            "Api Key").required().envvar("TIMEULAR_KEY");
        ap.refer(&mut secret)
            .add_option(&["-s", "--api-secret"], Store,
            "Api Secret").required().envvar("TIMEULAR_SECRET");
        ap.refer(&mut argument)
            .add_argument("current", Collect,
            "Get the current running task");
        ap.add_option(&["-v", "--version"],
            Print(env!("CARGO_PKG_VERSION").to_string()), "Show version");
        ap.parse_args_or_exit();
    }
    let tapi: TimeularApi;
    let path = env::temp_dir().join("timeular.token");
    let strres = fs::read_to_string(&path);
    match strres {
        Ok(s) => tapi = TimeularApi::new_from_token(&s),
        Err(_) => {
            let authres = TimeularApi::new(&key, &secret);
            match authres {
                Ok(res) => {
                    tapi = res;
                    let mut file = fs::File::create(path).unwrap();
                    file.write_all(tapi.token.as_bytes()).unwrap();
                },
                Err(e) => panic!(e)
            }
        }
    }
    if argument.len() > 0 {
    //    match argument.get(0).unwrap() {
    //    }
    } else {
        io::stdout().write_all(tapi.cur_tracking_str().as_bytes()).unwrap();
    }


}
