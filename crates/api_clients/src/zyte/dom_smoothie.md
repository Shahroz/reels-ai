use std::error::Error;

use dom_smoothie::{Article, Config, Readability};

fn main() -> Result<(), Box<dyn Error>> {
let html = include_str!("../test-pages/rustwiki_2024.html");
let document_url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";

    // for more options check the documentation
    let cfg = Config {
        max_elements_to_parse: 9000,
        ..Default::default()
    };
    // Readability supplies an optional `Config`. If `cfg` is omitted, 
    // then a default `Config` instance will be used.
    // Readability also supplies an optional `document_url` parameter, 
    // which may be used to transform relative URLs into absolute URLs.
    let mut readability = Readability::new(html, Some(document_url), Some(cfg))?;

    let article: Article = readability.parse()?;

    println!("{:<15} {}","Title:", article.title);
    println!("{:<15} {:?}","Byline:", article.byline);
    println!("{:<15} {}","Length:", article.length);
    println!("{:<15} {:?}","Excerpt:", article.excerpt);
    println!("{:<15} {:?}","Site Name:", article.site_name);
    println!("{:<15} {:?}", "Dir:", article.dir);
    println!("{:<15} {:?}","Published Time:", article.published_time);
    println!("{:<15} {:?}","Modified Time:", article.modified_time);
    println!("{:<15} {:?}","Image:", article.image);
    // This uri can be taken only from ld+json
    println!("{:<15} {:?}","URL", article.url);

    // Skipping article.content since it is too large.
    // To check out the html content of the article please have a look at
    // `./test-pages/rustwiki_2024_result.html`
    // println!("HTML Content: {}", article.content);

    // Skipping article.text_content since it is too large.
    // To check out the html content of the article please have a look at 
    // `./test-pages/rustwiki_2024_result.txt`
    //println!("Text Content: {}", article.text_content);

    // Right now, `text_content` provides almost the same result 
    // as readability.js, which is far from perfect. 
    // It may squash words together if element nodes don't have a whitespace before closing, 
    // and currently, I have no definitive opinion on this matter.

    Ok(())
}