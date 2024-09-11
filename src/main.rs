use core::str;
use std::{env, error::Error};

const HELP_MESSAGE: &str = "
Commands: 
    - host [port=8629]              host a server on optional [port]
    - join <address> [port=8629]    join a server on <address> via optional [port]
";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mode: &str = args.get(1).expect(HELP_MESSAGE);

    match mode {
        "join" => under_control_ler::join(&args),
        "host" => under_control_ler::host(&args),
        _ => {}
    }
    dbg!(&args);

    Ok(())
}
