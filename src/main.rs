mod parser;

use chrono::Datelike;
use chrono::{DateTime, Local};
use dirs;
use linked_hash_map::LinkedHashMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

struct ListItem {
    name: String,
    date: chrono::NaiveDate,
    priority: i32,
    done: bool,
    repeat_every: i64,
    repeat_next: i64,
}

enum ListEntry {
    Item(ListItem),
    List(String),
}

fn serialise_date(date: &chrono::NaiveDate) -> i32 {
    date.num_days_from_ce()
}

fn deserialise_date(date: i32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_num_days_from_ce_opt(date).unwrap()
}

impl ListEntry {
    fn to_yaml(&self) -> Yaml {
        match self {
            ListEntry::Item(item) => {
                let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
                map.insert(Yaml::String("type".into()), Yaml::String("item".into()));
                map.insert(
                    Yaml::String("name".into()),
                    Yaml::String(item.name.to_owned()),
                );
                map.insert(Yaml::String("done".into()), Yaml::Boolean(item.done));
                if item.priority != 0 {
                    map.insert(
                        Yaml::String("priority".into()),
                        Yaml::Integer(item.priority.into()),
                    );
                }
                if item.date.num_days_from_ce() != 0 {
                    map.insert(
                        Yaml::String("date".into()),
                        Yaml::Integer(serialise_date(&item.date).into()),
                    );
                }
                if item.repeat_every != 0 {
                    map.insert(
                        Yaml::String("repeat_every".into()),
                        Yaml::Integer(item.repeat_every.into()),
                    );
                }
                if item.repeat_next != 0 {
                    map.insert(
                        Yaml::String("repeat_next".into()),
                        Yaml::Integer(item.repeat_next.into()),
                    );
                }
                Yaml::Hash(map)
            }
            ListEntry::List(list) => {
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
            "item" => ListEntry::Item(ListItem {
                name: y["name"].as_str().unwrap().to_owned(),
                date: deserialise_date(y["date"].as_i64().unwrap_or(0) as i32),
                priority: y["priority"].as_i64().unwrap_or(0) as i32,
                done: y["done"].as_bool().unwrap_or(false),
                repeat_every: y["repeat_every"].as_i64().unwrap_or(0),
                repeat_next: y["repeat_next"].as_i64().unwrap_or(0),
            }),

            "list" => ListEntry::List(y["name"].as_str().unwrap().to_owned()),

            _ => panic!("Expected either 'item' or 'list', got '{}'", ty),
        }
    }
}

struct TodoList {
    name: String,
    items: Vec<ListEntry>,
}

impl TodoList {
    fn new(name: String) -> Self {
        TodoList {
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
            Yaml::String(self.name.to_owned()),
        );
        map.insert(Yaml::String("entries".into()), Yaml::Array(out));
        Yaml::Hash(map)
    }

    fn from_yaml(val: &Yaml) -> Self {
        let name = val["name"].as_str().unwrap().to_owned();
        let mut entries: Vec<ListEntry> = Vec::new();
        for y in val["entries"].as_vec().unwrap() {
            entries.push(ListEntry::from_yaml(&y));
        }
        Self {
            name,
            items: entries,
        }
    }

    fn num_valid_entries<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &Vec<TodoList>,
        predicate: &mut F,
    ) -> usize {
        self.items
            .iter()
            .map(|item| match item {
                ListEntry::Item(item) => {
                    if predicate(&item) {
                        1
                    } else {
                        0
                    }
                }
                ListEntry::List(name) => get_list_by_name(all, name)
                    .unwrap()
                    .num_valid_entries(all, predicate),
            })
            .sum()
    }

    fn print<F: FnMut(&&ListItem) -> bool>(&self, all: &Vec<TodoList>, mut predicate: F) {
        let max = self.get_max_size(&all, 0);
        self.print_inner(all, 0, max, &mut predicate, true);
    }

    fn print_without_date<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &Vec<TodoList>,
        mut predicate: F,
    ) {
        let max = self.get_max_size(&all, 0);
        self.print_inner(all, 0, max, &mut predicate, false);
    }

    fn print_inner<F: FnMut(&&ListItem) -> bool>(
        &self,
        all: &Vec<TodoList>,
        indent: usize,
        maxsize: usize,
        predicate: &mut F,
        print_date: bool,
    ) {
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

        println!("{}{}:", " ".repeat(indent * 4), self.name);
        let indent = indent + 1;
        let indentstr = " ".repeat(indent * 4 - 1);
        for entry in entries_to_print {
            match entry {
                ListEntry::List(list_name) => {
                    get_list_by_name(all, list_name)
                        .unwrap()
                        .print_inner(all, indent, maxsize, predicate, print_date);
                }
                ListEntry::Item(item) => {
                    if print_date && item.date.num_days_from_ce() != 0 || item.priority != 0 {
                        let tabs = " ".repeat(maxsize - indentstr.len() - item.name.len());
                        println!(
                            "{}{}{}{}\t{}",
                            if item.done { "✓" } else { " " },
                            indentstr,
                            item.name,
                            tabs,
                            item.date.format("%d/%m/%Y"),
                            // item.priority
                        );
                    } else {
                        println!(
                            "{}{}{}",
                            if item.done { "✓" } else { " " },
                            indentstr,
                            item.name
                        )
                    }
                }
            }
        }
    }
    fn get_max_size(&self, all: &Vec<TodoList>, indent: usize) -> usize {
        let mut max = indent * 4 + self.name.len() + 1;
        let indent = indent + 1;
        for entry in &self.items {
            match entry {
                ListEntry::List(list_name) => {
                    max = std::cmp::max(
                        max,
                        get_list_by_name(all, list_name)
                            .unwrap()
                            .get_max_size(all, indent),
                    );
                }
                ListEntry::Item(item) => max = std::cmp::max(max, indent * 4 + item.name.len()),
            }
        }
        max
    }
}

fn load_yaml(fname: &Path) -> std::io::Result<Vec<TodoList>> {
    let mut file = std::fs::File::open(fname)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let yaml_parsed = YamlLoader::load_from_str(&contents).unwrap();

    let mut lists: Vec<TodoList> = Vec::new();
    for v in yaml_parsed {
        lists.push(TodoList::from_yaml(&v));
    }
    Ok(lists)
}

fn save_yaml(fname: &Path, lists: &Vec<TodoList>) -> std::io::Result<()> {
    let mut file = std::fs::File::create(fname)?;
    let mut out = String::new();

    for list in lists {
        {
            let mut emitter = YamlEmitter::new(&mut out);
            emitter.dump(&list.to_yaml()).unwrap();
        }
        out.push('\n');
    }

    file.write_all(&out.into_bytes())?;
    Ok(())
}

#[rustfmt::skip]
fn usage() {
    println!("Usage:\ttodo <action> ...");
    println!("\tls  lists                        Show all the lists");
    println!("\tl   list <list name>             Show the items in the specified list");
    println!("\tn   new <name>                   Create a new list");
    println!("\trl  rmlist <list>                Delete the specified list");
    println!("\ta   add <list> <name> [date]     Add a new item to the specified list");
    println!("\tal  addlist <dest> <src>         Add a reference of list <src> to list <dest>");
    println!("\td   done <list> <item>           Mark the specified item as done");
    println!("\tda  doneall <list>               Mark all items in list as done");
    println!("\tuda undoneall <list>             Mark all items in list as not done");
    println!("\trm  remove <list> <item>         Remove <item> from <list>");
    println!("\tmv  move <list> <item> <list>    Move an <item> from <list> to another <list>");
    // println!("\tr   repeat <list> <item> <time>  Set an item to repeat (mark as un-done) every <time>");
    println!("\tar  autorm <list>                Remove all items in <list> that are marked as done");
    println!("\tt   today <list> [--short]       List all tasks with a deadline of today.\n                                         If --short is passed, return only the number of tasks, do not list them.");
    println!("\tw   week <list> [--short]        List all tasks with a deadline of within the next 7 days");
    println!("\tod  overdue <list> [--short]     List all non-completed tasks with a deadline in the past");
}

fn get_list_by_name<'a>(lists: &'a Vec<TodoList>, name: &str) -> Option<&'a TodoList> {
    let mut item: Option<&'a TodoList> = None;
    for i in lists {
        if i.name == name {
            return Some(i);
        }
        if i.name.starts_with(name) {
            if let Some(_) = item {
                return None;
            }
            item = Some(i);
        }
    }
    item
}

fn get_mut_list_by_name<'a>(lists: &'a mut Vec<TodoList>, name: &str) -> Option<&'a mut TodoList> {
    let mut item: Option<&'a mut TodoList> = None;
    for i in lists {
        if i.name == name {
            return Some(i);
        }
        if i.name.starts_with(name) {
            if let Some(_) = item {
                return None;
            }
            item = Some(i);
        }
    }
    item
}

fn get_index_by_name(list: &TodoList, itemname: &str) -> usize {
    let mut idx: usize = usize::MAX;
    let mut cidx: usize = 0;
    for item in &list.items {
        let citemname = match &item {
            ListEntry::List(l) => &l,
            ListEntry::Item(i) => &i.name,
        };
        if citemname == itemname {
            idx = cidx;
        }

        cidx += 1;
    }

    if idx == usize::MAX {
        cidx = 0;
        for item in &list.items {
            let citemname = match &item {
                ListEntry::List(l) => &l,
                ListEntry::Item(i) => &i.name,
            };
            if citemname.starts_with(itemname) {
                if idx == usize::MAX {
                    idx = cidx;
                } else {
                    return usize::MAX - 1;
                }
            }
            cidx += 1;
        }
    }
    idx
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%d/%m/%y") {
        Some(d)
    } else if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y") {
        Some(d)
    } else {
        None
    }
}

fn main() {
    parse_date("");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
        return;
    }
    let mut list_file =
        dirs::config_dir().expect("Unable to locate config directory. What OS are you on?!");
    list_file.push("todo");
    std::fs::create_dir_all(&list_file)
        .expect("Unable to create the config directory. Do you have the right permissions?");
    list_file.push("todo.yml");
    let mut lists = load_yaml(list_file.as_path()).unwrap_or(Vec::new());

    let mut modified = false;
    match args[1].as_str() {
        "list" | "l" => {
            if args.len() < 3 || args.len() > 4 {
                usage();
                return;
            }

            if let Some(list) = get_list_by_name(&lists, &args[2]) {
                list.print(&lists, |_| true);
            } else {
                println!("List \"{}\" does not exist!", &args[2]);
            }
        }
        "lists" | "ls" => {
            for i in &lists {
                println!("{}", i.name);
            }
        }
        "new" | "n" => {
            if args.len() < 3 {
                usage();
                return;
            }
            let list_name = args[2..].join(" ");
            lists.push(TodoList::new(list_name));
            modified = true;
        }
        "rmlist" | "rl" => {
            if args.len() < 3 {
                usage();
                return;
            }
            let item = args[2..].join(" ");
            let name = if let Some(list) = get_list_by_name(&lists, &item) {
                list.name.to_owned()
            } else {
                println!("List \"{}\" does not exist!", &item);
                return;
            };
            lists.retain(|l| l.name != name);
            modified = true;
        }
        "add" | "a" => {
            if args.len() <= 3 {
                usage();
                return;
            }
            if let Some(list) = get_mut_list_by_name(&mut lists, &args[2]) {
                let last_arg = &args[args.len() - 1];

                let (name, date) = if let Some(timestamp) = parse_date(last_arg) {
                    (args[3..(args.len() - 1)].join(" "), timestamp)
                } else {
                    (
                        args[3..].join(" "),
                        chrono::NaiveDate::from_num_days_from_ce_opt(0).unwrap(),
                    )
                };

                list.items.push(ListEntry::Item(ListItem {
                    name,
                    date,
                    priority: 0,
                    done: false,
                    repeat_every: 0,
                    repeat_next: 0,
                }));
                modified = true;
            } else {
                println!("List \"{}\" does not exist!", &args[2]);
            }
        }

        "addlist" | "al" => {
            if args.len() <= 3 {
                usage();
                return;
            }
            let lname = if let Some(list2) = get_list_by_name(&lists, &args[3]) {
                list2.name.to_owned()
            } else {
                println!("List \"{}\" does not exist!", &args[3]);
                "".to_string()
            };
            if let Some(list) = get_mut_list_by_name(&mut lists, &args[2]) {
                if lname != "" {
                    list.items.push(ListEntry::List(lname));
                    modified = true;
                }
            } else {
                println!("List \"{}\" does not exist!", &args[2]);
            }
        }
        "done" | "d" => {
            if args.len() == 4 {
                let name = &args[2];
                if let Some(list) = get_mut_list_by_name(&mut lists, name) {
                    let itemname = &args[3];
                    let idx = get_index_by_name(list, itemname);
                    if idx == usize::MAX {
                        println!("Item \"{}\" does not exist!", itemname);
                    } else if idx == usize::MAX - 1 {
                        println!(
                            "Item \"{}\" is not specific enough to match a single item",
                            itemname
                        );
                    } else {
                        if let ListEntry::Item(i) = &mut list.items[idx] {
                            i.done = !i.done;
                            modified = true;
                        } else {
                            println!(
                                "You can't done a list silly (todo add this feature cos its cool)"
                            );
                        }
                    }
                } else {
                    println!("List \"{}\" does not exist!", name);
                }
            } else {
                usage();
            }
        }
        "doneall" | "da" | "undoneall" | "uda" => {
            let target_state = args[1] == "doneall" || args[1] == "da";
            if args.len() == 3 {
                let name = &args[2];
                if let Some(list) = get_mut_list_by_name(&mut lists, name) {
                    for item in list.items.iter_mut() {
                        if let ListEntry::Item(item) = item {
                            item.done = target_state;
                        }
                    }
                    modified = true;
                } else {
                    println!("List \"{}\" does not exist!", name);
                }
            } else {
                usage();
            }
        }
        "rm" | "remove" | "r" => {
            if args.len() == 4 {
                let name = &args[2];
                if let Some(list) = get_mut_list_by_name(&mut lists, name) {
                    let itemname = &args[3];
                    let idx = get_index_by_name(list, itemname);
                    if idx == usize::MAX {
                        println!("Item \"{}\" does not exist!", itemname);
                    } else if idx == usize::MAX - 1 {
                        println!(
                            "Item \"{}\" is not specific enough to match a single item",
                            itemname
                        );
                    } else {
                        list.items.remove(idx);
                        modified = true;
                    }
                } else {
                    println!("List \"{}\" does not exist!", name);
                }
            } else {
                usage();
            }
        }

        "move" | "mv" | "m" => {
            if args.len() == 5 {
                if get_list_by_name(&lists, &args[4]).is_none() {
                    println!("List \"{}\" does not exist!", &args[4]);
                }
                let item = if let Some(list) = get_mut_list_by_name(&mut lists, &args[2]) {
                    let itemname = &args[3];
                    let idx = get_index_by_name(list, itemname);
                    if idx == usize::MAX {
                        println!("Item \"{}\" does not exist!", itemname);
                        None
                    } else if idx == usize::MAX - 1 {
                        println!(
                            "Item \"{}\" is not specific enough to match a single item",
                            itemname
                        );
                        None
                    } else {
                        Some(list.items.remove(idx))
                    }
                } else {
                    println!("List \"{}\" does not exist!", &args[2]);
                    None
                };

                if let Some(item) = item {
                    let l = get_mut_list_by_name(&mut lists, &args[4]).unwrap(); //already checked before
                    l.items.push(item);
                    modified = true;
                }
            } else {
                usage();
            }
        }

        "autorm" | "ar" => {
            if args.len() < 3 {
                usage();
                return;
            }
            let list_name = args[2..].join(" ");
            if let Some(list) = get_mut_list_by_name(&mut lists, &list_name) {
                list.items.retain(|item| match item {
                    ListEntry::Item(item) => !item.done,
                    _ => true,
                });
                modified = true;
            } else {
                println!("List \"{}\" does not exist!", &list_name);
            }
        }
        "today" | "t" | "week" | "w" | "overdue" | "od" => {
            if args.len() < 3 {
                usage();
                return;
            }
            use chrono::Duration;
            // find out the minimum and maximum allowed difference between the deadline date and today
            let (min_diff, max_diff, description) = match args[1].as_str() {
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
                (args[2..args.len() - 1].join(" "), true)
            } else {
                (args[2..].join(" "), false)
            };

            if let Some(list) = get_list_by_name(&lists, &list_name) {
                let now: DateTime<Local> = Local::now();
                let today = now.date_naive();
                let mut filter = |item: &&ListItem| {
                    !item.done && item.date - today < max_diff && item.date - today >= min_diff
                };
                if short {
                    let num = list.num_valid_entries(&lists, &mut filter);
                    if num == 0 {
                        // don't bother printing if there's none. maybe should make this configurable.
                        return;
                    }
                    println!(
                        "You have {} deadline{} {}",
                        num,
                        if num == 1 { "" } else { "s" },
                        description
                    );
                } else {
                    list.print_without_date(&lists, filter);
                }
            } else {
                println!("List \"{}\" does not exist!", &list_name);
            }
        }
        _ => {
            println!("Unrecognised command");
        }
    }
    if modified {
        save_yaml(list_file.as_path(), &lists).unwrap();
    }
}
