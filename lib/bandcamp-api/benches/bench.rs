use {
    criterion::*,
    scrape::Scraper,
    bandcamp_api::data::{
        Query,
        search::*
    }
};

criterion_group!(benches, search);
criterion_main!(benches);

fn search(c: &mut Criterion) {
    const DOC: &str = include_str!("search.html");

    let new = || Scraper::new(std::io::Cursor::new(DOC.into())).into();

    c.bench_function("search", |b| b.iter(|| Search::query(new())));
}
