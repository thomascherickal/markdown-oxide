use std::path::Path;

use itertools::Itertools;
use tower_lsp::lsp_types::{HoverParams, Hover, HoverContents, MarkupContent, MarkupKind};
use tower_lsp::jsonrpc::Result;

use crate::vault::{Vault, Reference, Referenceable};

pub fn hover(vault: &Vault, params: HoverParams, path: &Path) -> Option<Hover> {

    let cursor_position = params.text_document_position_params.position;

    let links = vault.select_references(Some(&path))?;
    let (refpath, reference) = links.iter().find(|&l| 
        l.1.data().range.start.line <= cursor_position.line && 
        l.1.data().range.end.line >= cursor_position.line && 
        l.1.data().range.start.character <= cursor_position.character &&
        l.1.data().range.end.character >= cursor_position.character
    )?;

    match reference {
        Reference::Link(_) => {
            let positions = vault.select_referenceable_nodes(None);
            let referenceable = positions.iter().find(|i| i.is_reference(&vault.root_dir(), &reference, &refpath))?;


            let range = referenceable.get_range();
            let links_text: String = (range.start.line..=range.end.line + 10)
                .map(|ln| vault.select_line(&referenceable.get_path(), ln as usize))
                .flatten() // flatten those options!
                .map(|vec| String::from_iter(vec))
                .join("");

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: match referenceable {
                        Referenceable::File(_, _) => format!("File Preview:\n---\n\n{}", links_text),
                        Referenceable::Heading(_, _) => format!("Heading Preview:\n---\n\n{}", links_text),
                        Referenceable::IndexedBlock(_, _) => format!("Block Preview:\n---\n\n{}", links_text),
                        _ => format!("Preview:\n---\n\n{}", links_text),
                    } 
                }),
                range: None
            })
        }
        _ => None
    }



}
