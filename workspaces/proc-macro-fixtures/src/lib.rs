use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, Token};

// input a map of key value pairs, and return tuples
// that have nested stuff inside.
// we can then call a method called "into_dir"

#[derive(PartialEq, Debug, Clone)]
enum Fixture {
    File {
        name: PathBuf,
        contents: String,
    },
    Directory {
        name: PathBuf,
        fixtures: Vec<Fixture>,
    },
}

impl Fixture {
    fn join_to_right<P: AsRef<Path>>(self, path: P) -> Self {
        match self {
            Self::File { name, contents } => Self::File {
                name: name.join(path),
                contents,
            },
            Self::Directory { name, fixtures } => Self::Directory {
                name: name.join(path),
                fixtures,
            },
        }
    }

    fn join_to_left<P: AsRef<Path>>(self, path: P) -> Self {
        match self {
            Self::File { name, contents } => Self::File {
                name: path.as_ref().join(name),
                contents,
            },
            Self::Directory { name, fixtures } => Self::Directory {
                name: path.as_ref().join(name),
                fixtures,
            },
        }
    }

    fn to_files(self) -> Vec<(PathBuf, String)> {
        match self {
            Self::File { name, contents } => vec![(name, contents)],
            Self::Directory { name, fixtures } => fixtures
                .into_iter()
                .map(|fixture| fixture.join_to_left(name.to_owned()))
                .flat_map(|fixture| fixture.to_files())
                .collect(),
        }
    }
}

enum Input {
    Pair(Expr, Expr),
}

impl Input {
    fn pair(input: ParseStream) -> Result<Input> {
        let first: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let second: Expr = input.parse()?;
        let result = Self::Pair(first, second);
        Ok(result)
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::pair(input)
    }
}

#[proc_macro]
pub fn fixtures(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    let expanded = match input {
        Input::Pair(first, second) => quote! {
            vec![(#first, #second)]
        },
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod test {
    use super::Fixture;

    mod join_to_left {
        use super::Fixture;
        use std::path::PathBuf;

        #[test]
        fn should_join_to_left_on_file() {
            let name = "name";
            let contents = "contents";
            let fixture = Fixture::File {
                name: PathBuf::from(name),
                contents: contents.to_owned(),
            };
            let parent = "parent".to_string();
            let fixture = fixture.join_to_left(parent.to_owned());
            let expected = Fixture::File {
                name: PathBuf::from(parent + "/" + name),
                contents: contents.to_owned(),
            };
            assert_eq!(fixture, expected);
        }

        #[test]
        fn should_join_to_left_on_directory() {
            let name = "name";
            let fixtures = vec![];
            let fixture = Fixture::Directory {
                name: PathBuf::from(name),
                fixtures: fixtures.clone(),
            };
            let parent = "parent".to_string();
            let fixture = fixture.join_to_left(parent.to_owned());
            let expected = Fixture::Directory {
                name: PathBuf::from(parent + "/" + name),
                fixtures,
            };
            assert_eq!(fixture, expected);
        }
    }

    mod to_files {
        use super::Fixture;

        #[test]
        fn should_prepend_names_correctly() {
            let fixture = Fixture::Directory {
                name: "first".into(),
                fixtures: vec![
                    Fixture::File {
                        name: "second".into(),
                        contents: "".into(),
                    },
                    Fixture::Directory {
                        name: "third".into(),
                        fixtures: vec![Fixture::File {
                            name: "fourth".into(),
                            contents: "".into(),
                        }],
                    },
                ],
            };

            let result = fixture.to_files();

            let expected = vec![
                ("first/second".into(), "".into()),
                ("first/third/fourth".into(), "".into()),
            ];

            assert_eq!(result, expected);
        }
    }
}
