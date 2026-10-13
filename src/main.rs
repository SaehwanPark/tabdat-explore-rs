#![forbid(unsafe_code)]

pub use tabdat_shell::{catalog, cli, help, json_results};

fn main() {
  let args = std::env::args_os()
    .map(|os| os.to_string_lossy().into_owned())
    .collect::<Vec<_>>();
  let exit_code = match cli::parse_args(args.into_iter().skip(1)) {
    Ok(cli_args) => cli::run_cli(cli_args),
    Err(err) => {
      use std::io::Write;
      let mut errors = std::io::stderr().lock();
      match writeln!(errors, "{}\ntabdat: error: {err}", cli::USAGE).and_then(|_| errors.flush()) {
        Ok(()) => 2,
        Err(_) => 1,
      }
    }
  };
  if exit_code != 0 {
    std::process::exit(exit_code);
  }
}
