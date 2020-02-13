extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate getopts;
#[macro_use]
extern crate failure;

use getopts::Options;
use std::collections::HashMap;
use std::process;
use failure::Error;

#[derive(Debug, Clone)]
struct Tab {
    name: String,
    number: usize,
    panes: usize,
}

impl Tab {
    fn new(name: &str, number: usize, panes: usize) -> Tab {
        Tab {
            name: name.to_string(),
            number: number,
            panes: panes,
        }
    }
}

#[derive(Debug)]
struct Window {
    pub tabs: Vec<Tab>,
    pub attached: bool,
}

impl Window {
    fn new(tabs: Vec<Tab>, attached: bool) -> Window {
        Window {
            tabs: tabs,
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

fn build_windowlist() -> Result<WindowList, Error> {
    lazy_static! {
        static ref SESSION_RE: regex::Regex =
            regex::Regex::new(r"^(\d+) (\d+) (\d+)")
                .expect("Compiling regex");
    }

    let out = process::Command::new("tmux")
        .arg("list-sessions")
        .arg("-F").arg("#{session_name} #{session_windows} #{session_attached}")
        .output()?;
    let mut windows: WindowList = HashMap::new();

    for line in String::from_utf8_lossy(&out.stdout).split('\n') {
        if line == "" {
            break;
        }

        let cap = SESSION_RE.captures(&line)
            .ok_or(format_err!("Couldn't match line"))?;
        let id: usize = cap[1].parse()?;
        let num_windows: usize = cap[2].parse()?;
        let attached: usize = cap[3].parse()?;
        let vec = Vec::with_capacity(num_windows);
        windows.insert(id, Window::new(vec, attached > 0));
    }

    windows.populate();

    return Ok(windows);
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
                process::Command::new("tmux")
                    .arg("move-window")
                    .arg("-s")
                    .arg(format!("{}:{}", idx, tab.number))
                    .spawn()
                    .expect("Spawning tmux (Moving window)");
                return;
            }
        }
    }

    fn attach_cmd(&self) {
        if self.len() != 1 {
            panic!("Can only get with a single result");
        }

        for (idx, _) in self.iter() {
            process::Command::new("tmux")
                .arg("attach-session")
                .arg("-t")
                .arg(format!("{}", idx))
                .spawn()
                .expect("Spawning tmux (Attaching session)");
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
                    }
                    None => {}
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
            .output()
        {
            Ok(output) => output,
            Err(e) => panic!("failed to spawn: {}", e),
        };
        lazy_static! {
            static ref WINDOW_RE: regex::Regex =
                regex::Regex::new(r"^(\d+):(\d+): (.*) \((\d+) panes\) \[(\d+)x(\d+)\]")
                    .expect("Compiling window regex");
        }

        for line in String::from_utf8_lossy(&out.stdout).split('\n') {
            if line == "" {
                return;
            }

            let cap = WINDOW_RE.captures(&line).expect("Capturing windows");
            let win_: usize = cap[1].parse().expect("Parsing window number");
            let new_tab = Tab::new(
                &cap[3],
                cap[2].parse().expect("Parsing tab[2]"),
                cap[4].parse().expect("Prasing tab[4]"),
            );

            match self.get_mut(&win_) {
                Some(window) => {
                    window.push(new_tab);
                }
                None => unreachable!(),
            };
        }
    }
}

fn print_usage(opts: &Options) {
    let brief = "Usage: tinfo [options]";
    println!("{}", opts.usage(&brief));
}

fn main() -> Result<(), Error> {
    let windows = build_windowlist()?;

    let args: Vec<_> = std::env::args().collect();
    let mut opts = Options::new();
    opts.optflag("G", "get", "Bring matched window here");
    opts.optflag("a", "attach", "Attach to matched session");
    opts.optflag("h", "help", "Show this help");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}\n", f.to_string());
            print_usage(&opts);
            ::std::process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&opts);
        return Ok(());
    }

    if !matches.free.is_empty() {
        let searched = windows.select_tabs(&matches.free[0]);
        if matches.opt_present("G") {
            searched.get_cmd();
        } else if matches.opt_present("a") {
            searched.attach_cmd();
        } else {
            searched.dump();
        }
    } else {
        windows.dump();
    }

    Ok(())
}
