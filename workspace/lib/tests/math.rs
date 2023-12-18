use leoscript::common::variant::Variant;
use leoscript::run_script_from_string;

mod common;

#[test]
fn test_math_max() {
    test_success!(r#"
        function main()
            return Math.max(1, 2) == 2
        end
    "#);
}

#[test]
fn test_math_min() {
    test_success!(r#"
        function main()
            return Math.min(1, 2) == 1
        end
    "#);
}


#[test]
fn test_math_abs() {
    test_success!(r#"
        function main()
            return Math.abs(-1) == 1
        end
    "#);
}

#[test]
fn test_math_sqrt() {
    test_success!(r#"
        function main()
            return Math.sqrt(4) == 2.0
        end
    "#);
}