//! Proc macros for `souprune_vessel`.
//!
//! `souprune_vessel` 的过程宏。

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Result, Token, braced, bracketed, parse_macro_input};

enum PerformanceSection {
    Prototypes(SectionEntries),
    Behaviors(SectionEntries),
    Timeline(SectionItems),
    Duration(Expr),
}

enum EntryItem {
    Pair { key: Expr, value: Expr },
    Spread(Expr),
}

struct SectionEntries {
    items: Vec<EntryItem>,
}

enum TimelineItem {
    Value(Expr),
    Spread(Expr),
}

struct SectionItems {
    items: Vec<TimelineItem>,
}

struct PerformanceInput {
    sections: Vec<PerformanceSection>,
}

impl Parse for SectionEntries {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            if content.peek(Token![..]) {
                let _: Token![..] = content.parse()?;
                items.push(EntryItem::Spread(content.parse()?));
            } else {
                let key: Expr = content.parse()?;
                let _: Token![=>] = content.parse()?;
                let value: Expr = content.parse()?;
                items.push(EntryItem::Pair { key, value });
            }

            if content.is_empty() {
                break;
            }
            let _: Token![,] = content.parse()?;
        }

        Ok(Self { items })
    }
}

impl Parse for SectionItems {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        bracketed!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            if content.peek(Token![..]) {
                let _: Token![..] = content.parse()?;
                items.push(TimelineItem::Spread(content.parse()?));
            } else {
                items.push(TimelineItem::Value(content.parse()?));
            }

            if content.is_empty() {
                break;
            }
            let _: Token![,] = content.parse()?;
        }

        Ok(Self { items })
    }
}

impl Parse for PerformanceInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut sections = Vec::new();
        while !input.is_empty() {
            let section: Ident = input.parse()?;
            match section.to_string().as_str() {
                "prototypes" => sections.push(PerformanceSection::Prototypes(input.parse()?)),
                "behaviors" => sections.push(PerformanceSection::Behaviors(input.parse()?)),
                "timeline" => sections.push(PerformanceSection::Timeline(input.parse()?)),
                "duration" => {
                    let _: Token![:] = input.parse()?;
                    sections.push(PerformanceSection::Duration(input.parse()?));
                }
                other => {
                    return Err(syn::Error::new(
                        section.span(),
                        format!("unknown performance! section: {other}"),
                    ));
                }
            }

            if input.is_empty() {
                break;
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(Self { sections })
    }
}

#[proc_macro]
pub fn performance(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as PerformanceInput);

    let mut prototypes_body = Vec::new();
    let mut behaviors_body = Vec::new();
    let mut timeline_body = Vec::new();
    let mut duration_expr = quote!(None);

    for section in input.sections {
        match section {
            PerformanceSection::Prototypes(entries) => {
                prototypes_body.extend(entries.items.into_iter().map(|item| match item {
                    EntryItem::Pair { key, value } => quote! {
                        __vessel_prototypes.insert((#key).into(), #value);
                    },
                    EntryItem::Spread(expr) => quote! {
                        __vessel_prototypes.extend(#expr);
                    },
                }));
            }
            PerformanceSection::Behaviors(entries) => {
                behaviors_body.extend(entries.items.into_iter().map(|item| match item {
                    EntryItem::Pair { key, value } => quote! {
                        __vessel_behaviors.insert((#key).into(), #value);
                    },
                    EntryItem::Spread(expr) => quote! {
                        __vessel_behaviors.extend(#expr);
                    },
                }));
            }
            PerformanceSection::Timeline(items) => {
                timeline_body.extend(items.items.into_iter().map(|item| match item {
                    TimelineItem::Value(expr) => quote! {
                        __vessel_timeline.push(#expr);
                    },
                    TimelineItem::Spread(expr) => quote! {
                        __vessel_timeline.extend(#expr);
                    },
                }));
            }
            PerformanceSection::Duration(expr) => {
                duration_expr = quote!(Some(#expr));
            }
        }
    }

    quote! {{
        let mut __vessel_prototypes = ::std::collections::HashMap::new();
        #(#prototypes_body)*
        let mut __vessel_behaviors = ::std::collections::HashMap::new();
        #(#behaviors_body)*
        let mut __vessel_timeline = ::std::vec::Vec::new();
        #(#timeline_body)*
        ::souprune_schema::danmaku::DanmakuPerformance {
            prototypes: __vessel_prototypes,
            behaviors: __vessel_behaviors,
            timeline: __vessel_timeline,
            duration: #duration_expr,
        }
    }}
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parses_spread_sections() {
        let input = syn::parse2::<PerformanceInput>(quote! {
            prototypes { "a" => foo(), ..more_prototypes }
            behaviors { ..more_behaviors }
            timeline [event(), ..more_events]
            duration: ::souprune_schema::danmaku::DurationExpr::Literal(1.0)
        })
        .expect("performance macro input should parse");

        assert_eq!(input.sections.len(), 4);
    }
}
