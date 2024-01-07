use leoscript::runtime::ir::variant::Variant;

mod common;

#[test]
fn if_true() {

    let script = r#"
         function main(input)
            var output = input
            if input == 1 then
                output = input * 5
            end
            return output
        end
    "#;

    let test_matrix = vec![
        (Variant::Integer(1), Variant::Integer(5)),
        (Variant::Integer(2), Variant::Integer(2)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0]), test.1);
    }

}

#[test]
fn if_bool() {

    let script = r#"
         function main(input)
            var output = 2
            if input then
                output = 1
            end
            return output
        end
    "#;

    let test_matrix = vec![
        (Variant::Bool(true), Variant::Integer(1)),
        (Variant::Bool(false), Variant::Integer(2)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0]), test.1);
    }
}

#[test]
fn if_not_bool() {

    let script = r#"
         function main(input)

            var output = 2

            if not input then
                output = 1
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Bool(true), Variant::Integer(2)),
        (Variant::Bool(false), Variant::Integer(1)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0]), test.1);
    }
}

#[test]
fn if_not_expression() {

    let script = r#"
         function main(input)

            var output = true

            if not input == 5 then
                output = false
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Integer(1), Variant::Bool(false)),
        (Variant::Integer(2), Variant::Bool(false)),
        (Variant::Integer(3), Variant::Bool(false)),
        (Variant::Integer(4), Variant::Bool(false)),
        (Variant::Integer(5), Variant::Bool(true)),
        (Variant::Integer(6), Variant::Bool(false)),
        (Variant::Integer(7), Variant::Bool(false)),
        (Variant::Integer(8), Variant::Bool(false)),
        (Variant::Integer(9), Variant::Bool(false)),
        (Variant::Integer(10), Variant::Bool(false)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0]), test.1);
    }
}

#[test]
fn if_and() {

    let script = r#"
         function main(input1, input2, input3)

            var output = false

            if input1 and input2 and not input3 then
                output = true
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Bool(true), Variant::Bool(true), Variant::Bool(true), Variant::Bool(false)),
        (Variant::Bool(true), Variant::Bool(true), Variant::Bool(false), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(false), Variant::Bool(true), Variant::Bool(false)),
        (Variant::Bool(true), Variant::Bool(false), Variant::Bool(false), Variant::Bool(false)),
        (Variant::Bool(false), Variant::Bool(true), Variant::Bool(true), Variant::Bool(false)),
        (Variant::Bool(false), Variant::Bool(true), Variant::Bool(false), Variant::Bool(false)),
        (Variant::Bool(false), Variant::Bool(false), Variant::Bool(true), Variant::Bool(false)),
        (Variant::Bool(false), Variant::Bool(false), Variant::Bool(false), Variant::Bool(false)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0, test.1, test.2]), test.3);
    }
}

#[test]
fn if_or() {

    let script = r#"
         function main(input1, input2, input3)

            var output = false

            if input1 or input2 or input3 then
                output = true
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Bool(true), Variant::Bool(true), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(true), Variant::Bool(false), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(false), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(false), Variant::Bool(false), Variant::Bool(true)),
        (Variant::Bool(false), Variant::Bool(true), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(false), Variant::Bool(true), Variant::Bool(false), Variant::Bool(true)),
        (Variant::Bool(false), Variant::Bool(false), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(false), Variant::Bool(false), Variant::Bool(false), Variant::Bool(false)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0, test.1, test.2]), test.3);
    }
}

#[test]
fn if_or_and() {

    let script = r#"
         function main(input1, input2, input3)

            var output = false

            if input1 and input2 or input3 then
                output = true
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Bool(true), Variant::Bool(true), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(true), Variant::Bool(false), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(false), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(true), Variant::Bool(false), Variant::Bool(false), Variant::Bool(false)),
        (Variant::Bool(false), Variant::Bool(true), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(false), Variant::Bool(true), Variant::Bool(false), Variant::Bool(false)),
        (Variant::Bool(false), Variant::Bool(false), Variant::Bool(true), Variant::Bool(true)),
        (Variant::Bool(false), Variant::Bool(false), Variant::Bool(false), Variant::Bool(false)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0, test.1, test.2]), test.3);
    }
}

#[test]
fn if_else_if_else() {

    let script = r#"
         function main(input)

            var output = input

            if input == 1 then
                output = input * 2
            else if input == 2 then
                output = input * 3
            else
                output = input * 4
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Integer(1), Variant::Integer(2)),
        (Variant::Integer(2), Variant::Integer(6)),
        (Variant::Integer(3), Variant::Integer(12)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0]), test.1);
    }
}

#[test]
fn if_else() {

    let script = r#"
         function main(input)

            var output = input

            if input == 2 then
                output = input * 2
            else
                output = input * 3
            end

            return output

        end

    "#;

    let test_matrix = vec![
        (Variant::Integer(1), Variant::Integer(3)),
        (Variant::Integer(2), Variant::Integer(4)),
        (Variant::Integer(3), Variant::Integer(9)),
    ];

    for test in test_matrix {
        test_success!(script, Some(vec![test.0]), test.1);
    }
}

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
