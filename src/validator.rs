use std::collections::HashMap;

pub type ValidationError = String;
pub type ValidationResult = Option<ValidationError>;
pub type ValidatorResult = HashMap<String, Vec<ValidationError>>;

type ValidatorFn = fn(Option<&Vec<u8>>) -> ValidationResult;

pub struct Validator {
    validators: HashMap<String, Vec<ValidatorFn>>,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            validators: HashMap::with_capacity(8),
        }
    }

    pub fn field(&mut self, name: &str, validation: &[ValidatorFn]) -> &mut Self {
        self.validators
            .insert(name.to_owned(), validation.to_owned());
        self
    }

    pub fn check(&self, fields: &HashMap<String, Vec<u8>>) -> ValidatorResult {
        let mut errors = HashMap::new();

        for (field, validators) in self.validators.iter() {
            let value = fields.get(field);

            let result: Vec<String> = validators
                .iter()
                .filter_map(|validate| validate(value))
                .collect();

            if !result.is_empty() {
                errors.insert(field.to_owned(), result);
            }
        }

        errors
    }
}

#[allow(unused)]
pub mod v {
    use std::convert::TryInto;
    use std::str::from_utf8_unchecked;

    use regex::Regex;
    use strum::EnumVariantNames;
    use strum::VariantNames;

    use super::ValidationResult;

    pub fn required(input: Option<&Vec<u8>>) -> ValidationResult {
        match input {
            Some(input)
                if !unsafe { String::from_utf8_unchecked(input.clone()) }
                    .trim()
                    .is_empty() =>
            {
                None
            }
            _ => Some("Field is required".to_owned()),
        }
    }

    pub fn email(input: Option<&Vec<u8>>) -> ValidationResult {
        const EMAIL_REGEX: &'static str = r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#;

        let email_matcher = Regex::new(EMAIL_REGEX).unwrap();

        match input {
            Some(input) => match std::str::from_utf8(input) {
                Ok(input) => {
                    if email_matcher.is_match(input) {
                        None
                    } else {
                        Some("Email is not valid".to_owned())
                    }
                }
                _ => Some("Email is not valid utf-8".to_owned()),
            },
            None => None,
        }
    }

    pub fn url(input: Option<&Vec<u8>>) -> ValidationResult {
        match input {
            Some(input) => match std::str::from_utf8(input) {
                Ok(input) => match url::Url::parse(&input) {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                },
                _ => Some("URL is not valid utf-8".to_owned()),
            },
            None => None,
        }
    }

    pub fn one_of<S: VariantNames>(input: Option<&Vec<u8>>) -> ValidationResult {
        match input {
            Some(input) => match std::str::from_utf8(input) {
                Ok(input) => {
                    if S::VARIANTS.contains(&input) {
                        None
                    } else {
                        Some(format!(
                            "Expected a selection from: {}",
                            S::VARIANTS
                                .iter()
                                .copied()
                                .collect::<Vec<&str>>()
                                .join(", ")
                        ))
                    }
                }
                _ => Some("Selection is not valid utf-8".to_owned()),
            },
            None => None,
        }
    }

    pub fn length<const MIN: usize, const MAX: usize>(input: Option<&Vec<u8>>) -> ValidationResult {
        match input {
            Some(input) => {
                let length = input.len();

                if MAX == 0 {
                    if length < MIN {
                        Some(format!(
                            "Input too short, expected at least {} characters",
                            MIN
                        ))
                    } else {
                        None
                    }
                } else if MIN == 0 {
                    if length > MAX {
                        Some(format!(
                            "Input too long, expected at most {} characters",
                            MAX
                        ))
                    } else {
                        None
                    }
                } else if length < MIN {
                    Some(format!(
                        "Input too short, expected between {} and {} characters",
                        MIN, MAX
                    ))
                } else if length > MAX {
                    Some(format!(
                        "Input too long, expected between {} and {} characters",
                        MIN, MAX
                    ))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    #[allow(non_snake_case)]
    pub trait BinaryUnit: Sized + Into<usize> {
        fn get_binary_prefix(self) -> (usize, &'static str) {
            const UNITS: [&'static str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

            let mut prefix = 0;
            let mut value = self.into();
            while value > 1024 && prefix < 4 {
                prefix += 1;
                value /= 1024
            }

            (value, UNITS[prefix])
        }

        fn B(self) -> usize {
            self.into()
        }

        fn KiB(self) -> usize {
            KiB(self.into())
        }

        fn MiB(self) -> usize {
            MiB(self.into())
        }

        fn GiB(self) -> usize {
            GiB(self.into())
        }

        fn TiB(self) -> usize {
            TiB(self.into())
        }
    }

    impl<T: Sized + Into<usize>> BinaryUnit for T {}

    #[allow(non_snake_case)]
    pub const fn B(size: usize) -> usize {
        size
    }

    #[allow(non_snake_case)]
    pub const fn KiB(size: usize) -> usize {
        B(size) * 1024
    }

    #[allow(non_snake_case)]
    pub const fn MiB(size: usize) -> usize {
        KiB(size) * 1024
    }

    #[allow(non_snake_case)]
    pub const fn GiB(size: usize) -> usize {
        MiB(size) * 1024
    }

    #[allow(non_snake_case)]
    pub const fn TiB(size: usize) -> usize {
        GiB(size) * 1024
    }

    pub fn file_size<const MIN: usize, const MAX: usize>(
        input: Option<&Vec<u8>>,
    ) -> ValidationResult {
        match input {
            Some(input) => {
                let size = input.len();
                let (min, minpref) = MIN.get_binary_prefix();
                let (max, maxpref) = MAX.get_binary_prefix();

                if MAX == 0 {
                    if size < MIN {
                        Some(format!(
                            "File too small, expected at least {}{}",
                            min, minpref
                        ))
                    } else {
                        None
                    }
                } else if MIN == 0 {
                    if size > MAX {
                        Some(format!(
                            "File too large, expected at most {}{}",
                            max, maxpref
                        ))
                    } else {
                        None
                    }
                } else if size < MIN {
                    Some(format!(
                        "File too small, expected between {}{} and {}{}",
                        min, minpref, max, maxpref
                    ))
                } else if size > MAX {
                    Some(format!(
                        "File too large, expected between {}{} and {}{}",
                        min, minpref, max, maxpref
                    ))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn irange<const MIN: isize, const MAX: isize>(input: Option<&Vec<u8>>) -> ValidationResult {
        match input {
            Some(input) => {
                let input = match unsafe { from_utf8_unchecked(input) }.parse::<isize>() {
                    Ok(v) => v,
                    Err(_) => return Some(format!("Expected an integer")),
                };

                if input < MIN {
                    Some(format!(
                        "Value too small, expected between {} and {}",
                        MIN, MAX
                    ))
                } else if input > MAX {
                    Some(format!(
                        "Value too large, expected between {} and {}",
                        MIN, MAX
                    ))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn urange<const MIN: usize, const MAX: usize>(input: Option<&Vec<u8>>) -> ValidationResult {
        match input {
            Some(input) => {
                let input = match unsafe { from_utf8_unchecked(input) }.parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => return Some(format!("Expected an integer")),
                };

                if input < MIN {
                    Some(format!(
                        "Value too small, expected between {} and {}",
                        MIN, MAX
                    ))
                } else if input > MAX {
                    Some(format!(
                        "Value too large, expected between {} and {}",
                        MIN, MAX
                    ))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
