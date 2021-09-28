use json;
use std::io::Read;
use std::io::Write;

struct ListItem {
    name: String,
    date: u64,
    priority: i32,
}

enum ListEntry<'a> {
    Item(ListItem),
    List(&'a TodoList<'a>),
}

impl<'a> ListEntry<'a> {
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
                    name: list.name.to_owned(),
                }
            }
        }
    }
}

struct TodoList<'a> {
    name: String,
    items: Vec<ListEntry<'a>>,   
}

impl<'a> TodoList<'a> {
    fn new(name: String) -> Self {
        TodoList { name, items: Vec::new() }
    }

    fn from_json(val: &json::JsonValue) -> Self {
        let mut items: Vec<ListEntry> = Vec::new();
        for v in val["items"].members() {
            if v["type"] == "item" {
                items.push(ListEntry::Item(
                    ListItem { name: v["name"].to_string(),
                               date: v["date"].as_u64().unwrap(),
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

    fn print(&self) {
        for entry in &self.items {
            match entry {
                ListEntry::List(list) => {
                    list.print();
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

fn usage() {
    println!("Usage: idk");
}

fn get_list_by_name<'a>(lists: &'a Vec<TodoList>, name: &str) -> std::option::Option<&'a TodoList<'a>> {
    for i in lists {
        if i.name == name {
            return Some(i);
        }
    }
    None
}

fn get_mut_list_by_name<'a, 'b>(lists: &'b mut Vec<TodoList<'a>>, name: &str) -> std::option::Option<&'b mut TodoList<'a>> {
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
    let mut lists = load("test.json").unwrap();
    match args[1].as_str() {
        "list" => {
            if args.len() != 3 {
                usage();
                return;
            }
            if let Some(list) = get_list_by_name(&lists, &args[2]) {
                list.print();
            } else {
                println!("List does not exist!");
            }
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
                    args[4].parse::<u64>().unwrap()
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
        _ => {
            println!("Unrecognised command");
        }
    }
    save("test.json", &lists);
}
