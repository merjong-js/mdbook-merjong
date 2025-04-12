use mdbook::{
    book::{Book, BookItem},
    errors::Result as MdbookResult,
    preprocess::{Preprocessor, PreprocessorContext},
};
use pulldown_cmark::{CodeBlockKind::*, Event, Options, Parser, Tag, TagEnd};

pub struct Merjong;

impl Preprocessor for Merjong {
    fn name(&self) -> &str {
        "merjong"
    }
    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> MdbookResult<Book> {
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }
            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(preprocess(&chapter.content).map(|md| {
                    chapter.content = md;
                }));
            }
        });
        res.unwrap_or(Ok(())).map(|_| book)
    }
    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

fn preprocess(content: &str) -> MdbookResult<String> {
    let mut merjong_content = String::new();
    let mut in_merjong_block = false;

    let options = Options::empty();

    let mut code_span = 0..0;
    let mut start_new_code_span = true;

    let mut merjong_blocks = vec![];

    let events = Parser::new_ext(content, options);
    for (event, span) in events.into_offset_iter() {
        if let Event::Start(Tag::CodeBlock(Fenced(code))) = &event {
            if code.as_ref() == "merjong" {
                in_merjong_block = true;
                merjong_content.clear();
            }
        }

        if !in_merjong_block {
            continue;
        }

        if let Event::Text(_) = event {
            if start_new_code_span {
                code_span = span;
                start_new_code_span = false;
            } else {
                code_span = code_span.start..span.end;
            }

            continue;
        }

        if let Event::End(TagEnd::CodeBlock) = &event {
            in_merjong_block = false;

            let merjong_content = &content[code_span.clone()]
                .replace("\r\n", "\n")
                .trim_end()
                .to_string();
            let merjong_code = format!("<pre class=\"merjong\">{}</pre>", merjong_content);
            merjong_blocks.push((span, merjong_code));
            start_new_code_span = true;
        }
    }

    let mut new_content = content.to_string();

    for (span, block) in merjong_blocks.iter().rev() {
        let pre_content = &new_content[..span.start];
        let post_content = &new_content[span.end..];
        new_content = format!("{}{}{}", pre_content, block, post_content);
    }

    Ok(new_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn test_preprocess() {
        let content = r#"
# Chapter 1

```merjong
234m-234m-234m-222p-5m-5m
````

"#;

        let result = preprocess(content).expect("preprocess failed");

        println!("test_preprocess' s Result: {}", result);
    }
}
