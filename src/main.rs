extern crate reqwest;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate url;

#[macro_use]
extern crate structopt;
use api::Toggl;
use chrono::prelude::*;
use structopt::StructOpt;
//TODO / WARN there is bug in that everything is one hour out

///A command line CLI to bulk upload time entries to toggl in continious time blocks.
/// 
/// Example Usage:
/// 
/// To get a list of all the projects and their ids under the hss workspace do
/// 
/// toggl --get-project-ids true -t <API_TOKEN>
/// 
/// To log 7.5 hours for each day monday to friday for the month of july do the following
/// 
/// toggl -s '01-07-2018' -e '31-07-2018' -p 139353704 -t <SOME API TOKEN>
/// 
/// All date ranges are inclusive
#[derive(StructOpt, Debug)]
#[structopt(name = "toggl")]
struct Opt{
    /// The start datetime block in the format dd-mm-YYYY .
    /// For example the 31st of august 31-08-2018
    #[structopt(short = "s", long = "start")]
    start:Option<String>,
    /// The same format as the start time but for specifying the end block
    #[structopt(short = "e", long = "stop")]
    stop:Option<String>,

    /// The project id of the toggl project the time should be recorded against
    #[structopt(short = "p", long = "project-id")]
    pid:Option<isize>,
    /// The toggl API token which is found in the 'user profile' section on toggl
    #[structopt(short = "t", long = "token")]
    token:String,
    ///Prints the project id's to be used with the -p option 
    #[structopt( long = "get-project-ids")]
    project_ids:Option<bool>,
    ///Get the time entries in the range specified by the -s -e switches
    #[structopt(long ="get-time-entries")]
    time_entries:Option<bool>,
    /// The workspace id to be used with --get-project-ids
    #[structopt( long = "workspace-id",default_value="741311")]
    workspace_id:isize,
}
fn main() {
    //sick 138334910
    //support patched 138334610
    //1.6 139353704
    // 1.7 139353826
    //139353626 eforms embedded
    let opt = Opt::from_args();
    let toggl = Toggl::new(opt.token);
    
    if opt.project_ids.is_some() && opt.project_ids.unwrap(){
        let projects = toggl.get_projects(opt.workspace_id).unwrap();
        for proj in projects.iter(){
            println!("Project {} , {}",proj.id,proj.name );
        }
        std::process::exit(0);
    }
    
    let start = match opt.start{
        Some(x) =>  {
            let d = NaiveDate::parse_from_str(&x, "%d-%m-%Y").expect("Could not parse date format dd-mm-yyyy");
            Utc.ymd(d.year(), d.month(), d.day()).and_hms(0, 0, 0)
        },
        None => panic!("Start option cannot be empty!")
    };
    let end = match opt.stop{
        Some(x) => {
            let d = NaiveDate::parse_from_str(&x, "%d-%m-%Y").expect("Could not parse date format dd-mm-yyyy");
            Utc.ymd(d.year(), d.month(), d.day()).and_hms(0, 0, 0)
        },
        None => panic!("Start option cannot be empty!")
    };

    if opt.time_entries.is_some() && opt.time_entries.unwrap() {
        let entries =  match toggl.get_time_entries_in_range(start, end){
            Ok(x) => x,
            Err(e) => panic!("Error getting time entires {}",e)
        };


        //TODO print a text table list them in a table and allow basic navigation/ actions delete by id
        // so pressign d makes prompt [delete] >> 1173 for delete by id and range sytax 1173~1194 and 1173,1123,33432 for multiple
        for entry in entries{
            println!("From: {} To: {}",entry.start,entry.stop);
        }
        std::process::exit(0);
    }
    let pid = opt.pid.expect("Need a valid project id");
    if !(start <= end){//TODO maybe change to an assert
        panic!("the start date must be before the end date");
    }
    
    /* let start = Utc.datetime_from_str(start.as_str(), "%d-%m-%Y %H:%M:%S").unwrap();
    let end = Utc.datetime_from_str(end.as_str(), "%d-%m-%Y %H:%M:%S").unwrap(); */
    /* let start = Utc.datetime_from_str(opt.start.as_str(), "%d-%m-%Y").unwrap();
    let end = Utc.datetime_from_str(opt.stop.as_str(), "%d-%m-%Y").unwrap(); */
    
    let entries = time_entries_range(start, end, pid);
    for time_entry in entries{
        let check = toggl.create_time_entry(time_entry);
            match check {
                Ok(c) => {
                    if !c {
                        panic!("Error got false from upload {}", c);
                    }
                }
                Err(_) => panic!("Error "),
            }
        //sleep for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
    }


    /* let vip17 = 139353826;
    
    let toggl = Toggl::new(token.to_string());
    let start = Utc.ymd(2018, 09, 7).and_hms(0, 0, 0);
    let end = Utc.ymd(2018, 09, 14).and_hms(0, 0, 0);
    time_entries_range(start, end, toggl, vip17); */
    /* let dur = std::time::Duration::from_secs(27000);
    let t = api::TimeEntry::new(date, dur, vip17);
    println!("{:?}", t);
    let check = toggl.create_time_entry(t);

    match check {
        Ok(c) => println!("Response {}", c),
        Err(e) => println!("Error {}", e),
    } */

    
}

/// Create a list of time entries between the given start and end range for a given project id 
/// This does not create them in toggle
pub fn time_entries_range(start: DateTime<Utc>, end: DateTime<Utc>, pid: isize) -> Vec<api::TimeEntry>{
    //while start is before end create entry for each day
    let mut current = start;
    let days = chrono::Duration::days(1);
    let dur = std::time::Duration::from_secs(27000);
    let mut entries:Vec<api::TimeEntry> = vec!();
    //isbfore and after methods arnt needed with chrono time as the arithmatic ops are overloaded
    while current <= end {
        let time_entry = api::TimeEntry::new(current, dur, pid);
        //only do this for monday to friday
        if current.weekday() != chrono::Weekday::Sat && current.weekday() != chrono::Weekday::Sun {
            entries.push(time_entry);
        }
        //add 1 day to current
        current = current + days;
    }
    return entries;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    /// Test Helper util to get the day of the week a time entry is on
    fn day_of_week(entry: &api::TimeEntry) -> Option<Weekday>{
        let start = DateTime::parse_from_rfc3339(entry.start.as_str()).unwrap();
        let end = DateTime::parse_from_rfc3339(entry.stop.as_str()).unwrap();
        if start < end && start.date() == end.date() {
            return Some(start.weekday());
        }
        return None;
    }
    #[test]
    fn test_time_enties_range_inclusive(){
        let start = Utc.datetime_from_str("31-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let end = Utc.datetime_from_str("04-01-2019 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let entries = time_entries_range(start, end, 2);
        assert_eq!(entries[0].duration,27000);
        assert_eq!(entries.len(),5);
    }
    #[test]
    fn test_time_enties_range_ignores_weekends(){
        let start = Utc.datetime_from_str("27-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let end = Utc.datetime_from_str("31-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let entries = time_entries_range(start, end, 2);
        assert_eq!(entries.len(),3);
        assert_eq!(day_of_week(&entries[0]).unwrap(),Weekday::Thu);
        assert_eq!(day_of_week(&entries[1]).unwrap(),Weekday::Fri);
        assert_eq!(day_of_week(&entries[2]).unwrap(),Weekday::Mon);
    }

    #[test]
    fn test_time_entries_range_single_day(){
        let start = Utc.datetime_from_str("27-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let end = Utc.datetime_from_str("27-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let entries = time_entries_range(start, end, 2);
        assert_eq!(entries.len(),1);
        assert_eq!(day_of_week(&entries[0]).unwrap(),Weekday::Thu);
    }
}

mod api {
    use reqwest;
    use reqwest::header::{Basic, Headers};
    use serde_json;
    //use chrono::offset::LocalResult;
    use chrono::prelude::*;
    
use url::form_urlencoded::{byte_serialize, parse};

pub struct ResponseWrapper{
}
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Project {
       pub  id: isize,
        wid: isize,
        //cid: isize,
        pub name: String,
        billable: bool,
        is_private: bool,
        active: bool,
        at: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct TimeEntry {
        pub description: String,
        pub pid: isize,
        pub start: String, //iso 8601 date
        pub stop: String,
        pub duration: u64,
        pub created_with: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct TimeEntryDetails{
        pub id:isize,
        pub guid:String,
        pub wid:isize,
        pub billable:bool,
        pub start:String,
        pub stop:String,
        pub duration:u64,
        pub duronly:bool,
        pub at:String,
        pub uid:isize
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Payload {
        time_entry: TimeEntry,
    }

    impl TimeEntry {
        ///Construct a new time entry for the day defined by date that lasts up to dur Duration
        /// against a project defined by the id pid
        pub fn new(date: DateTime<Utc>, dur: ::std::time::Duration, pid: isize) -> TimeEntry {
            let chr_dur = ::chrono::Duration::from_std(dur).unwrap();
            let start_time = Utc
                .ymd(date.year(), date.month(), date.day())
                .and_hms(9, 0, 0);
            let end_time = start_time + chr_dur;
            let description = "".to_string();

            /* let start = format!("{:?}", start_time);
            let stop = format!("{:?}", end_time); */
            let start = start_time.to_rfc3339();
            let stop = end_time.to_rfc3339();
            let duration = dur.as_secs();
            let created_with = "cli".to_string();
            let time_entry = TimeEntry {
                description,
                pid,
                start,
                stop,
                duration,
                created_with,
            };
            //TODO error handling
            return time_entry;
        }
    }

    pub struct Toggl {
        api_token: String,
    }
    
    impl Toggl {
        pub fn new(api_token: String) -> Toggl {
            return Toggl { api_token };
        }
        /// Gets all the time entries in a specified range - This will get at most 1000 entries which may destroy mem
        pub fn get_time_entries_in_range(&self,start:DateTime<Utc>,end:DateTime<Utc>) -> Result<Vec<TimeEntryDetails>,reqwest::Error>{
            let c = reqwest::Client::new();
            let start_d:String = byte_serialize(start.to_rfc3339().as_bytes()).collect();
            let end_d:String = byte_serialize(end.to_rfc3339().as_bytes()).collect();
            let url = format!("https://www.toggl.com/api/v8/time_entries?start_date={}&end_date={}",start_d,end_d);
            let mut r = c
                .get(url.as_str())
                .basic_auth(self.api_token.clone(), Some("api_token".to_string()))
                .send()?;
            
            let entries:Vec<TimeEntryDetails> = r.json()?;
            return Ok(entries);
        }

        pub fn create_time_entry(&self, time_entry: TimeEntry) -> Result<bool, reqwest::Error> {
            let url = "https://www.toggl.com/api/v8/time_entries";
            let c = reqwest::Client::new();
            let wrapped = Payload { time_entry };
            let body = serde_json::to_string(&wrapped).unwrap();
            println!("body {}", body);
            let r = c
                .post(url)
                .body(body)
                .basic_auth(self.api_token.clone(), Some("api_token".to_string()))
                .send()?;
            let check = r.status().is_success();
            println!("Status {}", r.status());
            return Ok(check);
        }
        pub fn get_projects(&self, wid: isize) -> Result<Vec<Project>, reqwest::Error> {
            //TODO single client instance?
            let c = reqwest::Client::new();
            let url = format!("https://www.toggl.com/api/v8/workspaces/{}/projects", wid);

            let mut r = c
                .get(url.as_str())
                .basic_auth(self.api_token.clone(), Some("api_token".to_string()))
                .send()?;
            let projects: Vec<Project> = r.json()?;
            return Ok(projects);
        }
    }

}
