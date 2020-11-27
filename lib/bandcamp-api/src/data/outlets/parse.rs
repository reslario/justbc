use {
    super::*,
    snafu::{
        Snafu,
        ResultExt,
        OptionExt
    },
    scrape::{
        BufMut,
        Scrape,
        filter::*,
        extract::*
    }
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("error extracting release information: {}", source))]
    Read { source: scrape::Error },
    #[snafu(display("missing info: {}", field))]
    MissingInfo { field: &'static str },
    #[snafu(display("error parsing outlet data: {}", source))]
    Serde { source: serde_json::Error },
    #[snafu(display("error parsing url: {}", source))]
    ParseUrl { source: url::ParseError }
}

type Result<T> = std::result::Result<T, Error>;

fn missing(field: &'static str) -> MissingInfo<&'static str> {
    MissingInfo { field }
}

pub(super) fn get_info(mut scraper: impl Scrape, buf: BufMut) -> Result<Info> {
    Ok(Info {
        name: meta(&mut scraper, "og:title", "name", buf)?,
        url: meta(&mut scraper, "og:url", "url", buf)?.parse().context(ParseUrl)?,
        bio: meta(&mut scraper, "og:description", "bio", buf).ok(),
        kind: kind(scraper, buf)?
    })
}

fn meta(scraper: impl Scrape, prop: &str, name: &'static str, buf: BufMut) -> Result<String> {
    scraper
        .into_filter(tag("meta").attr("property", prop).take(1))
        .extract(attr("content"), buf)
        .context(Read)?
        .context(missing(name))
}

fn kind(scraper: impl Scrape, buf: BufMut) -> Result<OutletKind> {
    #[derive(serde::Deserialize)]
    struct BandData {
        #[serde(default)]
        is_label: bool
    }

    scraper
        .into_filter(tag("script"))
        .find_extract(attr("data-band"), buf)
        .context(Read)?
        .map(|json| serde_json::from_str(&json)
            .context(Serde)
            .map(|BandData { is_label }| if is_label {
                OutletKind::Label
            } else {
                OutletKind::Artist
            })
        ).unwrap_or(Ok(OutletKind::Artist))
}

pub(super) fn get_releases(scraper: impl Scrape, buf: BufMut, info: &Info) -> Result<(Vec<Featured>, Vec<Release>)> {
    let mut lists = scraper
        .into_filter(div().class("leftMiddleColumns"))
        .into_take(1)
        .into_filter(tag("ol"));

    let mut list = lists.take(1);

    let featured = list
        .extract(attr("class"), buf)
        .context(Read)?
        .map(|classes| if classes.contains("featured-grid") {
            get_featured(list, buf, info)
        } else {
            Ok(vec![])
        })
        .ok_or_else(|| missing("releases").build())??;

    let releases = releases(lists.take(1), buf, info)?;

    Ok((featured, releases))
}

fn get_featured(list: impl Scrape, buf: BufMut, info: &Info) -> Result<Vec<Featured>> {
    let mut list = list.into_filter(tag("li"));

    std::iter::from_fn(|| featured(list.take(1), buf, info).transpose())
        .collect()
}

fn featured(mut list: impl Scrape, buf: BufMut, info: &Info) -> Result<Option<Featured>> {
    if let scrape::Event::Eof = list.read_event(buf).context(Read)? {
        Ok(None)
    } else {
        release_info(list, buf, info)
            .map(Featured)
            .map(Some)
    }
}

fn releases(list: impl Scrape, buf: BufMut, info: &Info) -> Result<Vec<Release>> {
    let mut list = list.into_filter(tag("li"));

    std::iter::from_fn(|| get_release(list.take(1), buf, info).transpose())
        .collect()
}

fn get_release(mut scraper: impl Scrape, buf: BufMut, info: &Info) -> Result<Option<Release>> {
    release_kind(&mut scraper, buf)?
        .map(|kind| Ok(Release {
            kind,
            info: release_info(scraper, buf, info)?
        }))
        .transpose()
}

fn release_kind(mut scraper: impl Scrape, buf: BufMut) -> Result<Option<ReleaseKind>> {
    use ReleaseKind::*;

    scraper
        .extract(attr("data-item-id"), buf)
        .context(Read)
        .map(|id| match id {
            Some(id) => match &id[..5] {
                "track" => Track.into(),
                "album" => Album.into(),
                _ => None
            },
            _ => None
        })
}

fn release_info(mut scraper: impl Scrape, buf: BufMut, info: &Info) -> Result<ReleaseInfo> {
    let url = url(&mut scraper, buf, &info.url)?;
    let (title, artist) = title_and_artist(&mut scraper, buf)?;
    scraper.read_to_end(buf).context(Read)?;

    Ok(ReleaseInfo {
        url,
        title,
        artist
    })
}


fn url(scraper: impl Scrape, buf: BufMut, base: &Url) -> Result<Url> {
    scraper
        .into_filter(tag("a"))
        .extract(attr("href"), buf)
        .context(Read)?
        .context(missing("url"))
        .and_then(|path| base.join(&path).context(ParseUrl))
}

fn title_and_artist(mut scraper: impl Scrape, buf: BufMut) -> Result<(String, Option<String>)> {
    let mut p = scraper.filter(tag("p").class(has("title")));

    let title = p
        .step(buf)
        .context(Read)?
        .extract(text, buf)
        .context(Read)?
        .context(missing("title"))?;

    let artist = p
        .filter(tag("span").class("artist-override"))
        .step(buf)
        .context(Read)?
        .extract(text, buf)
        .context(Read)?;

    Ok((title, artist))
}
