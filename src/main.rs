extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate clap;

use clap::{Arg, ArgGroup, App, AppSettings};
use std::process;
use std::collections::HashMap;

pub const VERSION: &'static str = "0.6.0";

#[derive(Debug, Clone)]
struct Tab {
    name: String,
    number: usize,
    panes: usize,
}

impl Tab {
    fn new(name: &str, number: usize, panes: usize) -> Tab {
        Tab { name: name.to_string(), number: number, panes: panes }
    }
}

#[derive(Debug)]
struct Window {
    pub tabs: Vec<Tab>,
    pub attached: bool,
}

impl Window {
    fn new(tabs: Vec<Tab>, attached: bool) -> Window {
        Window { tabs: tabs,
                 attached: attached,
        }
    }

    fn push(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    fn is_empty(&self) -> bool {
        return self.tabs.len() == 0;
    }
}

type WindowList = HashMap<usize, Window>;

trait WindowSearch {
    fn select_tabs(&self, searchterm: &str) -> Self;
    fn populate(&mut self);
    fn dump(&self);
    fn get_cmd(&self);
    fn attach_cmd(&self);
}

fn build_windowlist() -> WindowList {
    let out = match process::Command::new("tmux")
                                      .arg("list-sessions")
                                      .output() {
        Ok(output) => output,
        Err(e) => panic!("failed to spawn: {}", e),
    };
    lazy_static! {
        static ref SESSION_RE: regex::Regex = regex::Regex::new(r"^(\d+): \d+ windows \(.*\) \[\d+x\d+\]( \(attached\))?").unwrap();
    }
    let mut windows: WindowList = HashMap::new();

    for line in String::from_utf8_lossy(&out.stdout).split('\n') {
        if line == "" { break }

        let cap = SESSION_RE.captures(&line).unwrap();
        let win: usize = cap[1].parse().unwrap();
        let attached: bool = cap.get(2).is_some();
        windows.insert(win, Window::new(vec![], attached));
    }

    windows.populate();

    return windows;
}

impl WindowSearch for WindowList {
    fn dump(&self) {
        for (idx, window) in self.iter() {
            print!("Session: {}", idx);
            if window.attached {
                print!(" (attached)");
            }
            print!("\n");
            for tab in window.tabs.iter() {
                println!("  {}: {}", tab.number, tab.name);
            }
        }
    }

    fn get_cmd(&self) {
        if self.len() != 1 {
            panic!("Can only get with a single result");
        }

        for (idx, window) in self.iter() {
            if window.tabs.len() != 1 {
                panic!("Can only get with a single result");
            }

            for tab in window.tabs.iter() {
                process::Command::new("tmux").arg("move-window").arg("-s")
                    .arg(format!("{}:{}", idx, tab.number)).spawn().unwrap();
                return;
            }
        }
    }

    fn attach_cmd(&self) {
        if self.len() != 1 {
            panic!("Can only get with a single result");
        }

        for (idx, _) in self.iter() {
            process::Command::new("tmux").arg("attach-session").arg("-t")
                .arg(format!("{}", idx)).spawn().unwrap();
            return;
        }
    }

    fn select_tabs(&self, searchterm: &str) -> WindowList {
        let mut out: WindowList = HashMap::new();
        for (idx, window) in self.iter() {
            let mut _win: Window = Window::new(vec![], window.attached);
            for tab in window.tabs.iter() {
                match tab.name.find(searchterm) {
                    Some(_) => {
                        let newtab: Tab = (*tab).clone();
                        _win.push(newtab);
                    },
                    None => {},
                }
            }
            if !_win.is_empty() {
                out.insert(*idx, _win);
            }
        }
        return out;
    }

    fn populate(&mut self) {
        let out = match process::Command::new("tmux")
            .arg("list-windows")
            .arg("-a")
            .output() {
                Ok(output) => output,
                Err(e) => panic!("failed to spawn: {}", e),
            };
        lazy_static! {
            static ref WINDOW_RE: regex::Regex = regex::Regex::new(r"^(\d+):(\d+): (.*) \((\d+) panes\) \[(\d+)x(\d+)\]").unwrap();
        }

        for line in String::from_utf8_lossy(&out.stdout).split('\n') {
            if line == "" { return }

            let cap = WINDOW_RE.captures(&line).unwrap();
            let win_: usize = cap[1].parse().unwrap();
            let new_tab = Tab::new(&cap[3],
            cap[2].parse().unwrap(),
            cap[4].parse().unwrap());

            match self.get_mut(&win_) {
                Some(window) => { window.push(new_tab); },
                None => unreachable!()
            };
        }
    }
}



fn main() {
    let matches = App::new("tinfo")
        .setting(AppSettings::TrailingVarArg)
        .version(VERSION)
        .author("richö butts <richo@psych0tik.net>")
        .about("Fetch information about running tmux sessions and windows")
        .group(ArgGroup::with_name("action")
               .args(&["get", "attach"])
               .required(false))
        .arg(Arg::with_name("search terms")
             .multiple(true))
        .arg(Arg::with_name("get")
             .short("G")
             .long("get")
             .help("String match a window and bring it here"))
        .arg(Arg::with_name("attach")
             .short("a")
             .long("attach")
             .help("Attach to matched session"))
        // .arg(Arg::with_name("fetch")
        //      .short("f")
        //      .long("fetch")
        //      .help("Fetch a window from it's session:window pair"))
        .get_matches();

    let windows = build_windowlist();

    if let Some(searchterms) = matches.values_of("search terms") {
        let query: String = searchterms.collect();
        let searched = windows.select_tabs(&query);
        if matches.is_present("get") {
            searched.get_cmd();
        } else if matches.is_present("attach") {
            searched.attach_cmd();
        } else {
            searched.dump();
        }
    } else {
        windows.dump();
    }
}

#[cfg(test)]
mod tests {
    extern crate toml;

    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_versions_all_up_to_date() {
        let mut fh = File::open("Cargo.toml").unwrap();
        let mut contents = String::new();
        fh.read_to_string(&mut contents).unwrap();

        let config = contents.parse::<toml::Value>().unwrap();

        assert_eq!(Some(VERSION), config["package"]["version"].as_str());
    }
}
