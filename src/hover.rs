use std::path::Path;

use tower_lsp::lsp_types::{HoverParams, Hover, HoverContents};

use crate::{ui::preview_reference, vault::Vault};

pub fn hover(vault: &Vault, params: HoverParams, path: &Path) -> Option<Hover> {

    let cursor_position = params.text_document_position_params.position;

    let links = vault.select_references(Some(&path))?;
    let (refpath, reference) = links.iter().find(|&l| 
        l.1.data().range.start.line <= cursor_position.line && 
        l.1.data().range.end.line >= cursor_position.line && 
        l.1.data().range.start.character <= cursor_position.character &&
        l.1.data().range.end.character >= cursor_position.character
    )?;

    return preview_reference(vault, refpath, reference).and_then(|markup| Some(Hover { contents: HoverContents::Markup(markup), range: None  }))
}