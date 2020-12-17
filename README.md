# Digital Garden

A content authoring tool with bidirectional links and other garden features

## Commands

### Write in your garden

```shell
## open $EDITOR and go through the process of writing interactively
garden write
## write to the sparkfile
garden write -sm something here
## append to a file or create a new one
garden write -t Some Title -m Some message
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

### Publish

Turn a directory of markdown files into html.

```shell
garden publish -o ./publish
```

## Options

All garden commands accept an environment variable or flag to indicate which garden to operate on.

```shell
## With an ENV var
GARDEN_PATH=~/github/chris/sector garden write
## or the same thing with a flag
garden -p ~/github/chris/sector garden write
```
