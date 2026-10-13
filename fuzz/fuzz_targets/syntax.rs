#![no_main]
#![forbid(unsafe_code)]
use libfuzzer_sys::fuzz_target;
use std::path::Path;
use tabdat_language::{parse_command, parse_script, tokenize};

fuzz_target!(|input: &str| {
  assert_eq!(parse_command(input), parse_command(input));
  assert_eq!(tokenize(input), tokenize(input));
  assert_eq!(
    parse_script(input, Path::new("fuzz.td")),
    parse_script(input, Path::new("fuzz.td"))
  );
});
