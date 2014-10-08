#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::io::Command;
use std::io::Reader;
use std::io::process::ProcessOutput;
use std::collections::hashmap::HashMap;
struct Tab {
    name: String,
    number: uint,
    panes: uint,
}

impl Tab {
    fn new(name: &str, number: uint, panes: uint) -> Tab {
        Tab { name: from_str(name).unwrap(), number: number, panes: panes }
    }

    fn clone(&self) -> Tab {
        Tab::new(self.name.as_slice(), self.number, self.panes)
    }
}

struct Window {
    pub tabs: Vec<Tab>,
}

impl Window {
    fn new(tabs: Vec<Tab>) -> Window {
        Window { tabs: tabs }
    }

    fn push(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    fn empty(&self) -> bool {
        return self.tabs.len() == 0;
    }
}

type WindowList = HashMap<uint, Window>;

trait WindowSearch {
    fn select_tabs(&self, searchterm: &str) -> Self;
    fn insert_or_push(&mut self, win: uint, tab: Tab);
    fn dump(&self);
}

impl WindowSearch for WindowList {
    fn dump(&self) {
        for (idx, window) in self.iter() {
            println!("Session: {}", idx);
            for tab in window.tabs.iter() {
                println!("  {}: {}", tab.number, tab.name);
            }
        }
    }

    fn select_tabs(&self, searchterm: &str) -> WindowList {
        let mut out: WindowList = HashMap::new();
        for (idx, window) in self.iter() {
            let mut _win: Window = Window::new(vec![]);
            for tab in window.tabs.iter() {
                match tab.name.as_slice().find_str(searchterm) {
                    Some(_) => {
                        let newtab: Tab = (*tab).clone();
                        _win.push(newtab);
                    },
                    None => {},
                }
            }
            if !_win.empty() {
                out.insert(*idx, _win);
            }
        }
        return out;
    }

    fn insert_or_push(&mut self, win: uint, tab: Tab) {
        // This is super lurky. Double borrow bug?
        if self.contains_key(&win) {
            match self.find_mut(&win) {
                Some(window) => { window.push(tab); },
                None => unreachable!()
            };
        } else {
            self.insert(win, Window::new(vec!(tab)));
        }
    }
}

static WINDOW_RE: regex::Regex = regex!(r"^(\d+):(\d+): (.*) \((\d+) panes\) \[(\d+)x(\d+)\]");

fn output_to_windows(rdr: &str) -> WindowList {
    let mut windows: WindowList = HashMap::new();

    for line in rdr.split('\n') {
        if line == "" { return windows }

        let cap = WINDOW_RE.captures(line.as_slice()).unwrap();
        let win_: uint = from_str(cap.at(1)).unwrap();
        let new_tab = Tab::new(cap.at(3),
                               from_str::<uint>(cap.at(2)).unwrap(),
                               from_str::<uint>(cap.at(4)).unwrap());


        windows.insert_or_push(win_, new_tab);
    }

    return windows;
}

#[allow(unused_variable)]
fn main() {
    let out = match Command::new("tmux").arg("list-windows").arg("-a").spawn() {
        Ok(process) => {
            let ProcessOutput { status, output, error } =
                process.wait_with_output().unwrap();

             String::from_utf8(output).unwrap()
        },
        Err(e) => fail!("failed to spawn: {}", e),
    };

    let windows = output_to_windows(out.as_slice());

    match std::os::args().len() {
        0 => unreachable!(),
        1 => windows.dump(),
        2 => {
            let searched = windows.select_tabs(std::os::args()[1].as_slice());
            searched.dump();
        },
        _ => fail!("Ooops"),
    }
}
