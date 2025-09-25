#[macro_export]
macro_rules! green {
    ($text:expr) => {
        format!("\x1b[32m{}\x1b[0m", $text)
    };
}

#[macro_export]
macro_rules! red {
    ($text:expr) => {
        format!("\x1b[31m{}\x1b[0m", $text)
    };
}

#[macro_export]
macro_rules! impl_from {
    ($from_ty:ty => $to_enum:ident::$variant:ident $( : $into:ident )? ) => {
        impl From<$from_ty> for $to_enum {
            fn from(err: $from_ty) -> Self {
                $to_enum::$variant(
                    impl_from!(@maybe_into err $( $into )?)
                )
            }
        }
    };

    (@maybe_into $val:ident into) => { $val.into() };
    (@maybe_into $val:ident) => { $val };
}
