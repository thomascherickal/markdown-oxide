# obsidian-markdown-ls
Obsidian Language Server: Obsidian-flavored-markdown language server 
Implementing obsidian PKM features (and possibly more) in the form of a language server allows us to use these features in our favorite text editors (neovim or vscode!) and reuse other lsp related plugins (like Telescope, outline, and builtin lsp support)

## Installation for Neovim (there is no VS code plugin yet)

Make sure rust is installed properly and that you are using nvim cmp (I am not sure if it works in other completion engines)

1. Clone the repo
2. `Cargo build --release`
3. Add and adjust the following to your Neovim config  

```
local configs = require("lspconfig.configs")
configs["obsidian_ls"] = {
default_config = {
  root_dir = function() return vim.fn.getcwd() end,
  filetypes = {"markdown"},
  cmd = {"{path}"} -- replace {path} with the path to the --release build. 
  -- {path} will be {where ever you cloned from}/obsidian-ls/target/release/obsidian-ls
},
on_attach = on_attach, -- do this only if you have an on_attach function already
capabilities = capabilities, -- add the nvim cmp capabilities if using it
}
require("lspconfig").obsidian_ls.setup({})
```

then adjust your nvim-cmp source settings for the following. Note that this will likely change in the future.

```
{
    name = 'nvim_lsp',
         option = {
             obsidian_ls = {
                 keyword_pattern = [[\(\k\| \|\/\|#\)\+]]
             }
         }
},
```


1. Test it out! Go to definitions, get references, and more!

## Features

- [ ] Go to definition (or definitions) from ...
    - [X] File references
    - [X] Heading references
    - [X] Indexed block references. (I call indexed blocks the blocks that you directly link to. The link will look like [[file#^index]]. When linking from the obsidian editor, an *index* ^index is appended to the paragraph/block you are referencing)
    - [X] Tags: This will get the locations where the tag is placed; it will give all the locations where the #tag/subtag is written. This is different than the functionality of the reference, which will get all tag and subtag usages: references on #tag will give #tag, #tag/subtag, #tag/sub/subtag ... 
    - [X] Footnotes
    - [ ] Metadata tag
- [ ] Get references
    - [X] To file
    - [X] to heading
    - [X] to indexed block
    - [X] to tag (explained above)
    - [X] Footnotes
    - [ ] Metadata tag
- [ ] Completions
    - [X] File completions (requires extra nvim cmp configuration)
    - [X] Heading Completions (requires extra nvim cmp config)
    - [X] Indexed block completions. Somehow using Ripgrep to find the paragraphs/blocks in the vault, then appending an index in the file, then inserting a link (workaround supported; using "_" instead of any non_word characters)
    - [X] Footnotes
    - [ ] Create file completions
    - [ ] Make file completions faster in NvimCmp (this is mostly a neovim issue as moxide generates very many completions for large vaults)
    - [ ] Callout completions
    - [ ] Metadata completions
    - [ ] Dataview?
    - [ ] Metadata tag
    - [ ] \`\`\`query\`\`\` code blocks
- [X] Preview
    - [X] File
    - [X] Headings
    - [X] Indexed Blocks
    - [X] Footnotes
- [ ] Code Actions
    - [x] Missing file for link -> Create the file
    - [ ] Link suggestions (by text match or other)
    - [ ] Refactoring: Move headers or selections to a new file
- [X] Diagnostics
    - [X] Missing reference
    - [ ] Reference count
- [X] Symbols
    - [X] File symbols: Headings and subheadings
    - [X] Workspace headings: everythign linkable: files, headings, tags, ... Like a good search feature
    - [ ] Lists and indented lists
- [ ] Rename
    - [ ] File
    - [ ] Headings
    - [ ] Tags.
- [ ] Dataview support
- [ ] Take some influence from LogSeq!!!!! https://docs.logseq.com/#/page/start%20here
    - [ ] Support Logseq syntax and completions/parsing for block references
    - [ ] Support Logseq embeds
    - [ ] Support Completions for logseq tasks
    - [ ] Support https://docs.logseq.com/#/page/markdown
    - [ ] Influence from logseq shortcut completions; such as to dates like /tomorrow



### Dev Todo's

- [X] Extract the RwLock read pattern
- [ ] Fix: updating diagnostics after code action

# Alternatives

**I love open source and all open source authors!! I also believe healthy competition is good! Moxide is competing with some alternatives, and I want to make it the best at its job!!**

Here are the alternatives (open source authors are welcom to make PRs adding their projects here!)

- https://github.com/gw31415/obsidian-lsp ; I have been in discussions with the author; he/she is a med student and doesn't have time to maintain . I of course love his idea, but the current LS doesn't provide many obsidian specific features yet. 
- https://github.com/WhiskeyJack96/logseqlsp ; This is a cool project and a great inspiration for logseq support (which is upcoming). status: it doesn't seem that it is maintained; no commites for 3 months
- The og https://github.com/artempyanykh/marksman ; I used this for a while, but it is not obsidian specific and didn't act well w my vault

# Workflows

- Linking to daily notes in the future to set todos; checking references to those notes using the language server

# Obsidian Graph Flexing (and maybe a little exigence for the project)

This is my vault. As you can see, there is a lot of stuff in it. I also like speed. Obsidian and other language servers are a little slow with my vault. I need speed so I made this project to cmp my needs.

![image](https://github.com/Feel-ix-343/moxide/assets/88951499/3de26de6-1113-469f-8807-40dd6c2e1e03)

![image](https://github.com/Feel-ix-343/moxide/assets/88951499/9204ebf9-f927-4f1e-8563-12e79099debd)



# ---The--bottom--line--------------------------------------------------------

Listen. I really like vim motions. I also really like low latency terminal editing. I very much so also like my neovim plugins and config. And wow I also like using obsidian (and other md apps). Can't I just have it all??? Can't I brute text edit in neovim and preview and fine edit in the JS madness? Well, I thought I could; that is why I am making this. (And hopefully why you will help me!)
