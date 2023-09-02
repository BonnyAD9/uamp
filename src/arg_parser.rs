use crate::{messenger, uamp_app::ControlMsg};
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

macro_rules! control_args {
    ($(
        $(?$help:literal)?
        $($alias:literal)|+
        $( ( $($oneof   :ident)? = $($($sel :literal)|+ -> $seldef :expr),+ ) )?
        $( { $($optoneof:ident)? = $($($osel:literal)|+ -> $oseldef:expr),+ } )?
        $(   $($req     :ident)? =     $rt :ty                                )?
        $( [ $($opt     :ident)? =     $ot :ty                              ] )?
        => $msg:ident $(($def:expr))? $(: $val:expr)?
    );* $(;)?) => {
        pub fn parse_control_message(v: &str) -> Result<ControlMsg> {
            #[allow(unused_variables)]
            let s = v;
            #[allow(unused_parens)]
            let res = match v {
                $(
                    $($($oneof   )? v if starts!)?
                    $($($req     )? v if starts!)?
                    $($($opt     )? v if starts!)?
                    $($($optoneof)? v if starts!)?
                    (
                        $($($oneof   )? v, )?
                        $($($req     )? v, )?
                        $($($opt     )? v, )?
                        $($($optoneof)? v, )?
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
                                    typ: concat!($($($osel),+),+),
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
                                    typ: stringify!(value that satysfies $val),
                                })
                            }
                        )?

                        ControlMsg::$msg
                        $($($oneof   )? (v) )?
                        $($($req     )? (v) )?
                        $($($opt     )? (v) )?
                        $($($optoneof)? (v) )?
                    }
                ),*
                _ => return Err(Error::UnknownArgument(Some(v.to_owned()))),
            };

            Ok(res)
        }

        /*fn auto_instance_help() {
            print!(
                concat!(

                )
            );
        }*/
    };
}

control_args! {
    "play-pause" | "pp" {= "play" -> true, "pause" -> false} => PlayPause;

    "volume-up" | "vol-up" | "vu" [=f32] => VolumeUp(1.);

    "volume-down" | "vol-down" | "vd" [=f32] => VolumeDown(1.);

    "next-song" | "ns" [=usize] => NextSong(1);

    "previous-song" | "ps" [=usize] => PrevSong(1);

    "volume" | "vol" | "v" =f32 => SetVolume: |v| (0.0..0.).contains(v);

    "mute" [=bool] => Mute;

    "load-songs" => LoadNewSongs;

    "shuffle-playlist" | "shuffle" => Shuffle;

    "exit" | "close" | "x" => Close;
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
    printcln!(
        "{'g}Instance actions:
  {'y}pp  play-pause{'gr}[=(play | pause)]{'_}
    play or pause, when without argument, toggle between the states playing and
    paused

  {'y}vu  vol-up  volume-up{'gr}[=<mul>]{'_}
    increase the volume by the default amount, when {'bold w}mul{'_} is
    specified, multiply the volume increase by that number

  {'y}vd  vol-down  volume-down{'gr}[=<mul>]{'_}
    decrease the volume by the default amount, when {'bold w}mul{'_} is
    specified, multiply the volume decrease by that number

  {'y}ns  next-song{'gr}[=<N>]{'_}
    jump to the {'bold w}N{'_}th next song in the playlist. By default,
    {'bold w}N{'_} is 1.

  {'y}ps  prev-song  previous-song{'gr}[=<N>]{'_}
    jump to the {'bold w}N{'_}th previous song in the playlist. By default,
    {'bold w}N{'_} is 1.

  {'y}v  vol  volume{'bold w}=<value>{'_}
    set the volume to the given {'bold w}value{'_}, {'bold w}value{'_} must be
    in range from 0 to 1

  {'y}mute{'gr}[=(true | false)]{'_}
    mute/unmute, if the value is not specified, toggles between the states

  {'y}load-songs{'_}
    look for new songs

  {'y}shuffle  shuffle-playlist{'_}
    shuffles the current playlist

  {'y}pj  playlist-jump{'bold w}=<index>{'_}
    jumps to the given {'bold w}index{'_} in playlist, stops the playback if
    the {'bold w}index{'_} is out of range

  {'y}x  exit  close{'_}
    exits the instance
"
    )
}
