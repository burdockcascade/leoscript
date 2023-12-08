use leoscript_lib::common::variant::Variant;
use leoscript_lib::run_script;

mod common;

#[test]
pub fn test_dictionary() {

    //let _ = TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto);

    test_success!(r#"
        function main()
            var d = new Dictionary()
            d.set("a", 1)
            d.set("b", 2)
            d.set("c", 3)
            d.set("d", 4)
            d.set("e", 5)
            d.set("f", 6)
            d.set("g", 7)
            d.set("h", 8)
            d.set("i", 9)
            d.set("j", 10)
            d.set("k", 11)
            d.set("l", 12)
            d.set("m", 13)
            d.set("n", 14)
            d.set("o", 15)
            d.set("p", 16)
            d.set("q", 17)
            d.set("r", 18)
            d.set("s", 19)
            d.set("t", 20)
            d.set("u", 21)
            d.set("v", 22)
            d.set("w", 23)
            d.set("x", 24)
            d.set("y", 25)
            d.set("z", 26)
            return d.get("a") == 1 and d.get("b") == 2 and d.get("c") == 3 and d.get("d") == 4 and d.get("e") == 5 and d.get("f") == 6 and d.get("g") == 7 and d.get("h") == 8 and d.get("i") == 9 and d.get("j") == 10 and d.get("k") == 11 and d.get("l") == 12 and d.get("m") == 13 and d.get("n") == 14 and d.get("o") == 15 and d.get("p") == 16 and d.get("q") == 17 and d.get("r") == 18 and d.get("s") == 19 and d.get("t") == 20 and d.get("u") == 21 and d.get("v") == 22 and d.get("w") == 23 and d.get("x") == 24 and d.get("y") == 25 and d.get("z") == 26
        end
    "#);
}