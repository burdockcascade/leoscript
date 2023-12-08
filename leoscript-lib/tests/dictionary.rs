use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use leoscript_lib::common::variant::Variant;
use leoscript_lib::run_script;

mod common;

#[test]
pub fn test_dictionary() {

    let _ = TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto);

    test_success!(r#"
        function main()
            var d = new Dictionary()
            d.set("a", 1)
            d.set("b", 2)
            d.set("c", 3)
            return d.get("a") == 1
        end
    "#);
}