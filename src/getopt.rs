/*
 *   Chappell's lightweight getopt() for rust. Sharp edges and no warranty. 
 *
 * - Define the fields you want in the struct
 * - Define the default() implementation to return the default values
 * - Write a nice usage message
 * - modify the parse logic to suit
 *
 * - from main() call getopt::getopt() to obtain a Getopt structure from argv, OR,
 * - or use lazy_static! { static ref GETOPT: getopt::Getopt = getopt::getopt(); }
 *   for a global static. dbg!(& *GETOPT); 
 */

use std::env;
use std::process;

pub const DEFAULT_MAX_SESSIONS: usize = 2;

// Define your command line arguments here: name and type
#[derive(Debug)]
pub struct Getopt {
    pub verbose: bool,
    pub commands: Vec<String>,
    pub max_sessions: usize,
    pub private_key_file: String,
    pub username: String,
    pub args: Vec<String>,  // there are positional arguments
}

// Same here, setting the default values
impl Default for Getopt {
    fn default() -> Getopt {
        Getopt {
            verbose: false,
            commands: vec![],
            max_sessions: DEFAULT_MAX_SESSIONS,
            #[allow(deprecated)]
            private_key_file: format!("{}/.ssh/id_ed25519", std::env::home_dir().unwrap().display()),
            username: whoami::username(),
            args: vec![],
        }
    }
}

pub fn getopt() -> Getopt {

    let mut getopt = Getopt::default();

    let mut args = env::args();

    args.next(); // blow off the first argument (it's the process name)

    /*
     * Iterate through the arguments. Check for each '-X' case of interest
     * using next()/expect() to take the next argument as required.
     * continue or break to carry on or stop respectively
     *
     * Remaining positionals end up in Getopt::args
     */
    while let Some(arg) = args.next() {

        getopt.args.push(match arg.as_str() {

            /* integer parameter example, using expect() and parse() */
            "-n" => {
                getopt.max_sessions = args
                    .next()
                    .expect("expected maximum number of sessions")
                    .parse()
                    .expect("number of sessions must be valid integer");
                    
                continue;
            },

            /* boolean flag example */
            "-v" => {
                getopt.verbose = true;
                // getopt.verbose = ! getopt.verbose; // toggle
                continue;
            },

            /* string value example */
            "-c" => {
                getopt.commands.push(
                    args
                    .next()
                    .expect("expected command to execute on remote host")
                );
                continue;
            },

            "-I" => {
                getopt.private_key_file = args
                    .next()
                    .expect("expected filename of private key");
                continue;
            },

            "-u" => {
                getopt.username = args
                    .next()
                    .expect("expected username");
                continue;
            },

            // usage text
            "-h" => { crate::usage(); break; },
            "-?" => { crate::usage(); break; }

            // If nothing matches, collect it up as positional
            _ => arg,
        })
    }

    // You can add an optional default positional here
    if getopt.args.len()==0 {
        eprintln!("Need to specify at least one remote hostname!");
        crate::usage();
    }
    getopt
}
