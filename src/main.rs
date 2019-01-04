extern crate reqwest;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate serde;
extern crate serde_json;

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
/// To log 7.5 hours for each day monday to friday between the second of july and the 1st of august for a given pid do
/// the following 
/// 
/// toggl -s '02-07-2018 00:00:00' -e '01-08-2018 00:00:00' -p 139353704 -t <SOME API TOKEN>
#[derive(StructOpt, Debug)]
#[structopt(name = "toggl")]
struct Opt{
    /// The start datetime block in the format dd-mm-YYYY 00:00:00.
    /// For example the 31st of august 31-08-2018 00:00:00
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
        Some(x) => x,
        None => panic!("Start option cannot be empty!")
    };
    let end = match opt.stop{
        Some(x) => x,
        None => panic!("Start option cannot be empty!")
    };
    let pid = match opt.pid{
        Some(x) => x,
        None => panic!("Start option cannot be empty!")
    };

    let start = Utc.datetime_from_str(start.as_str(), "%d-%m-%Y %H:%M:%S").unwrap();
    let end = Utc.datetime_from_str(end.as_str(), "%d-%m-%Y %H:%M:%S").unwrap();
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

pub fn time_entries_range(start: DateTime<Utc>, end: DateTime<Utc>, pid: isize) -> Vec<api::TimeEntry>{
    //while start is before end create entry for each day
    let mut current = start;
    let days = chrono::Duration::days(1);
    let dur = std::time::Duration::from_secs(27000);
    let mut entries:Vec<api::TimeEntry> = vec!();
    //isbfore and after methods arnt needed with chrono time as the arithmatic ops are overloaded
    while current < end {
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

    /// Test Helper util to get the day of the week a time entry is on
    fn day_of_week(entry:api::TimeEntry){

    }
    #[test]
    fn test_time_enties_range_inclusive(){
        let start = Utc.datetime_from_str("31-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let end = Utc.datetime_from_str("04-01-2019 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let entries = time_entries_range(start, end, 2);
        //assert_eq!(entries[0].duration,27000);
        assert_eq!(entries.len(),5);
        //check duration is 7:30 hours
    }
    #[test]
    fn test_time_enties_range_ignores_weekends(){
        let start = Utc.datetime_from_str("27-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let end = Utc.datetime_from_str("31-12-2018 00:00:00", "%d-%m-%Y %H:%M:%S").unwrap();
        let entries = time_entries_range(start, end, 2);
        assert_eq!(entries.len(),3);
    }
}

mod api {
    use reqwest;
    use reqwest::header::{Basic, Headers};
    use serde_json;
    //use chrono::offset::LocalResult;
    use chrono::prelude::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Project {
       pub  id: isize,
        wid: isize,
        cid: isize,
        pub name: String,
        billable: bool,
        is_private: bool,
        active: bool,
        at: String,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct TimeEntry {
        description: String,
        pid: isize,
        start: String, //iso 8601 date
        stop: String,
        duration: u64,
        created_with: String,
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
