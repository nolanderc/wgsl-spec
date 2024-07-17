use scraper::selectable::Selectable;

use crate::*;

macro_rules! selector {
    ($text:literal) => {{
        static SELECTOR: std::sync::OnceLock<scraper::Selector> = std::sync::OnceLock::new();
        SELECTOR.get_or_init(|| {
            scraper::Selector::parse($text).expect(concat!("could not parse selector: ", $text))
        })
    }};
}

pub fn extract_tokens(html: &scraper::Html) -> TokenInfo {
    let mut keywords = html
        .select(selector!("#keyword-summary + ul code"))
        .map(|x| x.text().next().unwrap().to_owned())
        .collect::<Vec<_>>();
    keywords.sort();

    let extract_names = |id| {
        let selector = scraper::Selector::parse(&format!("#{id} ~ ul")).unwrap();
        let mut names = html
            .select(&selector)
            .next()
            .into_iter()
            .flat_map(|x| x.select(selector!("li")).map(|x| join_words(x.text()).replace('\'', "")))
            .collect::<Vec<_>>();
        names.sort();
        names
    };

    let attributes = extract_names("attribute-names");
    let builtin_values = extract_names("builtin-value-names");
    let interpolation_type_names = extract_names("interpolation-type-names");
    let interpolation_sampling_names = extract_names("interpolation-sampling-names");
    let primitive_types = extract_names("predeclared-types");

    let mut type_generators = html
        .select(selector!("#predeclared-types ~ table"))
        .next()
        .into_iter()
        .flat_map(|x| x.select(selector!("tr td:first-child")))
        .map(|x| join_words(x.text()))
        .collect::<Vec<_>>();
    type_generators.sort();

    let mut type_aliases = BTreeMap::new();
    for table in html.select(selector!("table")) {
        let mut headings = table.select(selector!("th"));
        let Some(alias_name) = headings.next() else { continue };
        let Some(original) = headings.next() else { continue };
        if !alias_name.text().any(|x| x.contains("Predeclared alias")) {
            continue;
        }
        if !original.text().any(|x| x.contains("Original type")) {
            continue;
        }

        for row in table.select(selector!("tbody tr")) {
            let mut columns = row.select(selector!("td"));
            let Some(name) = columns.next() else { continue };
            let Some(original) = columns.next() else { continue };

            let name = join_words(name.text()).trim().to_string();
            let original = join_words(original.text()).trim().to_string();
            type_aliases.insert(name, original);
        }
    }

    TokenInfo {
        keywords,
        attributes,
        builtin_values,
        interpolation_type_names,
        interpolation_sampling_names,
        primitive_types,
        type_generators,
        type_aliases,
    }
}

pub fn extract_functions(html: &scraper::Html) -> BTreeMap<String, Function> {
    // collect builtin functions and type constructors
    let mut function_overloads = Vec::new();
    for builtin_tables in html.select(selector!(".data.builtin")) {
        let mut signature = None;
        let mut description = None;
        let mut parameterization = Parameterization::default();

        for row in builtin_tables.select(selector!("tr")) {
            let mut columns = row.select(selector!("td"));
            let Some(header) = columns.next() else { continue };
            let Some(value) = columns.next() else { continue };

            if header.text().any(|x| x.contains("Overload")) {
                signature = Some(join_words(value.text()));
            } else if header.text().any(|x| x.contains("Description")) {
                description = Some(join_words(value.text()));
            } else if header.text().any(|x| x.contains("Parameterization")) {
                parameterization = parse_parameterization(value);
            }
        }

        let Some(signature) = signature else { continue };
        let description = description.unwrap_or_default();

        function_overloads.push(FunctionOverload {
            signature,
            description: Some(description),
            parameterization,
        })
    }

    let mut functions = BTreeMap::<String, _>::new();
    for function in function_overloads {
        let name = function_name_from_signature(&function.signature)
            .expect("missing function name in signature");
        let builtin = functions.entry(name.into()).or_insert_with(|| Function {
            overloads: Vec::new(),
            parameters: Vec::new(),
            description: None,
        });
        builtin.overloads.push(function);
    }

    // texture functions use a different structure from the other functions:
    if let Some(texture_section) = html.select(selector!("#texture-builtin-functions")).next() {
        assert_eq!(texture_section.value().name(), "h3");

        let sections = texture_section
            .next_siblings()
            .filter_map(scraper::ElementRef::wrap)
            .take_while(|x| !matches!(x.value().name(), "h1" | "h2" | "h3"))
            .filter(|x| x.value().name() == "h4");

        for section in sections {
            let mut subsections = section
                .next_siblings()
                .filter_map(scraper::ElementRef::wrap)
                .take_while(|x| !matches!(x.value().name(), "h1" | "h2" | "h3" | "h4"));

            let name = join_words(
                section
                    .select(selector!("code"))
                    .next()
                    .expect("missing texture function name")
                    .text(),
            );

            let Some(description) = subsections.next() else { continue };
            let mut description = join_words(description.text());

            let mut overloads = Vec::new();
            let mut parameters = Vec::new();

            while let Some(subsection) = subsections.next() {
                for row in subsection.select(selector!("tr.algorithm")) {
                    let mut columns = row.select(selector!("td"));
                    let Some(parameterization) = columns.next() else { continue };
                    let Some(overload) = columns.next() else { continue };

                    let signature = join_words(overload.text());

                    overloads.push(FunctionOverload {
                        signature,
                        description: None,
                        parameterization: parse_parameterization(parameterization),
                    })
                }

                if let Some(heading) = subsection.select(selector!("p strong")).next() {
                    if heading.text().any(|x| x.contains("Parameters:")) {
                        let Some(parameter_table) = subsections.next() else { continue };
                        for row in parameter_table.select(selector!("tr")) {
                            let mut columns = row.select(selector!("td"));
                            let Some(name) = columns.next() else { continue };
                            let Some(description) = columns.next() else { continue };
                            parameters.push(FunctionParameter {
                                name: join_words(name.text()),
                                description: join_words(description.text()),
                            })
                        }
                    }

                    if heading.text().any(|x| x.contains("Returns:")) {
                        description += "\n\nReturns:";
                        for rest in subsections.by_ref() {
                            description += "\n\n";
                            description += &join_words(rest.text());
                        }
                    }
                }
            }

            functions
                .insert(name, Function { overloads, parameters, description: Some(description) });
        }
    }

    // atomic functions use a different structure from the other functions:
    if let Some(atomic_section) = html.select(selector!("#atomic-builtin-functions")).next() {
        assert_eq!(atomic_section.value().name(), "h3");

        let sections = atomic_section
            .next_siblings()
            .filter_map(scraper::ElementRef::wrap)
            .take_while(|x| !matches!(x.value().name(), "h1" | "h2" | "h3"))
            .filter(|x| x.value().name() == "h4");

        for section in sections {
            let mut children = section
                .next_siblings()
                .filter_map(scraper::ElementRef::wrap)
                .take_while(|x| !matches!(x.value().name(), "h1" | "h2" | "h3" | "h4"))
                .peekable();

            while let Some(signatures) = children.next() {
                assert_eq!(signatures.value().name(), "pre");

                let mut description = String::new();
                while let Some(x) = children.peek() {
                    if x.value().name() == "pre" {
                        break;
                    }
                    description += &join_words(x.text());
                    description += " ";
                    children.next();
                }

                while description.ends_with(char::is_whitespace) {
                    description.pop();
                }

                let signatures = join_words(signatures.text());
                let mut signatures = signatures.trim_start();
                while signatures.starts_with("fn") {
                    let mut end = 1;
                    while end < signatures.len()
                        && !(signatures[end..].starts_with("fn")
                            || signatures[end..].starts_with("struct"))
                    {
                        end += 1;
                    }

                    let (signature, rest) = signatures.split_at(end);
                    let signature = signature.trim();
                    signatures = rest;

                    let Some(name) = function_name_from_signature(signature) else { continue };

                    functions.insert(
                        name.into(),
                        Function {
                            overloads: vec![FunctionOverload {
                                signature: signature.into(),
                                parameterization: Parameterization::default(),
                                description: None,
                            }],
                            parameters: Vec::new(),
                            description: Some(description.clone()),
                        },
                    );
                }
            }
        }
    }

    // Remove functions which are only used as examples within the spec
    functions.remove("S");
    functions.remove("T");

    for function in functions.values_mut() {
        for overload in function.overloads.iter_mut() {
            overload.signature = overload.signature.replace(" < ", "<").replace(" >", ">");
        }
    }
    functions
}

fn parse_parameterization(element: scraper::ElementRef) -> Parameterization {
    let mut typevars = BTreeMap::new();

    let mut children = element.children().peekable();
    while children.peek().is_some() {
        let contents = children
            .by_ref()
            .take_while(|x| !matches!(x.value().as_element().map(|x| x.name()), Some("br" | "p")));

        let content_text = contents
            .filter_map(|x| match x.value() {
                scraper::Node::Text(text) => Some(text.text.to_string()),
                scraper::Node::Element(_) => {
                    let element = scraper::ElementRef::wrap(x).unwrap();
                    Some(join_words(element.text()))
                },
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");

        let words = content_text.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            continue;
        }
        let var_name = words[0];

        let kind = if words.get(1) == Some(&"is")
            && !matches!(words.get(2), Some(&"a" | &"an" | &"concrete" | &"constructible"))
        {
            let mut types = Vec::with_capacity(words.len());

            for word in words.iter().copied().skip(2) {
                if matches!(word, "," | "or") {
                    continue;
                }
                let word = word.trim_end_matches(',');
                types.push(word.into());
            }

            ParameterizationKind::Types(types)
        } else {
            ParameterizationKind::Description(join_words(words[1..].iter().copied()))
        };

        typevars.insert(var_name.to_string(), kind);
    }

    Parameterization { typevars }
}

fn join_words<'a>(chunks: impl Iterator<Item = &'a str> + Clone) -> String {
    let mut buffer = String::with_capacity(chunks.clone().map(|x| x.len()).sum());
    for chunk in chunks {
        for word in chunk.split_whitespace() {
            if !buffer.is_empty() && !matches!(word, "." | "," | ":" | ";") {
                buffer.push(' ');
            }
            buffer.push_str(word);
        }
    }
    while buffer.ends_with(char::is_whitespace) {
        buffer.pop();
    }
    buffer
}

fn function_name_from_signature(signature: &str) -> Option<&str> {
    tokenize(signature).skip_while(|x| *x != "fn").nth(1)
}

fn tokenize(mut text: &str) -> impl Iterator<Item = &str> + '_ {
    std::iter::from_fn(move || {
        text = text.trim_start();
        if text.is_empty() {
            return None;
        }

        let mut i = 0;
        let bytes = text.as_bytes();

        while i < bytes.len()
            && matches!(bytes[i], b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'@')
        {
            i += 1;
        }

        let (word, rest) = text.split_at(i);
        text = rest;

        Some(word)
    })
}
