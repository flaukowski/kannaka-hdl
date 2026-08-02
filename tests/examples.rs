//! Every shipped .khdl example must parse and grow — a grammar change can
//! never silently break the files users copy first.

use kannaka_hdl::{grow::grow, parser::parse};

#[test]
fn every_shipped_example_parses_and_grows() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    let mut grown = 0;
    for entry in std::fs::read_dir(dir).expect("examples dir") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("khdl") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        let program =
            parse(&src).unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()));
        let plan =
            grow(&program).unwrap_or_else(|e| panic!("{} failed to grow: {e}", path.display()));
        assert!(!plan.leaves.is_empty(), "{} grew no leaves", path.display());
        grown += 1;
    }
    assert!(grown >= 3, "expected the shipped examples, found {grown}");
}
