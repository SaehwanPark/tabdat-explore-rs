#![forbid(unsafe_code)]

pub mod catalog;
pub mod cli;
pub mod help;

fn main() {
  let args = std::env::args_os()
    .map(|os| os.to_string_lossy().into_owned())
    .collect::<Vec<_>>();
  let exit_code = match cli::parse_args(args.into_iter().skip(1)) {
    Ok(cli_args) => cli::run_cli(cli_args),
    Err(err) => {
      eprintln!("{}", cli::USAGE);
      eprintln!("tabdat: error: {err}");
      2
    }
  };
  if exit_code != 0 {
    std::process::exit(exit_code);
  }
}
