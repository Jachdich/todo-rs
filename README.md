# todo-rs

Simple yet powerful terminal todo list program. The key feature is the ability to nest lists, creating a tree-like structure.

Commands:
```
Usage:	todo <action> ...
	ls  lists                        Show all the lists
	l   list <list name> [--small]   Show the items in the specified list.
	n   new <name>                   Create a new list
	rl  rmlist <list>                Delete the specified list
	a   add <list> <name> [date]     Add a new item to the specified list
	al  addlist <dest> <src>         Add a reference of list <src> to list <dest>
	d   done <list> <item>           Mark the specified item as done
	da  doneall <list>               Mark all items in list as done
	uda undoneall <list>             Mark all items in list as not done
	rm  remove <list> <item>         Remove <item> from <list>
	mv  move <source> <item> <dest>  Move an <item> from the list <source> to <dest>
	mva moveall <source> <dest>      Move every item from <source> into <dest>. Does not move sublist of source into itself
	rn  rename <list> <old> <new>    Rename an item in <list> from <old> to <new>
	rl  renamelist <old> <new>       Rename the list <old> to <new>
	ar  autorm <list>                Remove all items in <list> that are marked as done
	t   today <list> [--short]       List all tasks with a deadline of today.
                                         If --short is passed, return only the number of tasks, do not list them.
	w   week <list> [--short]        List all tasks with a deadline of within the next 7 days
	od  overdue <list> [--short]     List all non-completed tasks with a deadline in the past```
```

# Demo

![gh_todo_demo](https://user-images.githubusercontent.com/42205980/199619052-2e45f75a-dfd7-49d3-89ed-0dc8012916b1.png)

# TODO

A somewhat ironic section to have, but there are multiple issues. In fact, here's a screenshot of my todo list for my todo list on my todo list (if you get my drift)

![image](https://user-images.githubusercontent.com/42205980/199619349-6b686469-54d7-4574-8fc7-53593d2436e2.png)

There are many bugs and it's kind of unusable right now but I will finish it someday...
