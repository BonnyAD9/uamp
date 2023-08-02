use crate::messenger;
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
        "play-pause" | "pp" => {
            res.actions
                .push(Action::Message(messenger::Message::Control(
                    messenger::Control::PlayPause,
                )))
        }
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
    println!("Welcome in uamp by BonnyAD9\nVersion 0.1.0\n")
}

fn print_basic_help() {
    println!(
        "Usage:
  uamp
    starts the gui of the player

  uamp [action]
    does the given action

Actions:
  i instance [instance action]
    operates on a running instance of uamp

  h help -h -? --help
    shows help, with no argument whole help, with arguments only help specific
    to the given option.
    Available options are: basic, i instance
"
    )
}

fn print_instance_help() {
    println!(
        "Instance actions:
  pp play-pause
    toggle between the states playing and paused
"
    )
}
