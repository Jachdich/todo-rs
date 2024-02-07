#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(dead_code)]

mod parser;

use chrono::Datelike;
use chrono::{DateTime, Local};

use linked_hash_map::LinkedHashMap;
use std::{convert::TryInto, io::Read};
use std::io::Write;
use std::path::Path;
use yaml_rust::Yaml;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ListItem {
    name: String,
    date: Option<chrono::NaiveDate>,
    priority: i32,
    done: bool,
    repeat_every: i64,
    repeat_next: i64,
}

#[derive(Debug)]
pub enum ListEntry {
    Item(ListItem),
    List(String),
}

fn serialise_date(date: chrono::NaiveDate) -> i32 {
    date.num_days_from_ce()
}

fn deserialise_date(date: i32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_num_days_from_ce_opt(date).unwrap()
}

impl ListEntry {
    fn to_yaml(&self) -> Yaml {
        match self {
            Self::Item(item) => {
                let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
                map.insert(Yaml::String("type".into()), Yaml::String("item".into()));
                map.insert(
                    Yaml::String("name".into()),
                    Yaml::String(item.name.clone()),
                );
                map.insert(Yaml::String("done".into()), Yaml::Boolean(item.done));
                if item.priority != 0 {
                    map.insert(
                        Yaml::String("priority".into()),
                        Yaml::Integer(item.priority.into()),
                    );
                }
                if let Some(date) = item.date {
                    map.insert(
                        Yaml::String("date".into()),
                        Yaml::Integer(serialise_date(date).into()),
                    );
                }
                if item.repeat_every != 0 {
                    map.insert(
                        Yaml::String("repeat_every".into()),
                        Yaml::Integer(item.repeat_every),
                    );
                }
                if item.repeat_next != 0 {
                    map.insert(
                        Yaml::String("repeat_next".into()),
                        Yaml::Integer(item.repeat_next),
                    );
                }
                Yaml::Hash(map)
            }
            Self::List(list) => {
                let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
                map.insert(Yaml::String("type".into()), Yaml::String("list".into()));
                map.insert(Yaml::String("name".into()), Yaml::String(list.to_owned()));
                Yaml::Hash(map)
            }
        }
    }

    fn from_yaml(y: &Yaml) -> Self {
        let ty = y["type"].as_str().unwrap();
        match ty {
            "item" => Self::Item(ListItem {
                name: y["name"].as_str().unwrap().to_owned(),
                date: y["date"].as_i64().map(|date| deserialise_date(date.try_into().expect("Date is too large"))),
                priority: i32::try_from(y["priority"].as_i64().unwrap_or(0)).expect("Priority is too large"),
                done: y["done"].as_bool().unwrap_or(false),
                repeat_every: y["repeat_every"].as_i64().unwrap_or(0),
                repeat_next: y["repeat_next"].as_i64().unwrap_or(0),
            }),

            "list" => Self::List(y["name"].as_str().unwrap().to_owned()),

            _ => panic!("Expected either 'item' or 'list', got '{}'", ty),
        }
    }
}

#[derive(Debug)]
pub struct TodoList {
    name: String,
    items: Vec<ListEntry>,
}

impl TodoList {
    fn new(name: String) -> Self {
        Self {
            name,
            items: Vec::new(),
        }
    }

    fn to_yaml(&self) -> Yaml {
        let mut out: Vec<Yaml> = Vec::new();
        for item in &self.items {
            out.push(item.to_yaml());
        }

        let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
        map.insert(
            Yaml::String("name".into()),
            Yaml::String(self.name.clone()),
        );
        map.insert(Yaml::String("entries".into()), Yaml::Array(out));
        Yaml::Hash(map)
    }

    fn from_yaml(val: &Yaml) -> Self {
        let name = val["name"].as_str().unwrap().to_owned();
        let mut entries: Vec<ListEntry> = Vec::new();
        for y in val["entries"].as_vec().unwrap() {
            entries.push(ListEntry::from_yaml(y));
        }
        Self {
            name,
            items: entries,
        }
    }

    fn num_valid_entries<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &[Self],
        predicate: &mut F,
    ) -> usize {
        self.items
            .iter()
            .map(|item| match item {
                ListEntry::Item(item) => {
                    usize::from(predicate(&item))
                }
                ListEntry::List(name) => get_list_by_name(all, name)
                    .unwrap()
                    .num_valid_entries(all, predicate),
            })
            .sum()
    }

    fn print<F: FnMut(&&ListItem) -> bool>(&self, all: &[Self], mut predicate: F) -> String {
        let mut acc = String::new();
        let max = self.get_max_size(all, 0, &mut predicate);
        self.print_inner(all, 0, max, &mut predicate, true, &mut acc);
        acc
    }

    fn print_without_date<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &[Self],
        mut predicate: F,
    ) -> String {
        let mut acc = String::new();
        let max = self.get_max_size(all, 0, &mut predicate);
        self.print_inner(all, 0, max, &mut predicate, false, &mut acc);
        acc
    }

    fn print_inner<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &[Self],
        indent: usize,
        maxsize: usize,
        predicate: &mut F,
        print_date: bool,
        acc: &mut String,
    ) {
        use std::fmt::Write;
        if self.num_valid_entries(all, predicate) == 0 {
            return;
        }
        let entries_to_print = self
            .items
            .iter()
            .filter(|item| match item {
                ListEntry::Item(item) => predicate(&item),
                ListEntry::List(_) => true,
            })
            .collect::<Vec<&ListEntry>>();

        let all_done = self.num_valid_entries(all, &mut |item: &&ListItem| !item.done) == 0;
        writeln!(
            acc,
            "{}{}{}:",
            if all_done { "✓" } else { " " },
            " ".repeat(indent * 4),
            self.name
        )
        .unwrap();
        let indent = indent + 1;
        let indentstr = " ".repeat(indent * 4);
        for entry in entries_to_print {
            match entry {
                ListEntry::List(list_name) => {
                    get_list_by_name(all, list_name)
                        .unwrap()
                        .print_inner(all, indent, maxsize, predicate, print_date, acc);
                }
                ListEntry::Item(item) => {
                    if print_date && item.date.is_some() || item.priority != 0 {
                        let tabs = " ".repeat(maxsize - indentstr.len() - item.name.len());
                        let duration = item.date.unwrap() - chrono::Local::now().naive_local().date();
                        let time_until =
                            if duration.num_days() == 1 {
                                "in 1 day".into()
                            } else if duration.num_days() < 0 {
                                format!("{} days ago", -duration.num_days())
                            } else {
                                format!("in {} days", duration.num_days())
                            };
                        writeln!(
                            acc,
                            "{}{}{}{}\t{} ({})",
                            if item.done { "✓" } else { " " },
                            indentstr,
                            item.name,
                            tabs,
                            item.date.unwrap().format("%d/%m/%Y"),
                            time_until,
                            // item.priority
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            acc,
                            "{}{}{}",
                            if item.done { "✓" } else { " " },
                            indentstr,
                            item.name
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    fn get_max_size<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &[Self],
        indent: usize,
        predicate: &mut F,
    ) -> usize {
        let mut max = indent * 4 + self.name.len() + 1;
        let indent = indent + 1;
        for entry in &self.items {
            match entry {
                ListEntry::List(list_name) => {
                    max = std::cmp::max(
                        max,
                        get_list_by_name(all, list_name)
                            .unwrap()
                            .get_max_size(all, indent, predicate),
                    );
                }
                ListEntry::Item(item) if predicate(&item) => {
                    max = std::cmp::max(max, indent * 4 + item.name.len());
                }
                ListEntry::Item(_) => (),
            }
        }
        max
    }
}

fn load(fname: &Path) -> std::io::Result<Vec<TodoList>> {
    let mut file = std::fs::File::open(fname)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    match parser::parse_str(&contents) {
        Ok(l) => Ok(l),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e.0)),
    }
}

fn save(fname: &Path, lists: &[TodoList]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(fname)?;
    let out = parser::emit_str(lists);

    file.write_all(&out.into_bytes())?;
    Ok(())
}

#[rustfmt::skip]
fn usage() -> String {
    "Usage:\ttodo <action> ...\n".to_string() +
    "\tls  lists                        Show all the lists\n" +
    "\tl   list <list name> [--small]   Show the items in the specified list.\n" +
    "\tn   new <name>                   Create a new list\n" +
    "\trl  rmlist <list>                Delete the specified list\n" +
    "\ta   add <list> <name> [date]     Add a new item to the specified list\n" +
    "\tal  addlist <dest> <src>         Add a reference of list <src> to list <dest>\n" +
    "\td   done <list> <item>           Mark the specified item as done\n" +
    "\tda  doneall <list>               Mark all items in list as done\n" +
    "\tuda undoneall <list>             Mark all items in list as not done\n" +
    "\trm  remove <list> <item>         Remove <item> from <list>\n" +
    "\tmv  move <source> <item> <dest>  Move an <item> from the list <source> to <dest>\n" +
    "\tmva moveall <source> <dest>      Move every item from <source> into <dest>. Does not move sublist of source into itself\n" +
    "\trn  rename <list> <old> <new>    Rename an item in <list> from <old> to <new>\n" +
    "\trl  renamelist <old> <new>       Rename the list <old> to <new>\n" +
    // println!("\tr   repeat <list> <item> <time>  Set an item to repeat (mark as un-done) every <time>");
    "\tar  autorm <list>                Remove all items in <list> that are marked as done\n" +
    "\tt   today <list> [--short]       List all tasks with a deadline of today.\n                                         If --short is passed, return only the number of tasks, do not list them.\n" +
    "\tw   week <list> [--short]        List all tasks with a deadline of within the next 7 days\n" +
    "\tod  overdue <list> [--short]     List all non-completed tasks with a deadline in the past\n\n" +
    "When specifying lists and items, only the first few characters of their names are needed, as long a they\n" +
    "uniquely identify a single list or item. For example in a list containing both 'orange' and 'organic',\n" +
    "'or' would not work but 'ora' would be interpreted as 'orange'. In a list containing 'or' and 'orange',\n" + 
    "'or' would match 'or' because it's an exact match. 'ora' would be necessary to match 'orange'.\n\n" +
    "The last argument to a command need not be quoted as additional arguments are automatically concatinated\n" +
    "with a space. For example, `todo add list this item has multiple words` is valid."
}

fn get_list_by_name<'a>(lists: &'a [TodoList], name: &str) -> Result<&'a TodoList, String> {
    let mut item: Result<&'a TodoList, String> = Err(format!("List '{name}' does not exist"));
    for i in lists {
        if i.name == name {
            return Ok(i);
        }
        if i.name.starts_with(name) {
            if item.is_ok() {
                return Err(format!(
                    "List '{name}' is not specific enough to match a single item"
                ));
            }
            item = Ok(i);
        }
    }
    item
}

fn get_mut_list_by_name<'a>(
    lists: &'a mut [TodoList],
    name: &str,
) -> Result<&'a mut TodoList, String> {
    let mut item: Result<&'a mut TodoList, String> = Err(format!("List '{name}' does not exist"));
    for i in lists {
        if i.name == name {
            return Ok(i);
        }
        if i.name.starts_with(name) {
            if item.is_ok() {
                return Err(format!(
                    "List '{name}' is not specific enough to match a single item"
                ));
            }
            item = Ok(i);
        }
    }
    item
}

fn get_index_by_name(list: &TodoList, itemname: &str) -> Result<usize, String> {
    let mut idx = Err(format!("Item '{itemname}' does not exist"));
    for (item_index, item) in list.items.iter().enumerate() {
        let this_item_name = match &item {
            ListEntry::List(l) => l,
            ListEntry::Item(i) => &i.name,
        };
        if this_item_name == itemname {
            idx = Ok(item_index);
        }
    }

    if idx.is_err() {
        for (item_index, item) in list.items.iter().enumerate() {
            let this_item_name = match &item {
                ListEntry::List(l) => l,
                ListEntry::Item(i) => &i.name,
            };
            if this_item_name.starts_with(itemname) {
                if idx.is_err() {
                    idx = Ok(item_index);
                } else {
                    return Err(format!(
                        "Item '{itemname}' is not specific enough to match a single item"
                    ));
                }
            }
        }
    }
    idx
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%d/%m/%y").map_or_else(|_| chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").ok(), Some)
}

type CmdResult = Result<(String, bool), String>;

fn cmd_list(lists: &[TodoList], name: &str) -> CmdResult {
    if let Some(name) = name.strip_suffix("--short") {
        let list = get_list_by_name(lists, name.trim_end())?;
        let mut item_names: Vec<&str> = Vec::new();
        for i in &list.items {
            if let ListEntry::Item(i) = i {
                if !i.done {
                    item_names.push(&i.name);
                }
            }
        }
        Ok((item_names.join(", "), false))
    } else {
        let list = get_list_by_name(lists, name)?;
        Ok((list.print(lists, |_| true), false))
    }
}

fn cmd_lists(lists: &[TodoList]) -> (std::string::String, bool) {
    let mut res = String::new();
    for i in lists {
        res.push_str(&i.name);
        res.push('\n');
    }
    (res, false)
}

fn cmd_new(lists: &mut Vec<TodoList>, name: String) -> (std::string::String, bool) {
    lists.push(TodoList::new(name));
    (String::new(), true)
}

fn cmd_rmlist(lists: &mut Vec<TodoList>, name: &str) -> CmdResult {
    let name = get_list_by_name(lists, name)?.name.clone();
    lists.retain(|l| l.name != name);
    Ok((String::new(), true))
}

fn cmd_add(lists: &mut [TodoList], args: &[String]) -> CmdResult {
    let list = get_mut_list_by_name(lists, &args[0])?;
    let last_arg = &args[args.len() - 1];

    let (name, date) = parse_date(last_arg).map_or_else(|| (args[1..].join(" "), None), |timestamp| (args[1..(args.len() - 1)].join(" "), Some(timestamp)));

    list.items.push(ListEntry::Item(ListItem {
        name,
        date,
        priority: 0,
        done: false,
        repeat_every: 0,
        repeat_next: 0,
    }));
    Ok((String::new(), true))
}

fn cmd_addlist(lists: &mut [TodoList], dest_list: &str, src_list: &str) -> CmdResult {
    let lname = get_list_by_name(lists, src_list)?.name.clone();
    let list = get_mut_list_by_name(lists, dest_list)?;
    list.items.push(ListEntry::List(lname));
    Ok((String::new(), true))
}

fn cmd_done(lists: &mut [TodoList], list_name: &str, item_name: &str) -> CmdResult {
    let list = get_mut_list_by_name(lists, list_name)?;
    let idx = get_index_by_name(list, item_name)?;
    if let ListEntry::Item(i) = &mut list.items[idx] {
        i.done = !i.done;
        Ok((String::new(), true))
    } else {
        Err("You can't done a list silly (todo add this feature cos its cool)".to_string())
    }
}

fn cmd_doneall(lists: &mut [TodoList], list_name: &str, target_state: bool) -> CmdResult {
    let list = get_mut_list_by_name(lists, list_name)?;
    for item in &mut list.items {
        if let ListEntry::Item(item) = item {
            item.done = target_state;
        }
    }
    Ok((String::new(), true))
}

fn cmd_remove(lists: &mut [TodoList], list_name: &str, item_name: &str) -> CmdResult {
    let list = get_mut_list_by_name(lists, list_name)?;
    let idx = get_index_by_name(list, item_name)?;
    list.items.remove(idx);
    Ok((String::new(), true))
}

fn cmd_rename(lists: &mut [TodoList], list_name: &str, old: &str, new: &str) -> CmdResult {
    let list = get_mut_list_by_name(lists, list_name)?;
    let idx = get_index_by_name(list, old)?;
    if let ListEntry::Item(i) = &mut list.items[idx] {
        i.name = new.to_owned();
        Ok((String::new(), true))
    } else {
        Err("Renaming a list entry doesn't really make sense".to_string())
    }
}

fn cmd_rnlist(lists: &mut [TodoList], old: &str, new: &str) -> CmdResult {
    let list = get_mut_list_by_name(lists, old)?;
    list.name = new.to_owned();
    Ok((String::new(), true))
}

fn cmd_move(
    lists: &mut [TodoList],
    src_list_name: &str,
    dest_list_name: &str,
    item_name: &str,
) -> CmdResult {
    // check that the dest list exists first
    // otherwise, either the borrow checker will yell at me (lists is borrowed mutable twice in src_list and dest_list)
    // or a nonexistant dest list will casue the item to be removed and not replaced
    let _ = get_list_by_name(lists, dest_list_name)?;
    let src_list = get_mut_list_by_name(lists, src_list_name)?;
    let item_idx = get_index_by_name(src_list, item_name)?;
    let item = src_list.items.remove(item_idx);

    let dest_list = get_mut_list_by_name(lists, dest_list_name).unwrap(); // already checked
    dest_list.items.push(item);
    Ok((String::new(), true))
}
fn cmd_moveall(lists: &mut [TodoList], src_list_name: &str, dest_list_name: &str) -> CmdResult {
    // check that the dest list exists first
    // otherwise, either the borrow checker will yell at me (lists is borrowed mutable twice in src_list and dest_list)
    // or a nonexistant dest list will casue the item to be removed and not replaced
    let _ = get_list_by_name(lists, dest_list_name)?;
    let src_list = get_mut_list_by_name(lists, src_list_name)?;
    // Don't move a list into itself. Does not check recursively, so caution is still needed.
    // let mut items = src_list
    //     .items
    //     .extract_if(|item| match item {
    //         ListEntry::List(list) => list != dest_list_name,
    //         _ => true,
    //     })
    //     .collect::<Vec<ListEntry>>();

    // f***ing extract_if is nightly, so I guess I'll just implement it myself...
    let mut items = Vec::new();
    let mut i = 0;
    while i < src_list.items.len() {
        if matches!(&src_list.items[i], ListEntry::List(list) if list == dest_list_name) {
            i += 1;
        } else {
            let val = src_list.items.remove(i);
            items.push(val);
        }
    }

    let dest_list = get_mut_list_by_name(lists, dest_list_name).unwrap(); // already checked
    dest_list.items.append(&mut items);
    Ok((String::new(), true))
}

fn cmd_autorm(lists: &mut [TodoList], list_name: &str) -> CmdResult {
    let list = get_mut_list_by_name(lists, list_name)?;
    list.items.retain(|item| match item {
        ListEntry::Item(item) => !item.done,
        ListEntry::List(_) => true,
    });
    Ok((String::new(), true))
}

fn cmd_timeperiods(lists: &[TodoList], args: &[String], op: &str) -> CmdResult {
    use chrono::Duration;
    // find out the minimum and maximum allowed difference between the deadline date and today
    let (min_diff, max_diff, description) = match op {
        "today" | "t" => (Duration::days(0), Duration::days(1), "today"),
        "week" | "w" => (Duration::days(1), Duration::days(7), "this week"),
        "overdue" | "od" => (
            Duration::days(-365 * 1000), //1000 years ought to be enough
            Duration::days(0),
            "overdue",
        ),
        _ => unreachable!(),
    };

    let (list_name, short) = if args[args.len() - 1] == "--short" {
        (args[..args.len() - 1].join(" "), true)
    } else {
        (args.join(" "), false)
    };

    let list = get_list_by_name(lists, &list_name)?;
    let now: DateTime<Local> = Local::now();
    let today = now.date_naive();
    let mut filter = |item: &&ListItem| {
        item.date.is_some()
            && !item.done
            && item.date.unwrap() - today < max_diff
            && item.date.unwrap() - today >= min_diff
    };
    if short {
        let num = list.num_valid_entries(lists, &mut filter);
        if num == 0 {
            // don't bother printing if there's none. maybe should make this configurable.
            return Ok((String::new(), false));
        }
        Ok((
            format!(
                "You have {} deadline{} {}\n",
                num,
                if num == 1 { "" } else { "s" },
                description
            ),
            false,
        ))
    } else {
        Ok((list.print(lists, filter), false))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("{}", usage());
        return;
    }
    let mut list_file =
        dirs::config_dir().expect("Unable to locate config directory. What OS are you on?!");
    list_file.push("todo");
    std::fs::create_dir_all(&list_file)
        .expect("Unable to create the config directory. Do you have the right permissions?");
    list_file.push("todo.txt");
    let mut lists = load(list_file.as_path()).unwrap_or_default();

    let nargs = args.len() - 2;
    #[rustfmt::skip] // ree it looks better all nicely indented
    let result = match args[1].as_str() {
        "list"    | "l"       if nargs >= 1 => cmd_list(&lists, &args[2..].join(" ")),
        "lists"   | "ls"      if nargs == 0 => Ok(cmd_lists(&lists)),
        "new"     | "n"       if nargs > 0 => Ok(cmd_new(&mut lists, args[2..].join(" "))),
        "rmlist"  | "rl"      if nargs > 0 => cmd_rmlist(&mut lists, &args[2..].join(" ")),
        "add"     | "a"       if nargs >= 2 => cmd_add(&mut lists, &args[2..]),
        "addlist" | "al"      if nargs == 2 => cmd_addlist(&mut lists, &args[2], &args[3]),
        "done"    | "d"       if nargs >= 2 => cmd_done(&mut lists, &args[2], &args[3..].join(" ")),
        "autorm"  | "ar"      if nargs >= 1 => cmd_autorm(&mut lists, &args[2..].join(" ")),
        "rename"  | "rn"      if nargs >= 3 => cmd_rename(&mut lists, &args[2], &args[3], &args[4..].join(" ")),
        "renamelist" | "rl"   if nargs >= 2 => cmd_rnlist(&mut lists, &args[2], &args[3..].join(" ")),
        "rm" | "remove" | "r" if nargs >= 2 => cmd_remove(&mut lists, &args[2], &args[3..].join(" ")),
        "move" | "mv" | "m"   if nargs >= 3 => cmd_move(&mut lists, &args[2], &args[4..].join(" "), &args[3]),
        "moveall" | "mvall"
        | "mva" | "ma"        if nargs >= 2 => cmd_moveall(&mut lists, &args[2], &args[3..].join(" ")),
        "today" | "t"
        | "week" | "w"
        | "overdue" | "od"    if nargs >= 1 => cmd_timeperiods(&lists, &args[2..], &args[1]),
        "doneall" | "da" | "undoneall" | "uda" if nargs >= 1 => cmd_doneall(
            &mut lists,
            &args.join(" "),
            args[1] == "doneall" || args[1] == "da"
        ),
        _ => Err(usage()),
    };
    match result {
        Ok((msg, modified)) => {
            print!("{msg}");
            if modified {
                save(list_file.as_path(), &lists).unwrap();
            }
        }
        Err(e) => eprintln!("{e}"),
    }
}
