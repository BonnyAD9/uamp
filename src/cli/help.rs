use termal::{gradient, printcln};

use super::Result;

use super::{parsers::auto_instance_help, Args};

/// Parses help arguments
pub fn help<'a>(
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
