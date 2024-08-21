use nazmc_diagnostics_macros::nazmc_error_code;
use crate::ErrorLevel;
use crate::DiagnosticLevel;

// Derive error level for theses class and append codes to them

#[cfg(test)]
mod tests {
  use super::*;

  // my tests

  #[test]
  fn it_works(){
    assert_eq!(E0000::CODE, E0000::NAME);
    assert_eq!(E0001::CODE, E0001::NAME);
    assert_eq!(E0002::CODE, E0002::NAME);
    println!("{}", E0000::CODE);
    println!("{}", E0001::CODE);
    println!("{}", E0002::CODE);
  }
}

#[nazmc_error_code]
struct E0000;

#[nazmc_error_code]
struct E0001;

#[nazmc_error_code]
struct E0002;
