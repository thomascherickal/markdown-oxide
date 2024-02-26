<meta name="google-site-verification" content="YWllrHxS71HepBAFJqguFgFjDXHZ7rAIeSUzJTPW91o" />

# Markdown Oxide


Implementing obsidian PKM features (and possibly more) in the form of a language server allows us to use these features in our favorite text editors and reuse other lsp related plugins (like Telescope, outline, refactoring tools, ...)

## Usage

First, compile the plugin. Clone the repo and then run `cargo build --release`

Next, follow the directions for your editor

### VSCode

Go to [the vscode extension readme](./vscode-extension/README.md) and run the commands listed

### Neovim

Make sure rust is installed properly and that you are using nvim cmp (I am not sure if it works in other completion engines)

Adjust your neovim config as follows

```
local configs = require("lspconfig.configs")
configs["obsidian_ls"] = {
default_config = {
  root_dir = function() return vim.fn.getcwd() end,
  filetypes = {"markdown"},
  cmd = {"{path}"} -- replace {path} with the path to the --release build. 
  -- {path} will be {where ever you cloned from}/obsidian-ls/target/release/markdown-oxide
},
on_attach = on_attach, -- do this only if you have an on_attach function already
}
require("lspconfig").obsidian_ls.setup({
    capabilities = capabilities -- ensure that capabilities.workspace.didChangeWatchedFiles.dynamicRegistration = true
})
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


I also recommend enabling codelens in neovim. Add this snippet to your on\_attach function for nvim-lspconfig


```
-- refresh codelens on TextChanged and InsertLeave as well
vim.api.nvim_create_autocmd({ 'TextChanged', 'InsertLeave', 'CursorHold', 'LspAttach' }, {
    buffer = bufnr,
    callback = vim.lsp.codelens.refresh,
})
-- trigger codelens refresh
vim.api.nvim_exec_autocmds('User', { pattern = 'LspAttached' })
```


1. Test it out! Go to definitions, get references, and more!

NOTE: To get references on files, you must place your cursor/pointer on the first character of the first line of the file, and then get references. (In VSCode, you can also use the references code lens)

## Features

- Go to definition (or definitions) from ...
    - [X] File references [[file]]
    - [X] Heading references [[file#heading]]
    - [X] Block references. [[file#^index]] (I call indexed blocks the blocks that you directly link to. The link will look like [[file#^index]]. When linking from the obsidian editor, an *index* ^index is appended to the paragraph/block you are referencing)
    - [X] Tags #tag and #tag/subtag/..
    - [X] Footnotes: "paraphrased text[^footnoteindex]"
    - [ ] Metadata tag
- Get references
    - [X] For File when cursor is on the first character of the first line of the file. This will produce references not only to the file but also to headings and blocks in the file
    - [X] For block when the cursor is on the blocks index "...text *^index*"
    - [X] For tag when the cursor is on the tags declaration. Unlike go to definition for tags, this will produce all references to the tag and to the tag with subtags
    - [X] Footnotes when the cursor is on the declaration line of the footnote; *[^1]: description...*
- Completions (requires extra nvim cmp config; follow the directions above)
    - [X] File link completions
    - [X] Heading link Completions
    - [X] Block link completions (searches the text of the block) 
    - [X] Footnote link completions
    - [X] New Block link Completions through grep: to use this, type `[[ `, and after you press space, completions for every block in the vault will appear; continue typing to fuzzy match the block that you want; finally, select the block; a link will be inserter to the text document and an index (ex ^1j239) will be appended to the block in its respective file
    - [ ] Callout/admonition completions
    - [ ] Metadata completions
    - [ ] Dataview completions
    - [ ] Metadata tag completions
    - [ ] \`\`\`query\`\`\` code block completions
- Hover Preview
    - [X] File
    - [X] Headings
    - [X] Indexed Blocks
    - [X] Footnotes
- [ ] Code Actions
    - [x] Unresolved file link -> Create the file
    - [x] Unresolved heading link -> append heading to file and create file
    - [ ] Link suggestions (by text match or other)
    - [ ] Refactoring: Move headers or selections to a new file
    - [ ] Link an unlinked reference
    - [ ] Link all unlinked references to a referenceable
- [X] Diagnostics
    - [X] Unresolved reference
    - [ ] Unlinked reference
- [X] Symbols
    - [X] File symbols: Headings and subheadings
    - [X] Workspace headings: everythign linkable: files, headings, tags, ... Like a good search feature
    - [ ] Lists and indented lists
- [ ] Rename
    - [X] File (cursor must be in the first character of the first line)
    - [X] Headings
    - [X] Tags
    - [ ] Indexed Blocks
- [ ] Dataview support
- [ ] Take some influence from LogSeq!!!!! https://docs.logseq.com/#/page/start%20here
    - [ ] Support Logseq syntax and completions/parsing for block references
    - [ ] Support Logseq embeds
    - [ ] Support Completions for logseq tasks
    - [ ] Support https://docs.logseq.com/#/page/markdown
    - [ ] Influence from logseq shortcut completions; such as to dates like /tomorrow

# Alternatives

**I love open source and all open source authors!! I also believe healthy competition is good! Moxide is competing with some alternatives, and I want to make it the best at its job!!**

Here are the alternatives (open source authors are welcom to make PRs adding their projects here!)

- https://github.com/gw31415/obsidian-lsp ; I have been in discussions with the author; he/she is a med student and doesn't have time to maintain . I of course love his idea, but the current LS doesn't provide many obsidian specific features yet. 
- https://github.com/WhiskeyJack96/logseqlsp ; This is a cool project and a great inspiration for logseq support (which is upcoming). status: it doesn't seem that it is maintained; no commites for 3 months
- The og https://github.com/artempyanykh/marksman ; I used this for a while, but it is not obsidian specific and didn't act well w my vault

# Workflows

- Linking to daily notes in the future to set todos; checking references to those notes using the language server

# Obsidian Graph Flexing (and maybe a little exigence for the project)

This is my vault. As you can see, there is a lot of stuff in it. Along with notes, I also like speed. Obsidian, logseq, and other language servers are a little slow with my vault (and a little high latency in general). I need speed; Neovim and this LSP CMPs my needs! (I am using this daily, right now even). 

![image](https://github.com/Feel-ix-343/moxide/assets/88951499/3de26de6-1113-469f-8807-40dd6c2e1e03)

![image](https://github.com/Feel-ix-343/moxide/assets/88951499/9204ebf9-f927-4f1e-8563-12e79099debd)



# ---The--bottom--line--------------------------------------------------------

Listen. I really like vim motions. I also really like low latency terminal editing. I very much so also like my neovim plugins and config. And wow I also like using obsidian (and other md apps). Can't I just have it all??? Can't I brute text edit in neovim and preview and fine edit in the JS madness? Well, I thought I could; that is why I am making this. (And hopefully why you might help me!)
