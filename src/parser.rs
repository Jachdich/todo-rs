use crate::{ListEntry, ListItem, TodoList};

// fn parse_one_list(s: &str) -> TodoList {
//     s.lines().map()
// }

struct ParseError(String);

fn parse_str(s: &str) -> Result<Vec<TodoList>, ParseError> {
    let res: Vec<TodoList> = Vec::new();
    let mut lines = s.lines().enumerate();
    while let Some((line_num, mut item_name)) = lines.next() {
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
