use crate::{ListEntry, ListItem, TodoList};

// fn parse_one_list(s: &str) -> TodoList {
//     s.lines().map()
// }

struct ParseError(String);

fn parse_str(s: &str) -> Result<Vec<TodoList>, ParseError> {
    let res: Vec<TodoList> = Vec::new();
    let mut lines = s.lines().enumerate();
    while let Some((line_num, mut item_name)) = lines.next() {
        if item_name.trim().is_empty() {
            // skip empty lines
            continue;
            if first_char.is_none() {
                continue;
            }
        }
        let first_char = item_name.chars().next().unwrap(); // unwrap OK because we just checked that the string wasn't empty
        let item_name = item_name.trim_end();
        if !item_name.ends_with(':') {
            return Err(ParseError(format!(
                "Expected ':' at end of list definition (line {})",
                line_num + 1
            )));
        }
    }
    Ok(res)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     fn test_parser() ->
// }
