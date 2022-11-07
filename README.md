# DSV util

A small tool to run basic queries on DSV files.

1. [Commands](#commands)
2. [How to add a new command](#how-to-add-a-new-command)

## [Commands](#dsv-util)

### `"select" ("*" | (name ("," name)*))`

The `select` command keeps only the desired columns.

### `"where" name (("=" value) | ("like" pattern))`

The `where` command filters rows based on string equality or pattern matching of the value in the `name`d column.

### `"enum" name`

The `enum` command outputs two columns, respectively the number of appearances of a value and the value itself.  
This is similar in a way to running `uniq -c` in Bash.

### `"sort" "num"? name ("asc" | "desc")`

The `sort` command sorts rows based on the `name`d column.

### `"trim" ("*" | (name ("," name)*))`

The `trim` command removes heading and trailing whitespace from desired columns' values.

## [How to add a new command](#dsv-util)

Commands are organized in isolated modules as much as possible.  
**Steps**
1. Add the grammar rule to the grammar.pest file:
   - Create a new rule under the other commands' rules, right above the `command` rule.
   - Add that rule to the end of the `command` rule.
2. Add your command to the `Command` enum in `src/ast.rs`.
3. Handle matching from the rule to a new `Command::[your_rule]` in `src/ast.rs::build_command`.
4. Create your command's file in `src/operators/` and add the new module to `src/operators/mod.rs`.
5. Look at other commands to figure out how to write yours.
6. Add the match case for your rule in `src/ast.rs::Ast::run_on`.
