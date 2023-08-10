use bibtex_utils::field::{
    author::AuthorField,
    date::DateField,
    number::{NumberField, NumberFieldData},
    text::TextField,
};
use isocountry::CountryCode;
use itertools::Itertools;
use syntax::bibtex;
use titlecase::titlecase;
use url::Url;

use super::{
    entry::{EntryData, EntryKind},
    output::{Inline, InlineBuilder, Punct},
};

#[derive(Debug, Default)]
pub struct Driver {
    builder: InlineBuilder,
}

impl Driver {
    pub fn process(&mut self, entry: &bibtex::Entry) {
        let entry = EntryData::from(entry);
        match entry.kind {
            EntryKind::Article
            | EntryKind::DataSet
            | EntryKind::Manual
            | EntryKind::Misc
            | EntryKind::Online
            | EntryKind::Electronic
            | EntryKind::Www
            | EntryKind::Proceedings
            | EntryKind::MVProceedings
            | EntryKind::Set
            | EntryKind::Software
            | EntryKind::Thesis
            | EntryKind::MasterThesis
            | EntryKind::PhdThesis
            | EntryKind::Unknown => self.article_like(entry),
            EntryKind::Book
            | EntryKind::MVBook
            | EntryKind::InBook
            | EntryKind::BookInBook
            | EntryKind::SuppBook
            | EntryKind::Booklet
            | EntryKind::Collection
            | EntryKind::MVCollection
            | EntryKind::InCollection
            | EntryKind::SuppCollection
            | EntryKind::Periodical
            | EntryKind::SuppPeriodical
            | EntryKind::InProceedings
            | EntryKind::Conference
            | EntryKind::Reference
            | EntryKind::MVReference
            | EntryKind::InReference
            | EntryKind::Report
            | EntryKind::TechReport => self.book_like(entry),
            EntryKind::Patent => self.patent(entry),
        };
    }

    fn article_like(&mut self, mut entry: EntryData) {
        self.author(&mut entry);
        self.title(&mut entry);
        self.translator(&mut entry);
        self.commentator(&mut entry);
        self.annotator(&mut entry);
        self.version(&mut entry);
        self.journal(&mut entry);
        self.article_series(&mut entry);

        if let Some(volume) = entry.number.remove(&NumberField::Volume) {
            let number = entry
                .number
                .remove(&NumberField::Number)
                .map(|data| format!(".{}", data))
                .unwrap_or_default();

            self.builder.push(
                Inline::Regular(format!("{}{}", volume, number)),
                Punct::Space,
                Punct::Space,
            );
        }

        self.eid(&mut entry);
        self.issue_and_date(&mut entry);
        self.issue_title(&mut entry);
        self.editors(&mut entry);
        self.note(&mut entry);
        self.pages(&mut entry);
        self.issn(&mut entry);
        self.doi(&mut entry);
        self.eprint(&mut entry);
        self.url(&mut entry);
        self.addendum(&mut entry);
        self.pubstate(&mut entry);
    }

    fn book_like(&mut self, mut entry: EntryData) {
        self.author(&mut entry);
        self.title(&mut entry);
        self.inbook_title(&mut entry);
        self.main_title(&mut entry);
        self.event(&mut entry);
        self.editors(&mut entry);
        self.translator(&mut entry);
        self.commentator(&mut entry);
        self.annotator(&mut entry);
        self.introduction(&mut entry);
        self.foreword(&mut entry);
        self.afterword(&mut entry);
        self.edition(&mut entry);
        self.book_volume(&mut entry);
        self.volumes(&mut entry);
        self.book_series(&mut entry);
        self.how_published(&mut entry);
        self.note(&mut entry);
        self.publisher(&mut entry);
        self.date(&mut entry);
        self.chapter(&mut entry);
        self.eid(&mut entry);
        self.pages(&mut entry);
        self.page_total(&mut entry);
        self.isbn(&mut entry);
        self.doi(&mut entry);
        self.eprint(&mut entry);
        self.url(&mut entry);
        self.addendum(&mut entry);
        self.pubstate(&mut entry);
    }

    fn patent(&mut self, mut entry: EntryData) {
        self.author(&mut entry);
        self.title(&mut entry);
        self.patent_number(&mut entry);
        self.holder(&mut entry);
        self.date(&mut entry);
        self.doi(&mut entry);
        self.eprint(&mut entry);
        self.url(&mut entry);
        self.addendum(&mut entry);
        self.pubstate(&mut entry);
    }

    fn author(&mut self, entry: &mut EntryData) -> Option<()> {
        let author = entry.author.remove(&AuthorField::Author)?;
        self.builder.push(
            Inline::Regular(author.to_string()),
            Punct::Nothing,
            Punct::Colon,
        );

        Some(())
    }

    fn commentator(&mut self, entry: &mut EntryData) -> Option<()> {
        let commentator = entry.author.remove(&AuthorField::Commentator)?;
        self.builder.push(
            Inline::Regular(format!("With a comment. by {}", commentator)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn translator(&mut self, entry: &mut EntryData) -> Option<()> {
        let translator = entry.author.remove(&AuthorField::Translator)?;
        self.builder.push(
            Inline::Regular(format!("Trans. by {}", translator)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn annotator(&mut self, entry: &mut EntryData) -> Option<()> {
        let annotator = entry.author.remove(&AuthorField::Annotator)?;
        self.builder.push(
            Inline::Regular(format!("With annots. by {}", annotator)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn editors(&mut self, entry: &mut EntryData) {
        self.editor(entry, AuthorField::Editor, TextField::EditorType);
        self.editor(entry, AuthorField::EditorA, TextField::EditorTypeA);
        self.editor(entry, AuthorField::EditorB, TextField::EditorTypeB);
        self.editor(entry, AuthorField::EditorC, TextField::EditorTypeC);
    }

    fn editor(
        &mut self,
        entry: &mut EntryData,
        name_field: AuthorField,
        type_field: TextField,
    ) -> Option<()> {
        let editor = entry.author.remove(&name_field)?;
        let editor_type = entry
            .text
            .remove(&type_field)
            .map_or_else(|| "Ed. by".to_string(), |data| data.text);

        self.builder.push(
            Inline::Regular(format!("{editor_type} {editor}")),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn title(&mut self, entry: &mut EntryData) -> Option<()> {
        let title = entry.text.remove(&TextField::Title)?;
        let subtitle = entry
            .text
            .remove(&TextField::Subtitle)
            .map(|data| format!(". {}", data.text))
            .unwrap_or_default();

        self.builder.push(
            Inline::Quoted(format!("{}{}", title.text, subtitle)),
            Punct::Nothing,
            Punct::Dot,
        );

        let addon = entry.text.remove(&TextField::TitleAddon)?;
        self.builder
            .push(Inline::Regular(addon.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn main_title(&mut self, entry: &mut EntryData) -> Option<()> {
        let title = entry.text.remove(&TextField::MainTitle)?;
        let subtitle = entry
            .text
            .remove(&TextField::MainSubtitle)
            .map(|data| format!(". {}", data.text))
            .unwrap_or_default();

        self.builder.push(
            Inline::Italic(titlecase(&format!("{}{}", title.text, subtitle))),
            Punct::Dot,
            Punct::Dot,
        );

        let addon = entry.text.remove(&TextField::MainTitleAddon)?;
        self.builder
            .push(Inline::Regular(addon.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn inbook_title(&mut self, entry: &mut EntryData) -> Option<()> {
        let title = entry.text.remove(&TextField::BookTitle)?;
        let subtitle = entry
            .text
            .remove(&TextField::BookSubtitle)
            .map(|data| format!(". {}", data.text))
            .unwrap_or_default();

        self.builder.push(
            Inline::Italic(titlecase(&format!("{}{}", title.text, subtitle))),
            Punct::Dot,
            Punct::Dot,
        );

        let addon = entry.text.remove(&TextField::BookTitleAddon)?;
        self.builder
            .push(Inline::Regular(addon.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn issue_title(&mut self, entry: &mut EntryData) -> Option<()> {
        let issue = entry.text.remove(&TextField::IssueTitle)?;
        let subtitle = entry
            .text
            .remove(&TextField::IssueSubtitle)
            .map(|data| format!(" {}", data.text))
            .unwrap_or_default();

        self.builder.push(
            Inline::Italic(titlecase(&format!("{}{}", issue.text, subtitle))),
            Punct::Colon,
            Punct::Space,
        );

        if let Some(addon) = entry.text.remove(&TextField::IssueTitleAddon) {
            self.builder
                .push(Inline::Regular(addon.text), Punct::Dot, Punct::Dot);
        }

        Some(())
    }

    fn event(&mut self, entry: &mut EntryData) -> Option<()> {
        let title = entry.text.remove(&TextField::EventTitle)?;
        let addon = entry
            .text
            .remove(&TextField::EventTitleAddon)
            .map_or(String::new(), |addon| format!(". {}", addon.text));

        let venue = entry.text.remove(&TextField::Venue);
        let date = entry.date.remove(&DateField::EventDate);
        let venue_and_date = match (venue, date) {
            (None, None) => String::new(),
            (None, Some(date)) => format!(" ({date})"),
            (Some(venue), None) => format!(" ({})", venue.text),
            (Some(venue), Some(date)) => format!(" ({}, {})", venue.text, date),
        };

        self.builder.push(
            Inline::Regular(format!("{}{}{}", title.text, addon, venue_and_date)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn patent_number(&mut self, entry: &mut EntryData) -> Option<()> {
        let number = entry.number.remove(&NumberField::Number)?;

        let location = entry
            .text
            .remove(&TextField::Location)
            .map_or(String::new(), |location| {
                let text = location
                    .text
                    .split_whitespace()
                    .map(|word| {
                        if word == "countryuk" {
                            "United Kingdom"
                        } else {
                            word.strip_prefix("country")
                                .and_then(|code| CountryCode::for_alpha2_caseless(code).ok())
                                .map_or(word, |country| country.name())
                        }
                    })
                    .join(" ");

                format!(" ({})", text)
            });

        self.builder.push(
            Inline::Regular(format!("{}{}", number, location)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn holder(&mut self, entry: &mut EntryData) -> Option<()> {
        let holder = entry.text.remove(&TextField::Holder)?;
        self.builder
            .push(Inline::Regular(holder.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn version(&mut self, entry: &mut EntryData) -> Option<()> {
        let version = entry.text.remove(&TextField::Version)?;
        self.builder.push(
            Inline::Regular(format!("Version {}", version.text)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn article_series(&mut self, entry: &mut EntryData) -> Option<()> {
        let series = entry.text.remove(&TextField::Series)?;
        self.builder
            .push(Inline::Regular(series.text), Punct::Space, Punct::Space);

        Some(())
    }

    fn issue_and_date(&mut self, entry: &mut EntryData) -> Option<()> {
        let date = [DateField::Date, DateField::Year, DateField::Month]
            .iter()
            .filter_map(|field| entry.date.remove(field))
            .reduce(|a, b| a + b)?;

        let issue = entry
            .text
            .remove(&TextField::Issue)
            .map(|data| format!("{} ", data.text))
            .unwrap_or_default();

        self.builder.push(
            Inline::Regular(format!("({issue}{date})")),
            Punct::Space,
            Punct::Colon,
        );

        Some(())
    }

    fn chapter(&mut self, entry: &mut EntryData) -> Option<()> {
        let chapter = entry.text.remove(&TextField::Chapter)?;
        self.builder.push(
            Inline::Regular(format!("Chap. {}", chapter.text)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn date(&mut self, entry: &mut EntryData) -> Option<()> {
        let date = [DateField::Date, DateField::Year, DateField::Month]
            .iter()
            .filter_map(|field| entry.date.remove(field))
            .reduce(|a, b| a + b)?;

        self.builder
            .push(Inline::Regular(format!("{date}")), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn how_published(&mut self, entry: &mut EntryData) -> Option<()> {
        let how_published = entry.text.remove(&TextField::HowPublished)?;
        self.builder
            .push(Inline::Regular(how_published.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn publisher(&mut self, entry: &mut EntryData) -> Option<()> {
        let publisher = entry.text.remove(&TextField::Publisher)?.text;
        let location = entry
            .text
            .remove(&TextField::Location)
            .map_or(String::new(), |location| format!("{}: ", location.text));

        self.builder.push(
            Inline::Regular(format!("{location}{publisher}")),
            Punct::Dot,
            Punct::Comma,
        );

        Some(())
    }

    fn book_series(&mut self, entry: &mut EntryData) -> Option<()> {
        let series = entry.text.remove(&TextField::Series)?;
        let number = entry
            .number
            .remove(&NumberField::Number)
            .map_or(String::new(), |number| format!(" {}", number));

        self.builder.push(
            Inline::Regular(format!("{}{}", series.text, number)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn volumes(&mut self, entry: &mut EntryData) -> Option<()> {
        let volumes = entry.number.remove(&NumberField::Volume)?;
        self.builder.push(
            Inline::Regular(format!("{volumes} vols")),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn book_volume(&mut self, entry: &mut EntryData) -> Option<()> {
        let volume = entry.number.remove(&NumberField::Volume)?;
        let part = entry
            .number
            .remove(&NumberField::Part)
            .map_or(String::new(), |part| format!(".{part}"));

        self.builder.push(
            Inline::Regular(format!("Vol. {}{}", volume, part)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn pubstate(&mut self, entry: &mut EntryData) -> Option<()> {
        let pubstate = entry.text.remove(&TextField::Pubstate)?;
        self.builder
            .push(Inline::Regular(pubstate.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn addendum(&mut self, entry: &mut EntryData) -> Option<()> {
        let addendum = entry.text.remove(&TextField::Addendum)?;
        self.builder
            .push(Inline::Regular(addendum.text), Punct::Dot, Punct::Dot);

        Some(())
    }

    fn edition(&mut self, entry: &mut EntryData) -> Option<()> {
        let text = match entry.number.remove(&NumberField::Edition)? {
            NumberFieldData::Scalar(1) => "1st".to_string(),
            NumberFieldData::Scalar(2) => "2nd".to_string(),
            NumberFieldData::Scalar(3) => "3rd".to_string(),
            NumberFieldData::Scalar(number) => format!("{}th", number),
            NumberFieldData::Range(_, _) => return None,
            NumberFieldData::Other(text) => text,
        };

        self.builder
            .push(Inline::Regular(text), Punct::Dot, Punct::Dot);
        Some(())
    }

    fn introduction(&mut self, entry: &mut EntryData) -> Option<()> {
        let author = entry.author.remove(&AuthorField::Introduction)?;
        self.builder.push(
            Inline::Regular(format!("With an intro. by {}", author)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn foreword(&mut self, entry: &mut EntryData) -> Option<()> {
        let author = entry.author.remove(&AuthorField::Commentator)?;
        self.builder.push(
            Inline::Regular(format!("With a forew. by {}", author)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn afterword(&mut self, entry: &mut EntryData) -> Option<()> {
        let author = entry.author.remove(&AuthorField::Commentator)?;
        self.builder.push(
            Inline::Regular(format!("With an afterw. by {}", author)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn note(&mut self, entry: &mut EntryData) -> Option<()> {
        let note = entry.text.remove(&TextField::Note)?;
        self.builder
            .push(Inline::Regular(note.text), Punct::Dot, Punct::Comma);

        Some(())
    }

    fn pages(&mut self, entry: &mut EntryData) -> Option<()> {
        let pages = entry.number.remove(&NumberField::Pages)?;
        self.builder.push(
            Inline::Regular(pages.to_string()),
            Punct::Comma,
            Punct::Space,
        );

        Some(())
    }

    fn page_total(&mut self, entry: &mut EntryData) -> Option<()> {
        let page_total = entry.number.remove(&NumberField::PageTotal)?;
        self.builder.push(
            Inline::Regular(format!("{} pp", page_total)),
            Punct::Dot,
            Punct::Dot,
        );

        Some(())
    }

    fn journal(&mut self, entry: &mut EntryData) -> Option<()> {
        let title = entry
            .text
            .remove(&TextField::Journal)
            .or_else(|| entry.text.remove(&TextField::JournalTitle))?;

        let subtitle = entry
            .text
            .remove(&TextField::JournalSubtitle)
            .map_or(String::new(), |data| format!(" {}", data.text));

        self.builder.push(
            Inline::Italic(titlecase(&format!("{}{}", title.text, subtitle))),
            Punct::Dot,
            Punct::Space,
        );

        Some(())
    }

    fn eid(&mut self, entry: &mut EntryData) -> Option<()> {
        let eid = entry.text.remove(&TextField::Eid)?;
        self.builder
            .push(Inline::Regular(eid.text), Punct::Comma, Punct::Space);

        Some(())
    }

    fn isbn(&mut self, entry: &mut EntryData) -> Option<()> {
        let isbn = entry.text.remove(&TextField::Isbn)?;
        self.builder.push(
            Inline::Regular("ISBN".to_string()),
            Punct::Dot,
            Punct::Colon,
        );

        self.builder
            .push(Inline::Regular(isbn.text), Punct::Space, Punct::Dot);

        Some(())
    }

    fn issn(&mut self, entry: &mut EntryData) -> Option<()> {
        let issn = entry.text.remove(&TextField::Issn)?;
        self.builder.push(
            Inline::Regular("ISSN".to_string()),
            Punct::Dot,
            Punct::Colon,
        );

        self.builder
            .push(Inline::Regular(issn.text), Punct::Space, Punct::Dot);

        Some(())
    }

    fn url(&mut self, entry: &mut EntryData) -> Option<()> {
        let url = entry.text.remove(&TextField::Url)?;

        self.builder
            .push(Inline::Regular("URL".to_string()), Punct::Dot, Punct::Colon);

        let url = url.text;
        let alt = url.clone();

        self.builder
            .push(Inline::Link { url, alt }, Punct::Space, Punct::Space);

        let date = entry.date.remove(&DateField::UrlDate)?;
        self.builder.push(
            Inline::Regular(format!("({date})")),
            Punct::Space,
            Punct::Space,
        );

        Some(())
    }

    fn doi(&mut self, entry: &mut EntryData) -> Option<()> {
        let doi = entry.text.remove(&TextField::Doi)?;
        self.builder
            .push(Inline::Regular("DOI".to_string()), Punct::Dot, Punct::Colon);

        let alt = Url::parse(&doi.text)
            .ok()
            .filter(|url| !url.cannot_be_a_base())
            .map_or(doi.text, |url| url.path()[1..].to_string());

        let url = format!("https://doi.org/{alt}");

        self.builder
            .push(Inline::Link { url, alt }, Punct::Space, Punct::Dot);

        Some(())
    }

    fn eprint(&mut self, entry: &mut EntryData) -> Option<()> {
        let eprint = entry.text.remove(&TextField::Eprint)?;
        let eprint_type = entry
            .text
            .remove(&TextField::EprintType)
            .map_or_else(|| "eprint".to_string(), |data| data.text);

        if eprint_type.eq_ignore_ascii_case("arxiv") {
            self.builder.push(
                Inline::Regular("arXiv".to_string()),
                Punct::Dot,
                Punct::Colon,
            );

            self.builder.push(
                Inline::Link {
                    url: format!("https://arxiv.org/abs/{}", eprint.text),
                    alt: eprint.text,
                },
                Punct::Space,
                Punct::Dot,
            );
        } else {
            self.builder
                .push(Inline::Regular(eprint_type), Punct::Dot, Punct::Colon);

            self.builder
                .push(Inline::Regular(eprint.text), Punct::Space, Punct::Dot);
        }

        Some(())
    }

    pub fn finish(self) -> impl Iterator<Item = (Inline, Punct)> {
        self.builder.finish()
    }
}
