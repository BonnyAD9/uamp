use std::{time::Duration, str::FromStr, num::ParseFloatError};

use crate::{messenger, uamp_app::ControlMsg};
use anyhow::anyhow;
use itertools::Itertools;
use termal::{gradient, printcln};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Action that can be done with cli
pub enum Action {
    Message(messenger::Message),
}

/// Contains the CLI arguments values
#[derive(Default)]
pub struct Args {
    pub actions: Vec<Action>,
    pub should_exit: bool,
    pub should_run: bool,
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
macro_rules! starts {
    ($i:ident, $($s:literal)|+) => {{
        matches!($i, $($s)|+) || $($i.starts_with(concat!($s, "=")))||+
    }};
}

/// Adds control message as action to be sent to existing instance
macro_rules! msg_control {
    ($arg:ident, $msg:ident) => {
        $arg.actions
            .push(Action::Message(messenger::Message::Control(
                ControlMsg::$msg,
            )))
    };

    ($arg:ident, $msg:ident $t:tt) => {
        $arg.actions
            .push(Action::Message(messenger::Message::Control(
                ControlMsg::$msg$t,
            )))
    };
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
macro_rules! parse {
    ($t:ty, $s:expr, $id:expr) => {
        $s
        .parse::<$t>()
        .map_err(|_| Error::ParseError { id: $id, typ: stringify!($t)})?
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
macro_rules! get_param {
    ($t:ty, $v:expr) => {
        parse!(
            $t,
            get_after($v, "=")
                .ok_or(Error::MissingParameter(Some(format!("{}", $v))))?,
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
macro_rules! may_get_param {
    ($t:ty, $v:expr) => {
        match get_after($v, "=") {
            Some(v) => Some(parse!($t, v, None)),
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

/// Gets supstring immidietly following the substring `p` in `s`
fn get_after<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let mut i = s.split(p);
    i.next();
    i.next()
}

/// Parses the CLI arguments and returns the parsed arguments
pub fn parse_args<'a>(args: impl Iterator<Item = &'a str>) -> Result<Args> {
    let mut res = Args::default();

    let mut args = args.skip(1);

    while let Some(a) = args.next() {
        match a {
            "i" | "instance" => instance(&mut args, &mut res)?,
            "h" | "help" | "-h" | "--help" | "-?" => {
                help(&mut args, &mut res)?
            }
            a => return Err(Error::UnknownArgument(Some(a.to_owned()))),
        }
    }

    Ok(res)
}

/// Parses the instance action arguments
fn instance<'a>(
    args: &mut impl Iterator<Item = &'a str>,
    res: &mut Args,
) -> Result<()> {
    res.should_exit = true;
    let a = next!(args);

    if a == "--" {
        return Err(Error::UnexpectedEnd(Some("instance".to_owned())));
    }

    let msg = parse_control_message(a)?;
    res.actions
        .push(Action::Message(messenger::Message::Control(msg)));

    Ok(())
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
macro_rules! control_args {
    ($(
        $(? $help:literal)?
        $($alias:literal)|+
        $( ( = $($($sel :literal)|+ -> $seldef :expr),+ ) )?
        $( { = $($($osel:literal)|+ -> $oseldef:expr),+ } )?
        $(   =     $rt :ty $( : $rtn:literal)?            )?
        $( [ =     $ot :ty $( : $otn:literal)?          ] )?
        => $msg:ident $(($def:expr))? $(: $val:expr)?
    );* $(;)?) => {place_macro::place! {
        pub fn parse_control_message(v: &str) -> Result<ControlMsg> {
            #[allow(unused_variables)]
            let s = v;

            #[allow(unused_parens)]
            let res = match v {
                $(
                    $(__ignore__($($seldef )+) v if starts!)?
                    $(__ignore__($($oseldef)+) v if starts!)?
                    $(__ignore__(  $rt       ) v if starts!)?
                    $(__ignore__(  $ot       ) v if starts!)?
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
                        $(let v = match get_after(v, "=") {
                            $(
                                Some($($osel)|+) => Some($oseldef),
                            )+
                            None => None,
                            _ => {
                                return Err(Error::ParseError {
                                    id: Some(v.to_owned()),
                                    typ: __str__(__start__($(__start__($($osel " or ")+) " or ")+)),
                                })
                            }
                        };)?
                        $(let v = get_param!($rt, v);)?
                        $(let v = may_get_param!($ot, v);)?

                        $(let v = v.unwrap_or($def);)?

                        $(
                            if !{ $val }(&v) {
                                return Err(Error::ParseError {
                                    id: Some(s.to_owned()),
                                    typ: __strfy__(value that satysfies $val),
                                })
                            }
                        )?

                        ControlMsg::$msg
                        $(__ignore__($($seldef )+) (v.into()))?
                        $(__ignore__($($oseldef)+) (v.into()))?
                        $(__ignore__(  $rt       ) (v.into()))?
                        $(__ignore__(  $ot       ) (v.into()))?
                    }
                ),*
                _ => return Err(Error::UnknownArgument(Some(v.to_owned()))),
            };

            Ok(res)
        }

        fn auto_instance_help() {
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
                            "{'gr}[=<"
                            $($otn __ignore__)?(__strfy__($ot))
                            ">]{'_}"
                        )?
                        $("\n    " __repnl__($help, "\n    "))?
                        "\n\n",
                    )+
                )
            );
        }
    }};
}

control_args! {
    ? "Play or pause, when without argument, toggle between the states
       playing and paused."
    "play-pause" | "pp" {= "play" -> true, "pause" -> false} => PlayPause;

    ? "Increase the volume by the given amount. If the parameter is not
       present, increase by the default amount"
    "volume-up" | "vol-up" | "vu" [=f32] => VolumeUp;

    ? "Decrease the volume by the given amount. If the parameter is not
       present, decrease by the default amount"
    "volume-down" | "vol-down" | "vd" [=f32] => VolumeDown;

    ? "Jump to the next song, arguments specifies how much to jump (e.g.
       with argument '2' skips one song and plays the next)."
    "next-song" | "ns" [=usize] => NextSong(1);

    ? "Jump to the previous song, arguments specifies how much to jump
       (e.g. with argument '2' skips the previous song and plays the
       second previous song)."
    "previous-song" | "ps" [=usize] => PrevSong(1);

    ? "Set the volume to the given value. Value must be in range from 0 to 1"
    "volume" | "vol" | "v" =f32 => SetVolume: |v| (0.0..0.).contains(v);

    ? "Mute/Unmute, if the argument is not specified, toggles between
       the states"
    "mute" [=bool] => Mute;

    ? "Look for new songs."
    "load-songs" => LoadNewSongs;

    ? "Shuffles the current playlist."
    "shuffle-playlist" | "shuffle" => Shuffle;

    ? "Exits the instance"
    "exit" | "close" | "x" => Close;

    ? "Seeks to the given timestamp. Timestamp is in format 'h:m:s'."
    "seek-to" | "seek" =ParsableDuration: "{'_ gr}[[[<h>]:][<m>]:][<s>[.<s>]]"
        => SeekTo;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "Expected more arguments{} (the last argument requires that more follow)",
        if let Some(i) = .0 { format!(" after '{}'", i) } else { "".to_owned() }
    )]
    UnexpectedEnd(Option<String>),
    #[error(
        "Failed to parse argument{}, the argument must be {typ}",
        if let Some(i) = .id { i.as_str() } else { "" }
    )]
    ParseError {
        id: Option<String>,
        typ: &'static str,
    },
    #[error("Unknown option{}", .0.as_ref().map(|i| i.as_str()).unwrap_or(""))]
    UnknownArgument(Option<String>),
    #[error(
        "Missing parameter{}",
        if let Some(i) = .0 { format!(" for argument '{}'", i) } else { "".to_owned() })]
    MissingParameter(Option<String>),
}

//==================================<< HELP >>===============================//

/// Parses help arguments
fn help<'a>(
    args: &mut impl Iterator<Item = &'a str>,
    res: &mut Args,
) -> Result<()> {
    res.should_exit = true;

    let arg = match args.next() {
        Some(a) => a,
        None => {
            print_help();
            return Ok(());
        }
    };

    if arg == "--" {
        print_help();
        return Ok(());
    }

    // prepend the first argument back to a iterator
    let arg = [arg];
    let args = arg.iter().map(|s| *s).chain(args);

    print_help_header();

    for arg in args {
        match arg {
            "basic" => print_basic_help(),
            "i" | "instance" => print_instance_help(),
            a => println!("Invalid help option {a}"),
        }
    }

    Ok(())
}

/// Prints all of help
fn print_help() {
    print_help_header();
    print_basic_help();
    print_instance_help();
}

/// Prints the help header
fn print_help_header() {
    let v: Option<&str> = option_env!("CARGO_PKG_VERSION");
    printcln!(
        "Welcome in {'i g}uamp{'_} by {}{'_}
Version {}
",
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240)),
        v.unwrap_or("unknown")
    )
}

/// Prints the basic usage help
fn print_basic_help() {
    printcln!(
        "{'g}Usage:{'_}
  {'w bold}uamp{'_}
    starts the gui of the player

  {'w bold}uamp{'_} {'gr}[action] [--] [action] ...{'_}
    does the given action

{'g}Actions:{'_}
  {'y}i  instance {'bold w}<instance-action>{'_} {'gr}[--]{'_}
    operates on a running instance of uamp

  {'y}h  help  -h  -?  --help{'_}
    shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: {'bold w}basic{'_}, {'bold w}i instance{'_}
"
    )
}

/// Prints the instance help
fn print_instance_help() {
    printcln!("{'g}Instance actions:");
    auto_instance_help();
}

struct ParsableDuration(Duration);

impl From<Duration> for ParsableDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl Into<Duration> for ParsableDuration {
    fn into(self) -> Duration {
        self.0
    }
}

impl FromStr for ParsableDuration {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = s.split(':').collect_vec();
        let (h, m, s) = match r.len() {
            0 => return Err(anyhow!("Imput string is empty.")),
            1 => ("", "", r[0]),
            2 => ("", r[0], r[1]),
            3 => (r[0], r[1], r[2]),
            _ => return Err(anyhow!("Too many colons")),
        };

        let mut res = if s.is_empty() {
            Duration::ZERO
        } else {
            Duration::from_secs_f32(f32::from_str(s)?)
        };

        res += if m.is_empty() {
            Duration::ZERO
        } else {
            Duration::from_secs(u64::from_str(m)? * 60)
        };

        res += if h.is_empty() {
            Duration::ZERO
        } else {
            Duration::from_secs(u64::from_str(h)? * 3600)
        };

        Ok(res.into())
    }
}
