/// creates expression that checks whether a variable starts with any of the
/// strings
///
/// # Example
/// ```
/// let val = "arg2=hi";
/// if starts!(val, "arg1" | "arg2") {
///     // now we know that `val` starts either with `"arg1"` or `"arg2"`
/// }
/// ```
#[macro_export]
macro_rules! starts {
    ($i:ident, $($s:literal)|+) => {{
        matches!($i, $($s)|+) || $($i.starts_with(concat!($s, "=")))||+
    }};
}
