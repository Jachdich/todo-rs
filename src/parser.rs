use crate::{ListEntry, ListItem, TodoList};

// fn parse_one_list(s: &str) -> TodoList {
//     s.lines().map()
// }

#[derive(Debug)]
pub struct ParseError(pub String);

fn parse_text_item(line: &str, done: bool, line_num: usize) -> Result<ListEntry, ParseError> {
    let (date, rest_of_line) = if line.starts_with('@') {
        // parse the date
        let date_str = &line[1..11]; // TODO this might cause problems
        (
            Some(
                match chrono::NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
                    Ok(date) => date,
                    Err(_) => {
                        return Err(ParseError(format!(
                            "Invalid date literal (line {line_num})"
                        )))
                    }
                },
            ),
            &line[11..],
        )
    } else {
        (None, line)
    };
    Ok(ListEntry::Item(ListItem {
        name: rest_of_line.to_owned(),
        date,
        done,
        priority: 0,
        repeat_every: 0,
        repeat_next: 0,
    }))
}

fn parse_list_header(line: &str, line_num: usize) -> Result<TodoList, ParseError> {
    // Can probably remove this condition, because checked in the loop
    let first_char = line.chars().next();
    if first_char.is_some_and(char::is_whitespace) {
        return Err(ParseError(format!(
            "Unexpected indent, expected unindented list name (line {line_num})",
        )));
    }

    let item_name = line.trim_end();
    if !item_name.ends_with(':') {
        return Err(ParseError(format!(
            "Expected ':' at end of list definition (line {line_num})",
        )));
    }
    Ok(TodoList::new(item_name.trim_end_matches(':').to_owned()))
}

pub fn parse_str(s: &str) -> Result<Vec<TodoList>, ParseError> {
    let mut res: Vec<TodoList> = Vec::new();
    let lines = s.lines().enumerate();

    for (line_num, line) in lines {
        let line_num = line_num + 1;
        if line.trim().is_empty() {
            // skip empty lines
            continue;
        }
        if line.chars().next().is_some_and(char::is_whitespace) {
            let line = line.trim_start();
            let (init, rest) = line.split_at(1);
            let rest = rest.trim_start();

            let item = match init {
                "-" => parse_text_item(rest, false, line_num),
                "+" => parse_text_item(rest, true, line_num),
                "=" => Ok(ListEntry::List(rest.to_owned())),
                c => Err(ParseError(format!(
                        "Expected one of '-', '+' or '=' at the start of a list item, but instead found '{c}' (line {line_num})"
                    )))
            }?;
            // annoyingness to avoid using a match lol
            // basically returns an error if last_mut returns None
            res.last_mut().ok_or_else(|| ParseError(format!(
                    "Expected list header before item (line {line_num})"
                )))?
                .items
                .push(item);
        } else {
            res.push(parse_list_header(line, line_num + 1)?);
        }
    }
    Ok(res)
}

fn serialise_list(list: &TodoList) -> String {
    list.items
        .iter()
        .fold(list.name.clone() + ":\n", |mut acc, item| {
            acc += "\t";
            acc += &match item {
                ListEntry::List(name) => format!("= {name}"),
                ListEntry::Item(item) => format!(
                    "{} {}{}",
                    if item.done { "+" } else { "-" },
                    item.date.map_or_else(String::new, |date| format!("@{}", date.format("%d/%m/%Y"))),
                    &item.name
                ),
            };
            acc += "\n";
            acc
        })
}

pub fn emit_str(ls: &[TodoList]) -> String {
    ls.iter().fold(String::new(), |mut acc, list| {
        acc += &serialise_list(list);
        acc
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     fn test_parser() ->
// }
