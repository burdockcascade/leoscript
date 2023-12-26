use leoscript::run_script;
use leoscript::runtime::ir::variant::Variant;

mod common;

#[test]
fn test_match_case_and_default() {

    test_success!(r#"
        function main()

            var a = 2
            var b = false

            match a

                case 1 then
                    b = false
                end

                case 2 then
                    b = true
                end

                default then
                    b = false
                end

            end

            return b

        end
    "#);
}

#[test]
fn test_match_no_match_and_no_default() {

    test_success!(r#"
        function main()

            var a = 2
            var b = false

            match a

                case 1 then
                    b = false
                end

            end

            return not b

        end
    "#);
}

#[test]
fn test_match_case_with_mixed_values() {

    test_success!(r#"
        function main()

            var a = "Green"
            var b = false

            match a

                case 1 then
                    b = false
                end

                case "Orange" then
                    b = false
                end

                case "Green" then
                    b = true
                end

                default then
                    b = false
                end

            end

            return b

        end
    "#);
}

#[test]
fn test_match_case_default() {

    test_success!(r#"
        function main()

            var a = 9
            var b = false

            match a

                case 1 then
                    b = false
                end

                case 2 then
                    b = false
                end

                default then
                    b = true
                end

            end

            return b

        end
    "#);
}

#[test]
fn test_match_case_default_out_of_order() {

    test_success!(r#"
        function main()

            var a = 9
            var b = false

            match a

                case 1 then
                    b = false
                end

                default then
                    b = true
                end

                case 2 then
                    b = false
                end

            end

            return b

        end
    "#);
}