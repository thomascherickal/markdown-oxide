use std::{path::{Path, PathBuf}, time::SystemTime};

use chrono::{Duration, TimeDelta};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionTextEdit, Documentation, InsertTextFormat, Position, Range, TextEdit};

use crate::{config::Settings, ui::preview_referenceable, vault::{MDFile, MDHeading, Reference, Referenceable, Vault}};

use super::{matcher::{fuzzy_match_completions, Matchable, OrderedCompletion}, Completable, Completer, Context};

/// Range on a single line; assumes that the line number is known. 
type LineRange = std::ops::Range<usize>;

pub struct MarkdownLinkCompleter<'a> {
    /// The display text of a link to be completed
    pub display: (String, LineRange),
    /// the filepath of the markdown link to be completed
    pub path: (String, LineRange),
    /// the infile ref; the range is the whole span of the infile ref. (including the ^ for Block refs)
    pub infile_ref: Option<(PartialInfileRef, LineRange)>,

    pub partial_link: (String, LineRange),
    pub full_range: LineRange,
    pub line_nr: usize,
    pub position: Position,
    pub file_path: std::path::PathBuf,
    pub vault: &'a Vault,
    pub context_path: &'a Path,
    pub settings: &'a Settings
}

pub trait LinkCompleter<'a> : Completer<'a> {
    fn settings(&self) -> &'a Settings;
    fn completion_text_edit(&self, display: Option<&str>, refname: &str) -> CompletionTextEdit;
    fn entered_refname(&self) -> String;
    fn vault(&self) -> &'a Vault;
    fn position(&self) -> Position;
    fn path(&self) -> &'a Path;
    fn link_completions(&self) -> Vec<LinkCompletion<'a>>  where Self : Sync {

        let referenceables = self.vault().select_referenceable_nodes(None);

        let position = self.position();

        let unresolved_under_cursor = self.vault().select_reference_at_position(self.path(), position)
            .map(|reference| self.vault().select_referenceables_for_reference(reference, self.path()))
            .into_iter()
            .flatten()
            .find(|referenceable| referenceable.is_unresolved());

        let single_unresolved_under_cursor = unresolved_under_cursor.and_then(|referenceable| {
            let ref_count = self.vault().select_references_for_referenceable(&referenceable)?.len();

            if ref_count == 1 {
                Some(referenceable)
            } else {
                None
            }
        });

        // Get and filter referenceables
        let completions = referenceables
            .into_par_iter()
            .filter(|referenceable| { 
                Some(referenceable) != single_unresolved_under_cursor.as_ref()
            })
            .flat_map(|referenceable| LinkCompletion::new(referenceable.clone(), self))
            .collect::<Vec<_>>();

        completions
    }
}

impl<'a> LinkCompleter<'a> for MarkdownLinkCompleter<'a> {

    fn settings(&self) -> &'a Settings {
        self.settings
    }

    fn path(&self) -> &'a Path {
        self.context_path
    }
    fn position(&self) -> Position {
        self.position
    }

    fn vault(&self) -> &'a Vault {
        self.vault
    }

    fn entered_refname(&self) -> String {
        format!("{}{}", self.path.0, self.infile_ref.as_ref().map(|infile| infile.0.to_string()).unwrap_or("".to_string()))
    }

    /// Will add <$1> to the refname if it contains spaces
    fn completion_text_edit(&self, display: Option<&str>, refname: &str) -> CompletionTextEdit {


        let link_ref_text = match refname.contains(' ') {
            true => format!("<{}>", refname),
            false => refname.to_owned()
        };

        CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: Position {
                    line: self.line_nr as u32,
                    character: self.full_range.start as u32,
                },
                end: Position {
                    line: self.line_nr as u32,
                    character: self.full_range.end as u32,
                },
            },
            new_text: format!("[{}]({})", display.unwrap_or(""), link_ref_text)
        })
    }
}

impl<'a> Completer<'a> for MarkdownLinkCompleter<'a> {

    fn construct(context: Context<'a>, line: usize, character: usize) -> Option<Self>
    where Self: Sized {

        let Context { vault, opened_files: _, path, .. } = context;

        let line_chars = vault.select_line(path, line as isize)?;
        let line_to_cursor = line_chars.get(0..character)?;

        static PARTIAL_MDLINK_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\[(?<display>[^\[\]\(\)]*)\]\((?<path>[^\[\]\(\)\#]*)(\#(?<infileref>[^\[\]\(\)]*))?$").unwrap()
        }); // [display](relativePath)

        let line_string_to_cursor = String::from_iter(line_to_cursor);

        let captures = PARTIAL_MDLINK_REGEX.captures(&line_string_to_cursor)?;

        let (full, display, reftext, infileref) = (
            captures.get(0)?,
            captures.name("display")?,
            captures.name("path")?,
            captures.name("infileref"),
        );

        let line_string = String::from_iter(&line_chars);

        let reference_under_cursor =
        Reference::new(&line_string)
            .into_iter()
            .find(|reference| {
                reference.range.start.character <= character as u32
                && reference.range.end.character >= character as u32
            });

        let full_range = match reference_under_cursor {
            Some( reference @ (Reference::MDFileLink(..)
                | Reference::MDHeadingLink(..)
                | Reference::MDIndexedBlockLink(..)),
            ) => reference.range.start.character as usize..reference.range.end.character as usize,
            None if line_chars.get(character) == Some(&')') => {
                full.range().start..full.range().end + 1
            }
            _ => full.range(),
        };


        let partial_infileref = infileref.map(|infileref| {

            let chars = infileref.as_str().chars().collect::<Vec<char>>();

            let range = infileref.range();

            match chars.as_slice() {
                ['^', rest @ ..] => (PartialInfileRef::BlockRef(String::from_iter(rest)), range),
                [rest @ ..] => (PartialInfileRef::HeadingRef(String::from_iter(rest)), range),
            }

        });

        let partial = Some(MarkdownLinkCompleter {
            path: (reftext.as_str().to_string(), reftext.range()),
            display: (display.as_str().to_string(), display.range()),
            infile_ref: partial_infileref,
            partial_link: (full.as_str().to_string(), full.range()),
            full_range,
            line_nr: line,
            position: Position {
                line: line as u32,
                character: character as u32,
            },
            file_path: path.to_path_buf(),
            vault,
            context_path: context.path,
            settings: context.settings
        });

        partial
    }

    fn completions(&self) -> Vec<impl Completable<'a, MarkdownLinkCompleter<'a>>> {

        let filter_text = format!(
            "{}{}",
            self.path.0,
            self.infile_ref
                .clone()
                .map(|(infile, _)| format!("#{}", infile.completion_string()))
                .unwrap_or("".to_string())
        );

        let link_completions = self.link_completions();

        let matches = fuzzy_match_completions(&filter_text, link_completions);

        matches


    }

    /// The completions refname
    type FilterParams = &'a str;

    fn completion_filter_text(&self, params: Self::FilterParams) -> String {
        let filter_text = format!(
            "[{}]({}",
            self.display.0,
            params
        );

        filter_text

    }
}

#[derive(Debug, Clone)]
pub enum PartialInfileRef {
    HeadingRef(String),
    /// The partial reference to a block, not including the ^ index
    BlockRef(String)
}


impl ToString for PartialInfileRef {
    fn to_string(&self) -> String {
        match self {
            Self::HeadingRef(string) => string.to_owned(),
            Self::BlockRef(string) => format!("^{}", string)
        }
    }
}

impl PartialInfileRef {
    fn completion_string(&self) -> String {
        match self {
            PartialInfileRef::HeadingRef(s) => s.to_string(),
            PartialInfileRef::BlockRef(s) => format!("^{}", s),
        }
    }
}

pub struct WikiLinkCompleter<'a> {
    vault: &'a Vault,
    cmp_text: Vec<char>,
    files: &'a [PathBuf],
    index: u32,
    character: u32,
    line: u32,
    context_path: &'a Path,
    settings: &'a Settings
}

impl<'a> LinkCompleter<'a> for WikiLinkCompleter<'a> {

    fn settings(&self) -> &'a Settings {
        self.settings
    }

    fn path(&self) -> &'a Path {
        self.context_path
    }

    fn position(&self) -> Position {
        Position {
            line: self.line,
            character: self.character
        }
    }

    fn vault(&self) -> &'a Vault {
        self.vault
    }

    fn entered_refname(&self) -> String {
        String::from_iter(&self.cmp_text)
    }

    fn completion_text_edit(&self, display: Option<&str>, refname: &str) -> CompletionTextEdit {
        
        let text_edit = CompletionTextEdit::Edit(TextEdit {
            range: Range {
                start: Position {
                    line: self.line as u32,
                    character: self.index + 1 as u32, // index is right at the '[' in [[link]]; we want one more than that
                },
                end: Position {
                    line: self.line as u32,
                    character: self.character as u32,
                },
            },
            new_text: format!("{}{}", refname, display.map(|display| format!("|{}", display)).unwrap_or("".to_string()))
        });

        text_edit
    }
}

impl<'a> Completer<'a> for WikiLinkCompleter<'a> {


    fn construct(context: Context<'a>, line: usize, character: usize) -> Option<Self>
        where Self: Sized {

        let Context { vault, opened_files, path, .. } = context;

        let line_chars = vault.select_line(path, line as isize)?;

        let index = line_chars.get(0..=character)? // select only the characters up to the cursor
            .iter()
            .enumerate() // attach indexes
            .tuple_windows() // window into pairs of characters
            .collect::<Vec<(_, _)>>()
            .into_iter()
            .rev() // search from the cursor back
            .find(|((_, &c1), (_, &c2))| c1 == '[' && c2 == '[')
            .map(|(_, (i, _))| i); // only take the index; using map because find returns an option

        let index = index.and_then(|index| {
            if line_chars.get(index..character)?.into_iter().contains(&']') {
                None
            } else {
                Some(index)
            }
        });

        index.and_then(|index| {
            let cmp_text = line_chars.get(index+1..character)?;

            Some(WikiLinkCompleter{
                vault,
                cmp_text: cmp_text.to_vec(),
                files: opened_files,
                index: index as u32,
                character: character as u32,
                line: line as u32,
                context_path: context.path,
                settings: context.settings
            })
        })
    }

    fn completions(&self) -> Vec<impl Completable<'a, Self>> where Self: Sized {
        let WikiLinkCompleter { vault, cmp_text: _, files, index: _, character: _, line: _, context_path: _, .. } = self;

        match *self.cmp_text {
            // Give recent referenceables; TODO: improve this; 
            [] => {
                files
                    .iter()
                    .map(|path| {
                        match std::fs::metadata(path).and_then(|meta| meta.modified()) {
                            Ok(modified) => (path, modified),
                            Err(_) => (path, SystemTime::UNIX_EPOCH),
                        }
                    })
                    .sorted_by_key(|(_, modified)| *modified)
                    .flat_map(|(path, modified)| {

                        let referenceables = vault.select_referenceable_nodes(Some(&path));

                        let modified_string = modified.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs().to_string();

                        Some(referenceables.into_iter()
                            .flat_map(move |referenceable| Some(
                                OrderedCompletion::<WikiLinkCompleter, LinkCompletion>::new(
                                    LinkCompletion::new(referenceable, self)?,
                                    modified_string.clone()
                                ))
                            ))

                    })
                    .flatten()
                    .collect_vec()
            },
            ref filter_text @ [..] if !filter_text.contains(&']') => {
                let filter_text = &self.cmp_text;


                let link_completions = self.link_completions();

                let matches = fuzzy_match_completions(&String::from_iter(filter_text), link_completions);

                matches
            },
            _ => vec![]
        }
    }

    type FilterParams = &'a str;
    fn completion_filter_text(&self, params: Self::FilterParams) -> String {
        params.to_string()
    }
}







#[derive(Debug, Clone)]
pub enum LinkCompletion<'a> {
    File {
        mdfile: &'a MDFile,
        match_string: String,
        referenceable: Referenceable<'a>
    },
    Heading {
        heading: &'a MDHeading,
        match_string: String,
        referenceable: Referenceable<'a>
    },
    Block {
        match_string: String,
        referenceable: Referenceable<'a>
    },
    Unresolved {
        match_string: String,
        /// Infile ref includes all after #, including ^
        infile_ref: Option<String>,
        referenceable: Referenceable<'a>
    },
    DailyNote(MDDailyNote<'a>)
}

use LinkCompletion::*;

impl LinkCompletion<'_> {
    fn new<'a>(referenceable: Referenceable<'a>, completer: &impl LinkCompleter<'a>) -> Option<LinkCompletion<'a>> {
        if let Some(daily) = MDDailyNote::new(referenceable.clone(), completer) {
            Some(DailyNote(daily))
        } else {


            match referenceable {
                Referenceable::File(_, mdfile) => Some(File { mdfile, match_string: mdfile.path.file_stem()?.to_str()?.to_string(), referenceable }),
                Referenceable::Heading(path, mdheading) => Some(Heading {heading: mdheading, match_string: format!("{}#{}", path.file_stem()?.to_str()?, mdheading.heading_text), referenceable}),
                Referenceable::IndexedBlock(path, indexed) => Some(Block{ match_string: format!("{}#^{}", path.file_stem()?.to_str()?, indexed.index), referenceable}),
                Referenceable::UnresovledFile(_, file) => Some(Unresolved { match_string: file.clone(), infile_ref: None, referenceable }),
                Referenceable::UnresolvedHeading(_, s1, s2) => Some(Unresolved { match_string: format!("{}#{}", s1, s2), infile_ref: Some(s2.clone()), referenceable }),
                Referenceable::UnresovledIndexedBlock(_, s1, s2) => Some(Unresolved { match_string: format!("{}#^{}", s1, s2), infile_ref: Some(format!("^{}", s2)), referenceable }),
                _ => None
            }
        }
    }

    fn default_completion(&self, refname: &str, text_edit: CompletionTextEdit, filter_text: &str, vault: &Vault) -> CompletionItem {

        let referenceable = match self {
            Self::File { referenceable,.. }
            | Self::Heading { referenceable, .. }
            | Self::Block { referenceable, .. }
            | Self::Unresolved { referenceable, .. }
            | Self::DailyNote(MDDailyNote { referenceable, .. })=> referenceable
        };

        CompletionItem {
            label: refname.to_string(),
            kind: Some(match self {
                Self::File { mdfile: _, match_string: _, .. } => CompletionItemKind::FILE,
                Self::Heading { heading: _, match_string: _, .. } | Self::Block { match_string: _, .. } => CompletionItemKind::REFERENCE,
                Self::Unresolved { match_string: _, infile_ref: _, .. } => CompletionItemKind::KEYWORD,
                Self::DailyNote {..} => CompletionItemKind::EVENT
            }),
            label_details: match self {
                Self::Unresolved { match_string: _, infile_ref: _, .. } => Some(CompletionItemLabelDetails{
                    detail: Some("Unresolved".into()),
                    description: None
                }),
                _ => None
            },
            text_edit: Some(text_edit),
            filter_text: Some(filter_text.to_string()),
            documentation: preview_referenceable(vault, referenceable).map(Documentation::MarkupContent),
            ..Default::default()
        }
    }

    /// Refname to be inserted into the document
    fn refname(&self) -> String {
        match self {
            Self::DailyNote(MDDailyNote { ref_name, .. }) => ref_name.to_string() ,
            _ => self.match_string().to_string()
        }
    }

}


impl<'a> Completable<'a, MarkdownLinkCompleter<'a>>  for LinkCompletion<'a> {
    fn completions(&self, markdown_link_completer: &MarkdownLinkCompleter<'a>) -> impl Iterator<Item = CompletionItem> {

        let label = self.match_string();

        let display = &markdown_link_completer.display;

        let link_infile_ref = match self {
            File { mdfile: _, match_string: _, .. } 
            | Self::Block { match_string: _, .. }
            | Self::DailyNote {..} 
                => None,
            Self::Heading { heading, match_string: _, .. } => Some(heading.heading_text.to_string()),
            Self::Unresolved { match_string: _, infile_ref, .. } => infile_ref.clone()
        };

        let binding = (display.0.as_str(), link_infile_ref);
        let link_display_text = match binding {
            ("", Some(ref infile)) => &infile,
            // Get the first heading of the file, if possible. 
            ("", None) => match self {
                Self::File { mdfile, match_string: _, .. } => mdfile.headings.get(0).map(|heading| heading.heading_text.as_str()).unwrap_or(""),
                _ => ""
            }
            (display, _) => display,
        };

        let link_display_text = format!(
            "${{1:{}}}",
            link_display_text,
        );

        let text_edit = markdown_link_completer.completion_text_edit(Some(&link_display_text), &label);


        let filter_text = markdown_link_completer.completion_filter_text(label);

        std::iter::once(CompletionItem {
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..self.default_completion(&label, text_edit, &filter_text, markdown_link_completer.vault())
        })

    }
}


impl<'a> Completable<'a, WikiLinkCompleter<'a>> for LinkCompletion<'a> {
    fn completions(&self, completer: &WikiLinkCompleter<'a>) -> impl Iterator<Item = CompletionItem> {


        let refname = self.refname();
        let match_text = self.match_string();

        let text_edit = completer.completion_text_edit(None, &refname);

        let filter_text = completer.completion_filter_text(&match_text);

        std::iter::once(self.default_completion(&match_text, text_edit, &filter_text, completer.vault()))
    }
}


impl Matchable for LinkCompletion<'_> {
    /// The string used for fuzzy matching
    fn match_string(&self) -> &str {
        match self {
            File{mdfile: _, match_string, ..}
            | Heading { heading: _, match_string, .. }
            | Block { match_string, .. }
            | Unresolved { match_string, .. }
            | DailyNote(MDDailyNote { match_string, .. })  
                => &match_string,
        }
    }
}


#[derive(Clone, Debug)]
pub struct MDDailyNote<'a> {
    match_string: String,
    ref_name: String,
    referenceable: Referenceable<'a>
}

impl MDDailyNote<'_> {
    /// The refname used for fuzzy matching a completion - not the actual inserted text
    fn new<'a>(referenceable: Referenceable<'a>, completer: &impl LinkCompleter<'a>) -> Option<MDDailyNote<'a>> {

        let Some((filerefname, filter_refname)) = (match referenceable {
            Referenceable::File(path, mdfile) => {
                let filename = path.file_name();
                let dailynote_format = &completer.settings().dailynote;
                let date = filename.and_then(|filename| {

                    let filename = filename.to_str()?;
                    let filename = filename.replace(".md", "");
                    chrono::NaiveDate::parse_from_str(&filename, dailynote_format).ok()

                });
                let today = chrono::Local::now().date_naive();
                let file_refname = mdfile.path.file_stem()?.to_str()?.to_string();

                date.and_then(|date| match (date - today).num_days() {
                    0 => Some(format!("today: {}", file_refname)),
                    1 if date > today => Some(format!("tomorrow: {}", file_refname)),
                    1 if date < today => Some(format!("yesterday: {}", file_refname)),
                    _ => None
                }).map(|thing| (file_refname, thing))
            },
            _ => None
        }) else {
            return None;
        };

        Some(MDDailyNote{
            match_string: filter_refname,
            ref_name: filerefname,
            referenceable
        })
    }
}
