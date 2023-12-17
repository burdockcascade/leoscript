// macro that takes a script, runs it and tests the result is true
#[macro_export]
macro_rules! test_success {
    ($script:expr, $parameters:expr, $expected:expr) => {
        match run_script($script, "main", $parameters) {
            Ok(result) => {
                assert_eq!(result.result, Some($expected));
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    };

    ($script:expr, $parameters:expr) => {
        test_success!(script, $parameters, Variant::Bool(true))
    };

    ($script:expr) => {
        test_success!($script, None, Variant::Bool(true))
    };
}

#[macro_export]
macro_rules! test_success_matrix {
    ($script:expr, $parameters:expr, $expected:expr) => {
        for test in $expected {
            test_success!($script, Some(vec![test.0]));
        }
    };
}

// macro that runs script and tests for a compiler error
#[macro_export]
macro_rules! test_failure {
    ($script:expr, $parameters:expr, $expected:expr, $line:expr, $row:expr) => {
        match run_script($script, "main", $parameters) {
            Ok(result) => {
                assert!(false, "Expected error, got: {:?}", result);
            }
            Err(e) => {
                assert_eq!(
                    e,
                    ScriptError::CompilerError {
                        error: $expected,
                        line: $line,
                        column: $row,
                    }
                );
            }
        }
    };
}