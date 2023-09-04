/// Gets supstring immidietly following the substring `p` in `s`
pub fn get_after<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let mut i = s.split(p);
    i.next();
    i.next()
}

/// Gets the next value from a iterator, returns error when there is no value.
/// The two last arguments are used to produce the error message
/// - the first argument says the option after which the error occured
/// - the second argument explains what value was expected
///
/// It can also parse and validate the value
///
/// # Usage
/// ```
/// // gets the next value from iterator
/// let val = next!(iterator);
///
/// // gets the value and parses it into f32
/// let val = next!(f32, iterator, None);
///
/// // gets the value, parses it into f32 and validates it
/// let val = next!(
///     f32,
///     iterator,
///     |v| (0.0..=1.).contains(v),
///     None
/// );
/// ```
#[macro_export]
macro_rules! next {
    ($iter:ident) => {
        match $iter.next() {
            Some(a) => a,
            None => {
                return Err(Error::UnexpectedEnd(None));
            }
        }
    };

    ($typ:ident, $iter:ident, $id:expr) => {
        $iter
            .next()
            .ok_or(Error::UnexpectedEnd)?
            .parse::<$typ>()
            .map_err(|_| Error::ParseError { id: $id, typ: stringify!($typ)})?
    };

    ($typ:ident, $iter:ident, $val:expr, $id:expr) => {
        $iter
            .next()
            .ok_or(Error::UnexpectedEnd)?
            .parse::<$typ>()
            .map_err(|_| Error::ParseError { id: $id, typ: stringify!($typ)})?
            .and_then(|v| {
                if { $val }() {
                    Ok(v)
                } else {
                    Err(Error::ParseError {
                        id: $id,
                        typ: stringify!($typ that satysfies: $val),
                    })
                }
            })?
    };
}

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

/// Parses the string value, returns error if it cannot be parsed. The second
/// argument is used to produce the error message
///
/// It can also validate the value
///
/// # Examples
/// ```
/// // parses the `&str` to `f32`
/// let val = parse!(f32, "3.1415", None);
///
/// // parses the `&str` to `f32` and validates it
/// let val = parse!(
///     f32,
///     "3.1415",
///     |v| (0.0..=1.).contains(v),
///     None
/// );
/// ```
#[macro_export]
macro_rules! parse {
    ($t:ty, $s:expr, $id:expr) => {
        $s
        .parse::<$t>()
        .map_err(|_| $crate::cli::Error::ParseError { id: $id, typ: stringify!($t)})?
    };

    ($t:ty, $s:expr, $val:expr, $id:expr) => {
        $s
            .parse::<$t>()
            .map_err(|_| Error::ParseError { id: $id, typ: stringify!($t)})
            .and_then(|v| {
                if { $val }(&v) {
                    Ok(v)
                } else {
                    Err(Error::ParseError {
                        id: $id,
                        typ: stringify!($typ that satysfies: $val),
                    })
                }
            })?
    };
}

/// Gets the value from a string parameter with `=`
///
/// # Examples
/// ```
/// let v = get_param!(f32, "vol=0.5");
///
/// let v = get_param!(f32, "vol=0.5", |v| (0.0..1.).contains(v)));
/// ```
#[macro_export]
macro_rules! get_param {
    ($t:ty, $v:expr) => {
        $crate::parse!(
            $t,
            $crate::cli::macros::get_after($v, "=").ok_or(
                $crate::cli::Error::MissingParameter(Some(format!("{}", $v)))
            )?,
            None
        )
    };

    ($t:ty, $v:expr, $val:expr) => {
        parse!(
            $t,
            get_after($v, "=")
                .ok_or(Error::MissingParameter(Some(format!("{}", $v))))?,
            $val,
            None
        )
    };
}

/// Gets the value from a string parameter with `=`, returns none if there is
/// no value
///
/// # Examples
/// ```
/// let v = may_get_param!(f32, "vol=0.5");
///
/// let v = may_get_param!(f32, "vol=0.5", |v| (0.0..1.).contains(v)));
/// ```
#[macro_export]
macro_rules! may_get_param {
    ($t:ty, $v:expr) => {
        match $crate::cli::macros::get_after($v, "=") {
            Some(v) => Some($crate::parse!($t, v, None)),
            None => None,
        }
    };

    ($t:ty, $v:expr, $val:expr) => {
        match get_after($v, "=") {
            Some(v) => Some(parse!($t, v, $val, None)),
            None => None,
        }
    };
}

/// Generates function that parses arguments and help for it.
///
/// # Usage:
/// ```
/// control_args! {
///     ? "optional help for the argument, can have {'y}colors{'_}"
///     "one-of-required-option" | "oro"
///         (=
///             "arg-value" | "av" -> type::generate(),
///             "arg2" -> type::generate2()
///         ) => ControlMsgVariant:
///             |v| type::optional_validator(v);
///
///     "one-of-optional-option" | "ooo"
///         {=
///             "arg-value" | "av" -> type::generate(),
///             "arg2" -> type::generate2()
///         } => ControlMsgVariant(type::optional_default_value):
///             |v| type::optional_validator(v);
///
///     "optional-option" [=type] =>
///         ControlMsgVariant(type::optional_default_value);
///
///     ? "optional documentation of required-option"
///     "required-option" | "ro" =type => ControlMsgVariant;
///
///     "just-flag" => ControlMsgVariant;
/// }
/// ```
#[macro_export]
macro_rules! parse_arg {
    ($pfun:ident, $hfun:ident: $(
        $(? $help:literal)?
        $($alias:literal)|+
        $( ( = $($($sel :literal)|+ -> $seldef :expr),+ ) )?
        $( { = $($($osel:literal)|+ -> $oseldef:expr),+ } )?
        $(   =     $rt :ty $( : $rtn:literal)?            )?
        $( [ =     $ot :ty $( : $otn:literal)?          ] )?
        => $msg:ident $(($def:expr))? $(: $val:expr)?
    );* $(;)?) => {place_macro::place! {
        pub fn $pfun(v: &str) -> $crate::cli::Result<$crate::core::msg::ControlMsg> {
            #[allow(unused_variables)]
            let s = v;

            #[allow(unused_parens)]
            let res = match v {
                $(
                    $(__ignore__($($seldef )+) v if $crate::starts!)?
                    $(__ignore__($($oseldef)+) v if $crate::starts!)?
                    $(__ignore__(  $rt       ) v if $crate::starts!)?
                    $(__ignore__(  $ot       ) v if $crate::starts!)?
                    (
                        $(__ignore__($($seldef )+) v,)?
                        $(__ignore__($($oseldef)+) v,)?
                        $(__ignore__(  $rt       ) v,)?
                        $(__ignore__(  $ot       ) v,)?
                        $($alias)|+
                    ) => {
                        #[allow(redundant_semicolons)]
                        $(let v = match get_after(v, "=") {
                            $(
                                Some($($sel)|+) => $seldef
                            ),+
                            _ => {
                                return Err(Error::ParseError {
                                    id: Some(v.to_owned()),
                                    typ: concat!($($($sel),+),+),
                                })
                            }
                        })?
                        $(let v = match $crate::cli::macros::get_after(v, "=") {
                            $(
                                Some($($osel)|+) => Some($oseldef),
                            )+
                            None => None,
                            _ => {
                                return Err($crate::cli::Error::ParseError {
                                    id: Some(v.to_owned()),
                                    typ: __str__(__start__($(__start__($($osel " or ")+) " or ")+)),
                                })
                            }
                        };)?
                        $(let v = $crate::get_param!($rt, v);)?
                        $(let v = $crate::may_get_param!($ot, v);)?

                        $(let v = v.unwrap_or($def);)?

                        $(
                            if !{ $val }(&v) {
                                return Err($crate::cli::Error::ParseError {
                                    id: Some(s.to_owned()),
                                    typ: __strfy__(value that satysfies $val),
                                })
                            }
                        )?

                        $crate::core::msg::ControlMsg::$msg
                        $(__ignore__($($seldef )+) (v.into()))?
                        $(__ignore__($($oseldef)+) (v.into()))?
                        $(__ignore__(  $rt       ) (v.into()))?
                        $(__ignore__(  $ot       ) (v.into()))?
                    }
                ),*
                _ => return Err($crate::cli::Error::UnknownArgument(Some(v.to_owned()))),
            };

            Ok(res)
        }

        pub fn $hfun() {
            termal::printc!(
                __str__(
                    $(
                        "{'y}"
                        $("  " $alias)+
                        "{'_}"
                        $(
                            "{'bold w}=<(",
                            __start__($($($sel "|")+)+)
                            ")>{'_}"
                        )?
                        $(
                            "{'gr}[=("
                            __start__($($($osel "|")+)+)
                            ")]{'_}"
                        )?
                        $(
                            "{'bold w}="
                            $($rtn __ignore__)?("<" __strfy__($rt) ">")
                            "{'_}"
                        )?
                        $(
                            "{'gr}[="
                            $($otn __ignore__)?("<" __strfy__($ot) ">")
                            "]{'_}"
                        )?
                        $("\n    " __repnl__($help, "\n    "))?
                        "\n\n",
                    )+
                )
            );
        }
    }};
}
