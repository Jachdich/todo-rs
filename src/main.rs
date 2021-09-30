use json;
use yaml_rust::{Yaml, YamlLoader, YamlEmitter};
use linked_hash_map::LinkedHashMap;
use std::io::Read;
use std::io::Write;

struct ListItem {
    name: String,
    date: i64,
    priority: i32,
}

enum ListEntry {
    Item(ListItem),
    List(String),
}

impl ListEntry {
    fn to_json(&self) -> json::JsonValue {
        match self {
            ListEntry::Item(item) => {
                json::object!{
                    type: "item",
                    name: item.name.to_owned(),
                    date: item.date,
                    priority: item.priority
                }
            }
            ListEntry::List(list) => {
                json::object!{
                    type: "list",
                    name: list.to_owned(),
                }
            }
        }
    }
    
    fn to_yaml(&self) -> Yaml {
        match self {
            ListEntry::Item(item) => {
                let mut map: LinkedHashMap<Yaml, Yaml> = LinkedHashMap::new();
                map.insert(Yaml::String("type".into()), Yaml::String("item".into()));
                map.insert(Yaml::String("name".into()), Yaml::String(item.name.to_owned()));
                map.insert(Yaml::String("priority".into()), Yaml::Integer(item.priority.into()));
                map.insert(Yaml::String("date".into()),     Yaml::Integer(item.date.into()));
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
}

struct TodoList {
    name: String,
    items: Vec<ListEntry>,
}

impl TodoList {
    fn new(name: String) -> Self {
        TodoList { name, items: Vec::new() }
    }

    fn from_json(val: &json::JsonValue) -> Self {
        let mut items: Vec<ListEntry> = Vec::new();
        for v in val["items"].members() {
            if v["type"] == "item" {
                items.push(ListEntry::Item(
                    ListItem { name: v["name"].to_string(),
                               date: v["date"].as_i64().unwrap(),
                               priority: v["priority"].as_i32().unwrap() }
                    ));
            }
        }
        TodoList { name: val["name"].to_string(), items }
    }

    fn to_json(&self) -> json::JsonValue {
        let mut out = json::array![];
        for item in &self.items {
            out.push(item.to_json());
        }
        json::object!{
            name: self.name.to_owned(),
            items: out
        }
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
        let name = val["name"].as_str().to_owned();
    }

    fn print(&self, all: &Vec<TodoList>) {
        for entry in &self.items {
            match entry {
                ListEntry::List(list_name) => {
                    get_list_by_name(all, list_name).unwrap().print(all);
                }

                ListEntry::Item(item) => {
                    println!("{}: {} {} {}", self.name, item.name, item.date, item.priority);
                }
            }
        }
    }
}

fn load(fname: &str) -> std::io::Result<Vec<TodoList>> {
    let mut file = std::fs::File::open(fname)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json_parsed = json::parse(&contents).unwrap();
    let mut lists: Vec<TodoList> = Vec::new();
    for v in json_parsed["lists"].members() {
        lists.push(TodoList::from_json(v));
    }
    Ok(lists)
}

fn save(fname: &str, lists: &Vec<TodoList>) -> std::io::Result<()> {
    let mut file = std::fs::File::create(fname)?;
    let mut listsjson = json::array![];
    for list in lists {
        listsjson.push(list.to_json());
    }
    let out = json::object!{
        lists: listsjson
    };

    file.write_all(&out.dump().into_bytes())?;
    Ok(())
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
    let mut emitter = YamlEmitter::new(&mut out);
    
    for list in lists {
        emitter.dump(&list.to_yaml());
    }

    file.write_all(&out.into_bytes())?;
    Ok(())
}

fn usage() {
    println!("Usage: idk");
}

fn get_list_by_name<'a>(lists: &'a Vec<TodoList>, name: &str) -> std::option::Option<&'a TodoList> {
    for i in lists {
        if i.name == name {
            return Some(i);
        }
    }
    None
}

fn get_mut_list_by_name<'a>(lists: &'a mut Vec<TodoList>, name: &str) -> std::option::Option<&'a mut TodoList> {
    for i in lists {
        if i.name == name {
            return Some(i);
        }
    }
    None
}


/*todo list all
todo add all "test item"
todo addlist all
todo new all*/
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        usage();
        return;
    }
    let mut lists = load_yaml("test.yml").unwrap();
    match args[1].as_str() {
        "list" => {
            if args.len() != 3 {
                usage();
                return;
            }
            if let Some(list) = get_list_by_name(&lists, &args[2]) {
                list.print(&lists);
            } else {
                println!("List does not exist!");
            }
        }
        "lists" => {

        }
        "new" => {
            if args.len() != 3 {
                usage();
                return;
            }
            lists.push(TodoList::new(args[2].to_owned()));
            println!("Created list {}", &args[2]);
        }
        "add" => {
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
                        ListItem { name: name.to_owned(), date, priority }
                    )
                );
            } else {
                println!("List does not exist!");
            }
        }

        "addlist" => {
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
                    println!("List \"{}\" added successfully", &args[3]);
                }
            } else {
                println!("List \"{}\" does not exist!", &args[2]);
            }
        }
        _ => {
            println!("Unrecognised command");
        }
    }
    save_yaml("test.yml", &lists);
}
