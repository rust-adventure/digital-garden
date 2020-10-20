# Digital Garden

A content authoring tool with bidirectional links and other garden features

## Commands

### Initialize a new garden

```shell
## Initialize the default garden
garden init
## Initialize a garden at $PATH
garden init ~/github/chris/sector
```

### Write in your garden

```shell
## open $EDITOR and go through the process of writing interactively
garden write
## write to the sparkfile
garden write -sm something here
## append to a file or create a new one
garden write -t Some Title -m Some message
```

### Sync garden with remote source

This is a facade over git for our purposes, meaning that we don't have to handle the edge cases because we expect the user to drop down to git to resolve it. Yes this is "cheating", no merging text documents across multiple concurrent users is not part of this course.

It could sync to a remote store in the future.

```shell
garden sync
```

### Search

Garden expects MDX, although we won't be parsing it fully in this course. This leads to two things: bidirectional links and tags. We can search for files by the tag they include.

```shell
garden search -t rust
```

#### Bidirectional Links

```markdown
[[learning rust]]
```

#### Tags

```md
# Metadata

Talking about #rust and other things
```

## Options

All garden commands accept an environment variable or flag to indicate which garden to operate on.

```shell
## With an ENV var
GARDEN_PATH=~/github/chris/sector garden sync
## or the same thing with a flag
garden -p ~/github/chris/sector garden sync
```

There is a config file that is interacted with through the CLI and placed in a location on disk.

```shell
garden config edit
garden config path
```
