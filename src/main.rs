use yaml_rust::{Yaml, YamlLoader, YamlEmitter};
use linked_hash_map::LinkedHashMap;
use std::io::Read;
use std::io::Write;
use dirs;

struct ListItem {
    name: String,
    date: i64,
    priority: i32,
    done: bool,
    repeat_every: i64,
    repeat_next: i64,
}

enum ListEntry {
    Item(ListItem),
    List(String),
}

impl ListEntry {
    fn to_yaml(&self) -> Yaml {
        match self {
            ListEntry::Item(item) => {
                let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
                map.insert(Yaml::String("type".into()), Yaml::String("item".into()));
                map.insert(Yaml::String("name".into()), Yaml::String(item.name.to_owned()));
                map.insert(Yaml::String("done".into()),     Yaml::Boolean(item.done));
                if item.priority     != 0 { map.insert(Yaml::String("priority".into()),    Yaml::Integer(item.priority.into())); }
                if item.date         != 0 { map.insert(Yaml::String("date".into()),        Yaml::Integer(item.date.into())); }
                if item.repeat_every != 0 { map.insert(Yaml::String("repeat_every".into()),Yaml::Integer(item.repeat_every.into())); }
                if item.repeat_next  != 0 { map.insert(Yaml::String("repeat_next".into()), Yaml::Integer(item.repeat_next.into())); }
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
            "item" => {
                ListEntry::Item(ListItem {
                    name: y["name"].as_str().unwrap().to_owned(),
                    date: y["date"].as_i64().unwrap_or(0),
                    priority: y["priority"].as_i64().unwrap_or(0) as i32,
                    done: y["done"].as_bool().unwrap_or(false),
                    repeat_every: y["repeat_every"].as_i64().unwrap_or(0),
                    repeat_next: y["repeat_next"].as_i64().unwrap_or(0),
                })
            }

            "list" => {
                ListEntry::List(y["name"].as_str().unwrap().to_owned())
            }

            _ => panic!("Expected either 'item' or 'list', got '{}'", ty)
        }
    }
}

struct TodoList {
    name: String,
    items: Vec<ListEntry>,
}

impl TodoList {
    fn new(name: String) -> Self {
        TodoList { name, items: Vec::new() }
    }

    fn to_yaml(&self) -> Yaml {
        let mut out: Vec<Yaml> = Vec::new();
        for item in &self.items {
            out.push(item.to_yaml());
        }

        let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
        map.insert(Yaml::String("name".into()), Yaml::String(self.name.to_owned()));
        map.insert(Yaml::String("entries".into()), Yaml::Array(out));
        Yaml::Hash(map)
    }

    fn from_yaml(val: &Yaml) -> Self {
        let name = val["name"].as_str().unwrap().to_owned();
        let mut entries: Vec<ListEntry> = Vec::new();
        for y in val["entries"].as_vec().unwrap() {
            entries.push(ListEntry::from_yaml(&y));
        }
        Self { name, items: entries }
    }

    fn print(&self, all: &Vec<TodoList>, indent: usize, maxsize: usize) {
        println!("{}{}:", " ".repeat(indent * 4), self.name);
        let indent = indent + 1;
        let indentstr = " ".repeat(indent * 4 - 1);
        for entry in &self.items {
            match entry {
                ListEntry::List(list_name) => {
                    get_list_by_name(all, list_name).unwrap().print(all, indent, maxsize);
                }
                ListEntry::Item(item) => {
                    let tabs = " ".repeat(maxsize - indentstr.len() - item.name.len());
                    println!("{}{}{}{}\t{}\t{}", if item.done { "✓" } else { " " }, indentstr, item.name, tabs, item.date, item.priority);
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
                    max = std::cmp::max(max, get_list_by_name(all, list_name).unwrap().get_max_size(all, indent));
                }
                ListEntry::Item(item) => {
                    max = std::cmp::max(max, indent * 4 + item.name.len())
                }
            }
        }
        max
    }
}

fn load_yaml(fname: &str) -> std::io::Result<Vec<TodoList>> {
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

fn save_yaml(fname: &str, lists: &Vec<TodoList>) -> std::io::Result<()> {
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

fn usage() {
    println!("Usage:\ttodo <action> ...");
    println!("\tlists\t\t\t\tShow all the lists");
    println!("\tlist <list name>\t\tShow the items in the specified list");
    println!("\tnew <name>\t\t\tCreate a new list");
    println!("\tadd <list> <name> [date, [priority]]\tAdd a new item to the specified list");
    println!("\taddlist <dest> <src>\t\tAdd a reference of list <src> to list <dest>");
    println!("\tdone <list> <item>\t\tMark the specified item as done");
    println!("\trm <list> <item>\t\tRemove <item> from <list>");
    println!("\tmv <list> <item> <list>\t\tMove an <item> from <list> to another <list>");
    println!("\trepeat <list> <item> <time>\tSet an item to repeat (mark as un-done) every <time>");
    println!("\tautorm <list>\t\t\tRemove all items in <list> that are marked as done");
}

fn get_list_by_name<'a>(lists: &'a Vec<TodoList>, name: &str) -> std::option::Option<&'a TodoList> {
    let mut item: std::option::Option<&'a TodoList> = None;
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

fn get_mut_list_by_name<'a>(lists: &'a mut Vec<TodoList>, name: &str) -> std::option::Option<&'a mut TodoList> {
    let mut item: std::option::Option<&'a mut TodoList> = None;
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

fn get_index_by_item(list: &TodoList, itemname: &str) -> usize {
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


/*todo list all
todo add all "test item"
todo addlist all
todo new all*/
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
        return;
    }
    let mut config_dir = dirs::config_dir();
    let mut lists = load_yaml("test.yml").unwrap();
    match args[1].as_str() {
        "list" | "l" => {
            if args.len() != 3 {
                usage();
                return;
            }
            if let Some(list) = get_list_by_name(&lists, &args[2]) {
                let max = list.get_max_size(&lists, 0);
                list.print(&lists, 0, max);
            } else {
                println!("List does not exist!");
            }
        }
        "lists" | "ls" => {
            for i in &lists {
                println!("{}", i.name);
            }
        }
        "new" | "n" => {
            if args.len() != 3 {
                usage();
                return;
            }
            lists.push(TodoList::new(args[2].to_owned()));
        }
        "add" | "a" => {
            if args.len() <= 3 { 
                usage();
                return;
            }
            if let Some(list) = get_mut_list_by_name(&mut lists, &args[2]) {
                let name = &args[3];
                let date = if args.len() >= 5 {
                    args[4].parse::<i64>().unwrap()
                } else {
                    0
                };

                let priority = if args.len() >= 6 {
                    args[5].parse::<i32>().unwrap()
                } else {
                    0
                };
    
                list.items.push(
                    ListEntry::Item(
                        ListItem { name: name.to_owned(), date, priority, done: false, repeat_every: 0, repeat_next: 0 }
                    )
                );
            } else {
                println!("List does not exist!");
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
                    let idx = get_index_by_item(list, itemname);
                    if idx == usize::MAX {
                        println!("Item \"{}\" does not exist!", itemname);
                    } else if idx == usize::MAX - 1 {
                        println!("Item \"{}\" is not specific enough to match a single item", itemname);
                    } else {
                        if let ListEntry::Item(i) = &mut list.items[idx] {
                            i.done = !i.done;
                        } else {
                            println!("You can't done a list silly (todo add this feature cos its cool)");
                        }
                    }
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
                    let idx = get_index_by_item(list, itemname);
                    if idx == usize::MAX {
                        println!("Item \"{}\" does not exist!", itemname);
                    } else if idx == usize::MAX - 1 {
                        println!("Item \"{}\" is not specific enough to match a single item", itemname);
                    } else {
                        list.items.remove(idx);
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
                if let Some(list) = get_list_by_name(&lists, &args[4]) {
                    //what
                } else {
                    println!("List \"{}\" does not exist!", &args[4]);
                }
                let item = if let Some(list) = get_mut_list_by_name(&mut lists, &args[2]) {
                    let itemname = &args[3];
                    let idx = get_index_by_item(list, itemname);
                    if idx == usize::MAX {
                        println!("Item \"{}\" does not exist!", itemname);
                        None
                    } else if idx == usize::MAX - 1 {
                        println!("Item \"{}\" is not specific enough to match a single item", itemname);
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
                }
            } else {
                usage();
            }
        }
        _ => {
            println!("Unrecognised command");
        }
    }
    save_yaml("test.yml", &lists).unwrap();
}
