use std::fs;
use std::io::{self, Write};
use std::env;
use argh::FromArgs;


mod api;
use api::api::TimeularApi;

#[derive(FromArgs, PartialEq, Debug)]
/// A Timeular CLI written in Rust
struct CmdArgs {
    #[argh(option, short = 'k')]
    /// api Key. can also be set via 'TIMEULAR_API_KEY'
    api_key: Option<String>,

    #[argh(option, short = 's')]
    /// api Secret. can also be set via 'TIMEULAR_API_SECRET'
    api_secret: Option<String>,

    #[argh(subcommand)]
    nested: Option<SubCmds>,

}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCmds {
    TrackingCmd(TrackingCmd),
    NewCmd(NewCmd),
}

#[derive(FromArgs, PartialEq, Debug)]
/// show what is currently tracked
#[argh(subcommand, name = "tracking")]
struct TrackingCmd {
}

#[derive(FromArgs, PartialEq, Debug)]
/// start tracking a new activity
#[argh(subcommand, name = "track")]
struct NewCmd {
    #[argh(positional)]
    /// name of the activity
    activity_name: String,

    #[argh(positional)]
    /// notes
    notes: Option<String>,
}



fn main() {
    let mut args: CmdArgs = argh::from_env();
    match args.api_key {
        None => {
            args.api_key = Some(env::var("TIMEULAR_API_KEY").unwrap());
            args.api_secret = Some(env::var("TIMEULAR_API_SECRET").unwrap());

        }
        Some(_) => (),
    }

    let tapi: TimeularApi;
    let path = env::temp_dir().join("timeular.token");
    let strres = fs::read_to_string(&path);
    match strres {
        Ok(s) => tapi = TimeularApi::new_from_token(&s),
        Err(_) => {
            let authres = TimeularApi::new(&args.api_key.unwrap(), &args.api_secret.unwrap());
            match authres {
                Ok(res) => {
                    tapi = res;
                    let mut file = fs::File::create(path).unwrap();
                    file.write_all(tapi.token.as_bytes()).unwrap();
                },
                Err(e) => panic!(format!("{:?}",e))
            }
        }
    }
    match args.nested {
        Some(subarg) => {
            match subarg {
                SubCmds::TrackingCmd(_) => io::stdout().write_all(tapi.cur_tracking_str().as_bytes()).unwrap(),
                SubCmds::NewCmd(x) => {
                    let act = tapi.activities();
                    match act {
                        Ok(a) => {
                            let mut act_iter = a.activities.into_iter();
                            println!("{}", tapi.start_tracking(act_iter.find(|y| y.name == x.activity_name).unwrap().id).unwrap().message);

                        },
                        Err(e) => eprint!("{}",e),

                    }
                },
            }
        },
        None => io::stdout().write_all(tapi.cur_tracking_str().as_bytes()).unwrap(),
    }

}


