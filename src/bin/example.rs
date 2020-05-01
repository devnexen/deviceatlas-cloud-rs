extern crate dacloud;
extern crate clap;
use clap::{Arg, App, ArgMatches};

pub fn main() {
    let app: App<'static, 'static>;
    let args: ArgMatches<'static>;

    app = App::new("example")
               .arg(Arg::with_name("licence_key")
                    .short("l")
                    .long("licence_key")
                    .takes_value(true)
                    .required(true)
                    .help("Licence key"))
               .arg(Arg::with_name("host")
                    .short("h")
                    .long("host")
                    .takes_value(true)
                    .help("Host"))
               .arg(Arg::with_name("user_agent")
                    .short("u")
                    .long("user_agent")
                    .takes_value(true)
                    .required(true)
                    .help("User-Agent"));

    args = app.get_matches();
    let host = args.value_of("host").unwrap_or("http://region0.deviceatlascloud.com");
    let licence_key = args.value_of("licence_key").unwrap();
    let user_agent = args.value_of("user_agent").unwrap();
    let cfg = dacloud::Config::new(host.to_string(), licence_key.to_string(), 0 as usize);
    let mut inst = dacloud::Dacloud::new(cfg);
    inst.headers.insert(String::from("user-agent"), String::from(user_agent));
    let props = inst.req();
    println!("{} properties", props.len());

    for (k, v) in props {
        println!("{}: {}", k, v.to_string());
    }
}
