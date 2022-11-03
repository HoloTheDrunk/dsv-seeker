# DSV util

A small tool to run basic queries on DSV files.

## Commands

### `select (* | (name (, name)*))`

The `select` command keeps only the desired columns.

### `where name ((= value) | (like pattern))`

The `where` command filters rows based on string equality or pattern matching of the value in the `name`d column.

### `enum name`

The `enum` command outputs two columns, respectively the number of appearances of a value and the value itself.  
This is similar in a way to running `uniq -c` in Bash.

### `sort name`

The `sort` command sorts rows based on the `name`d column.
