# todo-rs

Simple yet powerful terminal todo list program. The key feature is the ability to nest lists, creating a tree-like structure.

Commands:
```
Usage:	todo <action> ...
	lists				Show all the lists
	list <list name>		Show the items in the specified list
	new <name>			Create a new list
	add <list> <name> [date, [priority]]	Add a new item to the specified list
	addlist <dest> <src>		Add a reference of list <src> to list <dest>
	done <list> <item>		Mark the specified item as done
	rm <list> <item>		Remove <item> from <list>
	mv <list> <item> <list>		Move an <item> from <list> to another <list>
	repeat <list> <item> <time>	Set an item to repeat (mark as un-done) every <time>
	autorm <list>			Remove all items in <list> that are marked as done
```

# Demo

![gh_todo_demo](https://user-images.githubusercontent.com/42205980/199619052-2e45f75a-dfd7-49d3-89ed-0dc8012916b1.png)
