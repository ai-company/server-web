#[macro_export]
macro_rules! basic_error {
    ( $name:ident, $reason: tt ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, $reason)
            }
        }
        impl std::error::Error for $name {}
    };
}

#[macro_export]
macro_rules! basic_error_with_message {
    ( $name:ident, $reason: tt ) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            Static(&'static str),
            String(String),
        }

        #[allow(dead_code)]
        impl $name {
            pub fn static_message(message: &'static str) -> $name {
                $name::Static(message)
            }

            pub fn string_message(message: String) -> $name {
                $name::String(message)
            }
        }

        impl<'a> std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                use $name::*;
                match self {
                    Static(msg) => write!(f, "{}:{}", $reason, msg),
                    String(msg) => write!(f, "{}:{}", $reason, msg),
                }
            }
        }
        impl<'a> std::error::Error for $name {}
    };
}

#[macro_export]
macro_rules! basic_error_with_dyn_message {
    ( $name:ident, $reason: tt ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            msg: String,
        }

        impl $name {
            pub fn new(message: String) -> $name {
                $name { msg: message }
            }
        }

        impl<'a> std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}:{}", $reason, self.msg)
            }
        }
        impl<'a> std::error::Error for $name {}
        // impl crate::DynError for $name {}
    };
}
