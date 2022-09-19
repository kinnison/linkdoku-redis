//! Markdown renderer

use pulldown_cmark::{Alignment, Event, HeadingLevel, LinkType, Options, Parser, Tag};
use yew::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct MarkdownRenderProps {
    pub markdown: String,
}

#[function_component(MarkdownRender)]
pub fn render_markdown_block(props: &MarkdownRenderProps) -> Html {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let mut stack_depth = 0usize;
    let mut inlines: Vec<Vec<Html>> = vec![Vec::new()];

    let mut alignments: Vec<Alignment> = Vec::new();
    let mut in_table_head = true;
    let mut cell_index = 0;

    let events = Parser::new_ext(&props.markdown, options);

    for event in events {
        match event {
            Event::Start(tag) => {
                stack_depth += 1;
                inlines.push(Vec::new());
                match tag {
                    Tag::Table(aligns) => {
                        alignments = aligns;
                        in_table_head = false;
                        cell_index = 0;
                    }
                    Tag::TableHead => {
                        in_table_head = true;
                        cell_index = 0;
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                // Process tag
                if stack_depth == 0 {
                    // We tried to end a tag when stack depth was 1, just give up Now
                    stack_depth = 10000;
                    break;
                }
                stack_depth -= 1;
                let mut request_fake_start = false;
                let content = inlines.pop().unwrap().into_iter().collect::<Html>();
                let new_content = match tag {
                    Tag::Paragraph => {
                        html! {
                            <p>{content}</p>
                        }
                    }
                    Tag::BlockQuote => {
                        html! {
                            <blockquote>{content}</blockquote>
                        }
                    }
                    Tag::Item => {
                        html! {
                            <li>{content}</li>
                        }
                    }
                    Tag::TableHead => {
                        // Fake a 'start' of a tbody as it were
                        request_fake_start = true;
                        cell_index = 0;
                        in_table_head = false;
                        html! {<thead><tr>{content}</tr></thead>}
                    }
                    Tag::TableRow => {
                        html! {
                            <tr>{content}</tr>
                        }
                    }
                    Tag::TableCell => {
                        let class = match alignments.get(cell_index) {
                            None | Some(Alignment::None) => classes!(),
                            Some(Alignment::Left) => classes!("has-text-left"),
                            Some(Alignment::Right) => classes!("has-text-right"),
                            Some(Alignment::Center) => classes!("has-text-centered"),
                        };
                        cell_index += 1;
                        if in_table_head {
                            html! {
                                <th class={class}>{content}</th>
                            }
                        } else {
                            html! {
                                <td class={class}>{content}</td>
                            }
                        }
                    }
                    Tag::Table(_) => {
                        // Tables are a bit magical, `content` is the body
                        // and we need to pop another for the head
                        if stack_depth == 0 {
                            // We tried to end a tag when stack's depth was 1, just give up Now
                            stack_depth = 10000;
                            break;
                        }
                        let thead = inlines.pop().unwrap().into_iter().collect::<Html>();
                        stack_depth -= 1;
                        html! {
                            <table>
                                {thead}
                                <tbody>
                                    {content}
                                </tbody>
                            </table>
                        }
                    }
                    Tag::Emphasis => {
                        html! {
                            <em>{content}</em>
                        }
                    }
                    Tag::Strong => {
                        html! {
                            <strong>{content}</strong>
                        }
                    }
                    Tag::Strikethrough => {
                        html! {
                            <s>{content}</s>
                        }
                    }
                    Tag::List(marker) => {
                        if let Some(start) = marker {
                            if start == 1 {
                                html! {
                                    <ol>{content}</ol>
                                }
                            } else {
                                html! {
                                    <ol start={format!("{}", start)}>{content}</ol>
                                }
                            }
                        } else {
                            html! {
                                <ul>{content}</ul>
                            }
                        }
                    }
                    Tag::CodeBlock(_kind) => {
                        // For now we're ignoring `kind`
                        html! {
                            <pre><code>{content}</code></pre>
                        }
                    }
                    Tag::FootnoteDefinition(_) => {
                        unreachable!()
                    }
                    Tag::Heading(level, id, classes) => {
                        let id = id.map(String::from);
                        let classes = {
                            let mut ret = Classes::new();
                            for class in classes {
                                ret.push(classes!(class.to_string()))
                            }
                            ret
                        };
                        match level {
                            HeadingLevel::H1 => html! {<h1 id={id} class={classes}>{content}</h1>},
                            HeadingLevel::H2 => html! {<h2 id={id} class={classes}>{content}</h2>},
                            HeadingLevel::H3 => html! {<h3 id={id} class={classes}>{content}</h3>},
                            HeadingLevel::H4 => html! {<h4 id={id} class={classes}>{content}</h4>},
                            HeadingLevel::H5 => html! {<h5 id={id} class={classes}>{content}</h5>},
                            HeadingLevel::H6 => html! {<h6 id={id} class={classes}>{content}</h6>},
                        }
                    }
                    Tag::Link(linktype, url, title) => {
                        // TODO: We should implement some kind of link transformer eventually
                        let url = match linktype {
                            LinkType::Email => format!("mailto:{}", url),
                            _ => url.into_string(),
                        };
                        html! {
                            <a href={url} title={title.into_string()}>{content}</a>
                        }
                    }
                    Tag::Image(_linktype, url, title) => {
                        // TODO: We should implement some kind of image URL transformer eventually
                        html! {
                            <img src={url.into_string()} title={title.into_string()} />
                        }
                    }
                };
                inlines[stack_depth].push(new_content);
                if request_fake_start {
                    // For example, table heads request this
                    inlines.push(Vec::new());
                    stack_depth += 1;
                }
            }
            Event::Text(content) => {
                // Process plain text
                inlines[stack_depth].push(html! {
                    <>
                    {content.into_string()}
                    </>
                });
            }
            Event::Code(content) => {
                // Process code text
                inlines[stack_depth].push(html! {
                    <code>{content.into_string()}</code>
                });
            }
            Event::Html(content) => {
                // Process plain HTML content.
                // Since we're not particularly interested in people messing with us, we won't allow this
                // so instead we'll render it as preformatted text
                inlines[stack_depth].push(html! {
                    <pre><code>
                        {content.into_string()}
                    </code></pre>
                });
            }
            Event::FootnoteReference(_noteref) => {
                // We do not enable footnotes, so this can't happen
                unreachable!()
            }
            Event::SoftBreak => {
                // Process soft break
                inlines[stack_depth].push(html! {{" "}});
            }
            Event::HardBreak => {
                // Process hard break
                inlines[stack_depth].push(html! {
                    <br />
                });
            }
            Event::Rule => {
                // Process rule
                inlines[stack_depth].push(html! {
                    <hr />
                });
            }
            Event::TaskListMarker(checked) => {
                // process task list marker
                inlines[stack_depth].push(html! {
                    <input type={"checkbox"} checked={checked} disabled={true} />
                });
            }
        }
    }
    if stack_depth != 0 || inlines.len() != 1 {
        return html! {
            <div class={"notification is-danger"}>
                {"Internal error rendering markdown, stack not balanced"}
            </div>
        };
    }

    html! {
        <div class={"content markdown"}>
            {
                inlines.pop().unwrap().into_iter().collect::<Html>()
            }
        </div>
    }
}
