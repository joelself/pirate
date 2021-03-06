/* Pirate - A command-line arrrrguments parser, written in Rust.
* Copyright (C) 2015 Zachary Dziura
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::collections::HashMap;
use std::collections::hash_map::Keys;

use errors::{Error, ErrorKind};
use vars::Vars;

pub type Matches = HashMap<String, String>;

pub fn matches(env_args: &[String], vars: &mut Vars) -> Result<Matches, Error> {
    let mut matches: Matches = HashMap::new();
    let mut args = env_args.iter();

    args.next(); // Remove the program name

    while let Some(mut current_arg) = args.next() {
        let mut arg_vec: Vec<String> = Vec::new();

        // Determine if current opt is in short, long, or arg form
        if &current_arg[..1] == "-" {
            if &current_arg[..2] == "--" { // Long form opt
                arg_vec.push(String::from(&current_arg[2..]));
            } else { // Short form opt
                // Assuming it's a group of short-form vars; e.g. tar -xzf
                for c in current_arg[1..].chars() {
                    let mut s = String::new();
                    s.push(c);
                    arg_vec.push(s);
                }
            }

            for arg in arg_vec.iter() {
                if vars.contains_opt(arg) {
                    let token = vars.get_opt(arg).unwrap();

                    if token.has_arg {
                        // NOTE: The corresponding arg MUST be immediately following
                        current_arg = match args.next() {
                            None =>  return Err(Error::new(ErrorKind::MissingArgument, arg.clone())),
                            Some(a) => a
                        };

                        matches.insert(token.name(), (*current_arg).clone());
                    } else {
                        matches.insert(token.name(), String::new());
                    }
                } else {
                    return Err(Error::new(ErrorKind::InvalidArgument, arg.clone()));
                }
            }
        } else { // Probably a required arg
            let arg = vars.get_arg().unwrap();
            matches.insert(arg.name(), (*current_arg).clone());
        }
    }

    match vars.arg_len() {
        0 => Ok( matches ),
        _ => Err(Error::new(ErrorKind::MissingArgument, vars.get_arg().unwrap().name())),
    }
}

pub trait Match {
    fn get(&self, arg: &str) -> Option<&String>;

    fn has_match(&self, arg: &str) -> bool;

    fn matches(&self) -> Keys<String, String>;
}

impl Match for Matches {
    fn get(&self, arg: &str) -> Option<&String> {
        self.get(arg)
    }

    fn has_match(&self, arg: &str) -> bool {
        let arg = String::from(arg);
        self.contains_key(&arg)
    }

    fn matches(&self) -> Keys<String, String> {
        self.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::{Match, matches};
    use super::super::vars::vars;
    
    #[test]
    fn test_matches_good() {
        let env_args = vec![String::from("test"), String::from("-a"), String::from("Test")];
        let opts = vec!["o/opt#An option", "a#An argument:"];
        
        let mut var = match vars("Test", &opts) {
            Ok(m) => m,
            Err(why) => panic!("An error occurred: {}", why)
        };
        
        let matches = matches(&env_args, &mut var).unwrap();
        
        let has_opt = match matches.get("opt") {
            Some(_) => true,
            None => false
        };
        let argument = matches.get("a").unwrap();
        assert_eq!(*argument, String::from("Test"));
        assert_eq!(has_opt, false);
    }

    #[test]
    #[should_panic]
    fn test_matches_bad() {
        let env_args = vec![String::from("test"), String::from("-a")];
        let opts = vec!["o/opt#An option", "a#An argument:"];
        
        let mut vars = vars("Test", &opts).unwrap();
        match matches(&env_args, &mut vars) {
            Ok(m) => m,
            Err(why) => panic!("An error occurred: {}", why)
        };
    }
}
