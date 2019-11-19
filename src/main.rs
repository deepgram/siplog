mod logging;

use chrono::{Local, NaiveDateTime};
use log::{error, Level};
use serde::Deserialize;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io;
use structopt::StructOpt;

#[derive(Deserialize, Clone, Debug)]
pub struct SipAppJson {
    level: usize,
    time: u64,
    msg: String,
    pid: usize,
    hostname: String,
    #[serde(rename = "type")]
    type_: Option<String>,
    stack: Option<String>,
    errno: Option<String>,
    syscall: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    secret: Option<String>,
    v: usize,
}

impl From<usize> for CustomLevel {
    fn from(item: usize) -> Self {
        if item == 10 {
            return CustomLevel::Trace;
        }
        if item == 20 {
            return CustomLevel::Debug;
        }
        if item == 30 {
            return CustomLevel::Info;
        }
        if item == 40 {
            return CustomLevel::Warn;
        }
        if item == 50 {
            return CustomLevel::Error;
        }
        if item == 60 {
            return CustomLevel::Error;
        }

        return CustomLevel::Trace;
    }
}

impl SipAppJson {
    pub fn custom_print(&self) {
        let level = Level::from(CustomLevel::from(self.level));
        let seconds = (self.time / 1000).try_into().unwrap();
        let nanoseconds = (1000000 * (self.time - 1000 * (self.time / 1000)))
            .try_into()
            .unwrap();
        let timestamp = NaiveDateTime::from_timestamp(seconds, nanoseconds)
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();
        let message = self.msg.clone();
        let (level, color) = match level {
            Level::Error => ("ERROR", 1),
            Level::Warn => ("WARN ", 3),
            Level::Info => ("INFO ", 7),
            Level::Debug => ("DEBUG", 4),
            Level::Trace => ("TRACE", 5),
        };
        // color, level, and timestamp
        let mut printout = format!("\x1B[1;3{}m[{} {}]", color, level, timestamp);

        // "extras"
        let extras_color = 2;
        printout = format!("{} \x1B[1;3{}m[", printout, extras_color);
        printout = format!("{}v:{}", printout, self.v);
        printout = format!("{} pid:{}", printout, self.pid);
        printout = format!("{} hostname:{}", printout, self.hostname.trim().to_string());
        if let Some(type_) = &self.type_ {
            printout = format!("{} type:{}", printout, type_.trim().to_string());
        }
        if let Some(stack) = &self.stack {
            printout = format!("{} stack:{}", printout, stack.trim().to_string());
        }
        if let Some(errno) = &self.errno {
            printout = format!("{} errno:{}", printout, errno.trim().to_string());
        }
        if let Some(syscall) = &self.syscall {
            printout = format!("{} syscall:{}", printout, syscall.trim().to_string());
        }
        if let Some(address) = &self.address {
            printout = format!("{} address:{}", printout, address.trim().to_string());
        }
        if let Some(port) = &self.port {
            printout = format!("{} port:{}", printout, port);
        }
        if let Some(secret) = &self.secret {
            printout = format!("{} secret:{}", printout, secret.trim().to_string());
        }
        printout = format!("{}]", printout);

        // message
        printout = format!("{}\x1B[0m {}", printout, message.trim().to_string());

        // final printout
        println!("{}", printout);
    }
}

pub enum CustomLevel {
    // these are identical to log::Level
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    // these are new
    Err,
    Warning,
    Console,
    Notice,
}

impl From<CustomLevel> for Level {
    fn from(item: CustomLevel) -> Self {
        match item {
            // these are identical to log::Level
            CustomLevel::Error => Level::Error,
            CustomLevel::Warn => Level::Warn,
            CustomLevel::Info => Level::Info,
            CustomLevel::Debug => Level::Debug,
            CustomLevel::Trace => Level::Trace,
            // these are new
            CustomLevel::Err => Level::Error,
            CustomLevel::Warning => Level::Warn,
            CustomLevel::Console => Level::Debug,
            CustomLevel::Notice => Level::Trace,
        }
    }
}

impl TryFrom<String> for CustomLevel {
    type Error = &'static str;

    fn try_from(item: String) -> Result<Self, &'static str> {
        // these are identical to log::Level
        if item == "ERROR" {
            return Ok(CustomLevel::Error);
        }
        if item == "WARN" {
            return Ok(CustomLevel::Warn);
        }
        if item == "INFO" {
            return Ok(CustomLevel::Info);
        }
        if item == "DEBUG" {
            return Ok(CustomLevel::Debug);
        }
        if item == "TRACE" {
            return Ok(CustomLevel::Trace);
        }
        // these are new
        if item == "ERR" {
            return Ok(CustomLevel::Err);
        }
        if item == "WARNING" {
            return Ok(CustomLevel::Warn);
        }
        if item == "CONSOLE" {
            return Ok(CustomLevel::Console);
        }
        if item == "NOTICE" {
            return Ok(CustomLevel::Notice);
        }

        Err("no such leven indicator recognized")
    }
}

pub fn custom_print(level: Level, timestamp: String, line: Option<String>, message: String) {
    let (level, color) = match level {
        Level::Error => ("ERROR", 1),
        Level::Warn => ("WARN ", 3),
        Level::Info => ("INFO ", 7),
        Level::Debug => ("DEBUG", 4),
        Level::Trace => ("TRACE", 5),
    };
    match line {
        Some(line) => {
            println!(
                "\x1B[1;3{}m[{} {} {}]\x1B[0m {}",
                color, level, timestamp, line, message
            );
        }
        None => {
            println!(
                "\x1B[1;3{}m[{} {}]\x1B[0m {}",
                color, level, timestamp, message
            );
        }
    }
}

#[derive(StructOpt, Clone)]
#[structopt(name = "siplog")]
struct SipLog {
    #[structopt(short = "v", parse(from_occurrences))]
    /// Increase the verbosity.
    verbosity: usize,
}

fn main() {
    // Parse command-line arguments.
    let args = SipLog::from_args();

    // Configure logging.
    logging::from_verbosity(
        args.verbosity,
        None,
        //        Some(vec!["mio", "tokio_reactor", "actix_net", "actix_web"]), // TODO: this is probably unnecessary
    );

    loop {
        // read line by line from stdin
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_num_bytes) => {
                let json_candidate: Option<SipAppJson> = serde_json::from_str(&input).ok();
                match json_candidate {
                    Some(json_line) => {
                        json_line.custom_print();
                        continue;
                    }
                    None => {}
                }

                // trim off excess spaces and newlines
                input = input.trim().to_string();

                // split the string
                let mut split: Vec<&str> = input.split(" ").collect();

                // search for source line
                let mut found_line: Option<String> = None;
                for index in 0..split.len() {
                    // assume source lines are of the format "/path/to/file:line_number" (potentially surrounded by brackets [])
                    let sub_split: Vec<&str> = split[index].split(":").collect();
                    if sub_split.len() != 2 {
                        continue;
                    }
                    // TODO: I would like to check that the first item in the sub split formatted as a valid path...
                    // check that the second item in the sub split is a number (potentially a line number)
                    let line_number = sub_split[1].to_string().replace("[", "").replace("]", "");
                    let line_number = line_number.parse::<i32>();
                    match line_number {
                        Ok(_line_number) => {
                            found_line = Some(split[index].to_string());
                            split.remove(index);
                            break;
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }

                // search for an indicator of a level
                let mut found_level: Option<Level> = None;
                for index in 0..split.len() {
                    // empirically, these may be surrounded by brackets [] or colons :)
                    let level_candidate = split[index]
                        .to_string()
                        .replace("[", "")
                        .replace("]", "")
                        .replace(":", "");
                    let level_candidate = CustomLevel::try_from(level_candidate);
                    match level_candidate {
                        Ok(level_candidate) => {
                            found_level = Some(Level::from(level_candidate));
                            split.remove(index);
                            break;
                        }
                        Err(_) => {}
                    }
                }

                // search for a timestamp
                let mut found_timestamp: Option<NaiveDateTime> = None;
                if split.len() >= 2 {
                    // check for a timestamp anywhere in the line (though it will usually be in the first two splits)
                    for index in 0..split.len() - 1 {
                        // combine two splits, timestamps should be in this format
                        // TODO: these replaces are largely unnecessary
                        let day = split[index]
                            .to_string()
                            .replace(|c: char| !c.is_ascii(), "")
                            .replace("[", "")
                            .replace("]", "");
                        let hour = split[index + 1]
                            .to_string()
                            .replace(|c: char| !c.is_ascii(), "")
                            .replace("[", "")
                            .replace("]", "");
                        let timestamp_candidate = day + " " + &hour;
                        // check if these two splits make a valid timestamp (minus the timezone)
                        let timestamp_candidate = NaiveDateTime::parse_from_str(
                            &timestamp_candidate,
                            "%Y-%m-%d %H:%M:%S%.3f", // TODO: why does this not force the precision to 3?
                        );
                        match timestamp_candidate {
                            // if we found a timestamp, set found_timestamp to it and remove the timestamp from the split
                            Ok(timestamp_candidate) => {
                                found_timestamp = Some(timestamp_candidate);
                                split.remove(index + 1);
                                split.remove(index);
                                break;
                            }
                            Err(_) => {}
                        }
                    }
                }

                // we might have a level now, if not, we'll use some default
                let level = match found_level {
                    Some(found_level) => found_level,
                    None => Level::Info,
                };

                // we might have a timestamp now, if not, we can make one
                let timestamp = match found_timestamp {
                    Some(found_timestamp) => {
                        found_timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
                    }
                    None => Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                };

                // print the final message
                let mut message = String::new();
                for str in split {
                    message.push_str(str);
                    message.push_str(" ");
                }
                message = message.trim().to_string(); // strip the last space we pushed
                custom_print(level, timestamp, found_line, message);
            }
            Err(error) => {
                error!("error readling line from stdin: {}", error);
            }
        }
    }
}
