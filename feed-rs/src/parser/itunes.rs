use crate::model::{Image, MediaCredit, MediaObject, MediaThumbnail, Feed, Category, MediaRating};
use crate::parser::atom::handle_text;
use crate::parser::util::{if_some_then, parse_npt};
use crate::parser::ParseFeedResult;
use crate::xml::{Element, NS};
use std::io::BufRead;
use std::time::Duration;

// TODO:
// - Handle <itunes:> elements for the whole feed in <channel>
// - More elements like itunes:subtitle, itunes:episode etc.

// Process <itunes> elements at channel level updating the Feed object as required
pub(crate) fn handle_itunes_channel_element<R: BufRead>(element: Element<R>, feed: &mut Feed) -> ParseFeedResult<()> {
    match element.ns_and_tag() {
        (Some(NS::Itunes), "image") => if_some_then(handle_image(element), |image| {
            // Assign to feed logo if not already set
            if feed.logo.is_none() {
                feed.logo = Some(image.image);
            }
        }),

        (Some(NS::Itunes), "category") => if_some_then(handle_category(element), |category| feed.categories.push(category)),

        (Some(NS::Itunes), "explicit") => if_some_then(handle_explicit(element), |rating| {
            // Assign if not already set from media
            if feed.rating.is_none() {
                feed.rating = Some(rating);
            }
        }),

        // Nothing required for unknown elements
        _ => {}
    }

    Ok(())
}

// Process <itunes> elements at item level and turn them into something that looks like MediaRSS objects.
pub(crate) fn handle_itunes_item_element<R: BufRead>(element: Element<R>, media_obj: &mut MediaObject) -> ParseFeedResult<()> {
    match element.ns_and_tag() {
        (Some(NS::Itunes), "title") => media_obj.title = handle_text(element)?,

        (Some(NS::Itunes), "image") => if_some_then(handle_image(element), |thumbnail| media_obj.thumbnails.push(thumbnail)),

        (Some(NS::Itunes), "duration") => if_some_then(handle_duration(element), |duration| media_obj.duration = Some(duration)),

        (Some(NS::Itunes), "author") => if_some_then(handle_author(element), |credit| media_obj.credits.push(credit)),

        (Some(NS::Itunes), "summary") => media_obj.description = handle_text(element)?,

        // Nothing required for unknown elements
        _ => {}
    }

    Ok(())
}

// Handles <itunes:author>
fn handle_author<R: BufRead>(element: Element<R>) -> Option<MediaCredit> {
    element.child_as_text().map(MediaCredit::new)
}

// Handles <itunes:category>
fn handle_category<R: BufRead>(element: Element<R>) -> Option<Category> {
    element.attr_value("text").map(|text| Category::new(&text))
}

// Handles <itunes:duration>
fn handle_duration<R: BufRead>(element: Element<R>) -> Option<Duration> {
    element.child_as_text().and_then(|text| parse_npt(&text))
}

// Handles <itunes:explicit> by mapping to {true|false} and wrapping in MediaRating instance
fn handle_explicit<R: BufRead>(element: Element<R>) -> Option<MediaRating> {
    element.child_as_text()
        .filter(|v| v.to_lowercase() == "true")
        .map(|v| MediaRating::new(v).urn("itunes"))
}

// Handles <itunes:image>
fn handle_image<R: BufRead>(element: Element<R>) -> Option<MediaThumbnail> {
    element.attr_value("href").map(|url| MediaThumbnail::new(Image::new(url)))
}
