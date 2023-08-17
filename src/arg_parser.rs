use crate::{messenger, uamp_app::ControlMsg};
use eyre::{Report, Result};

pub enum Action {
    Message(messenger::Message),
}

#[derive(Default)]
pub struct Args {
    pub actions: Vec<Action>,
    pub should_exit: bool,
    pub should_run: bool,
}

macro_rules! ret_err {
    ($($arg:tt)*) => {
        return Err(Report::msg(format!($($arg)*)))
    };
}

macro_rules! next {
    ($iter:ident, $after:literal, $what:literal) => {
        match $iter.next() {
            Some(a) => a,
            None => {
                ret_err!("Expected {} action after '{}'", $what, $after)
            }
        }
    };

    ($typ:ident, $iter:ident, $after:literal, $what:literal) => {
        match $iter
            .next()
            .ok_or(Report::msg("next is none"))
            .and_then(|v| v.parse::<$typ>().map_err(|e| e.into()))
        {
            Ok(a) => a,
            Err(_) => {
                ret_err!("Expected {} action after '{}'", $what, $after)
            }
        }
    };

    ($typ:ident, $iter:ident, $after:literal, $what:literal, $val:expr) => {
        match $iter
            .next()
            .ok_or(Report::msg("next is none"))
            .and_then(|v| v.parse::<$typ>().map_err(|e| e.into()))
        {
            Ok(a) if { $val }(&a) => a,
            _ => {
                ret_err!("Expected {} action after '{}'", $what, $after)
            }
        }
    };
}

macro_rules! starts {
    ($i:ident, $($s:literal)|+) => {
        $($i.starts_with($s))||+
    };
}

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

macro_rules! parse {
    ($t:ty, $msg:literal, $s:expr) => {
        match $s
            .ok_or(Report::msg("none"))
            .and_then(|s| s.parse::<$t>().map_err(|e| e.into()))
        {
            Ok(a) => a,
            _ => ret_err!("Expected {}", $msg),
        }
    };

    ($t:ty, $msg:literal, $s:expr, $val:expr) => {
        match $s
            .ok_or(Report::msg("none"))
            .and_then(|s| s.parse::<$t>().map_err(|e| e.into()))
        {
            Ok(a) if { $val }(&a) => a,
            _ => ret_err!("Expected {}", $msg),
        }
    };
}

macro_rules! maybe_parse {
    ($t:ty, $msg:literal, $s:expr) => {
        match $s.map(|s| s.parse::<$t>()) {
            Some(Err(_)) => ret_err!("Expected {}", $msg),
            Some(Ok(a)) => Some(a),
            _ => None,
        }
    };

    ($t:ty, $msg:literal, $s:expr, $val:expr) => {
        match $s.map(|s| s.parse::<$t>()) {
            Some(Err(_)) => ret_err!("Expected {}", $msg),
            Some(Ok(a)) if { $val }(&a) => Some(a),
            None => None,
            _ => ret_err!("Expected {}", $msg),
        }
    };
}

fn get_after<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let mut i = s.split(p);
    i.next();
    i.next()
}

pub fn parse_args<'a>(args: impl Iterator<Item = &'a str>) -> Result<Args> {
    let mut res = Args::default();

    let mut args = args.skip(1);

    while let Some(a) = args.next() {
        match a {
            "i" | "instance" => instance(&mut args, &mut res)?,
            "h" | "help" | "-h" | "--help" | "-?" => {
                help(&mut args, &mut res)?
            }
            a => ret_err!("Invalid argument '{a}'"),
        }
    }

    Ok(res)
}

fn instance<'a>(
    args: &mut impl Iterator<Item = &'a str>,
    res: &mut Args,
) -> Result<()> {
    res.should_exit = true;
    let a = next!(args, "instance", "instance action");

    match a {
        "play-pause" | "pp" => msg_control!(res, PlayPause),
        "volume-up" | "vol-up" | "vu" => msg_control!(res, VolumeUp),
        "volume-down" | "vol-down" | "vd" => msg_control!(res, VolumeDown),
        "next-song" | "ns" => msg_control!(res, NextSong),
        "previous-song" | "prev-song" | "ps" => msg_control!(res, PrevSong),
        v if starts!(v, "volume" | "vol" | "v") => {
            let vol = parse!(
                f32,
                "expected float in range 0..=1",
                get_after(v, "="),
                |v| (0.0..=1.).contains(v)
            );
            msg_control!(res, SetVolume(vol));
        }
        "mute" => msg_control!(res, ToggleMute),
        "find-songs" | "fs" => msg_control!(res, FindSongs),
        "--" => ret_err!("Expected instance action after 'instance'"),
        _ => ret_err!("Invalid argument '{a}' after 'instance'"),
    }

    Ok(())
}

//==================================<< HELP >>===============================//

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

fn print_help() {
    print_help_header();
    print_basic_help();
    print_instance_help();
}

fn print_help_header() {
    println!("Welcome in uamp by BonnyAD9\nVersion 0.1.1\n")
}

fn print_basic_help() {
    println!(
        "Usage:
  uamp
    starts the gui of the player

  uamp [action] [--] [action] ...
    does the given action

Actions:
  i  instance <instance-action> [--]
    operates on a running instance of uamp

  h  help  -h  -?  --help
    shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: basic, i instance
"
    )
}

fn print_instance_help() {
    println!(
        "Instance actions:
  pp  play-pause
    toggle between the states playing and paused

  vu  vol-up  volume-up
    increase the volume by the default amount

  vd  vol-down  volume-down
    decrease the volume by the default amount

  ns  next-song
    go to the next song

  ps  prev-song  previous-song
    go to the previous song

  v  vol  volume=VALUE
    set the volume to the given VALUE, VALUE must be in range from 0 to 1

  mute
    toggle mute/unmute

  fs  find-songs
    look for new songs
"
    )
}
