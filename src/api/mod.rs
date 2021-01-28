pub mod api {
    use reqwest::blocking::Client;
    use reqwest::Error;
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};
    use chrono::prelude::*;
    
    
    use const_format::concatcp;
    const API_URL: &str = "https://api.timeular.com/api/v3/";
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct TokenResponse  {
        token: Option<String>,
        message: Option<String>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ActivitiesResponse {
        pub activities: Vec<Activity>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[allow(non_snake_case)]
    pub struct Activity {
        pub id: String,
        pub name: String,
        pub color: String,
        pub integration: Option<String>,
        pub spaceId: String,
        pub deviceSide: Option<usize>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    #[allow(non_snake_case)]
    pub struct TrackingResponse {
        currentTracking: Option<CurrentTracking>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Message {
        pub message: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[allow(non_snake_case)]
    pub struct CurrentTracking {
        id: usize,
        activityId: String,
        startedAt: NaiveDateTime,
        note: Option<Note>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Note {
        text: Option<String>,
        tags: Vec<Tag>,
        mentions: Vec<Tag>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    #[allow(non_snake_case)]
    pub struct Tag {
        id: usize,
        key: String,
        label: String,
        scope: String,
        spaceId: String,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct TimeularApi {
        pub token: String,
    }
    
    impl TimeularApi {
    
        pub fn new (key: &String, secret: &String) -> Result<TimeularApi, Error> {
            let mut map = HashMap::new();
            map.insert("apiKey", key);
            map.insert("apiSecret", secret);
            let resp = Client::new().post(concatcp!(API_URL , "developer/sign-in"))
                .json(&map)
                .send()?.json::<TokenResponse>()?;

            match resp.token {
                Some(token) => return Ok(TimeularApi{ token: token }),
                None => panic!(resp.message.unwrap()),
            }
        }
        
        pub fn new_from_token (token: &String) -> TimeularApi {
            TimeularApi{ token: token.to_string() }
        }
    
        pub fn tracking(&self) -> Result<TrackingResponse, Error> {
            Client::new()
                .get(concatcp!(API_URL , "tracking"))
                .header("Authorization", String::from("Bearer ")+&self.token).send()?.json()
        }

        pub fn start_tracking(&self, activity_id: String) -> Result<Message, Error> {
            let mut map = HashMap::new();
            map.insert("startedAt", format!("{}", Utc::now().format("%FT%T%.3f")));
            Client::new()
                .post(&format!("{}tracking/{}/start",API_URL, activity_id))
                .json(&map)
                .header("Authorization", String::from("Bearer ")+&self.token).send()?.json()
        }
        
        pub fn activities(&self) -> Result<ActivitiesResponse, Error> {
            Client::new()
                .get(concatcp!(API_URL , "activities"))
                .header("Authorization", String::from("Bearer ")+&self.token).send()?.json::<ActivitiesResponse>()
        }
    
        pub fn cur_tracking_str(&self) -> String {
            let cur_tracking = self.tracking().unwrap().currentTracking;
            match cur_tracking {
                Some(cur) => {
                    let mut note = String::new();
                    let mut activities = self.activities().unwrap().activities.into_iter();
                    let activity_id = cur.activityId.clone();
                    let activity_name = activities.find(|x| x.id == activity_id).unwrap().name;
                    match cur.note {
                        Some(n) => note = n.text.unwrap_or("".to_string()),
                        _ => (),
                    }
                    let dur = Utc::now().naive_utc() - cur.startedAt;
    
                    format!("[{hour:02}:{min:02}] {activity}: {note}\n",
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

}
