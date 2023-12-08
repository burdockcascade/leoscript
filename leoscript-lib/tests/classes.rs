use leoscript_lib::common::variant::Variant;
use leoscript_lib::run_script;

mod common;

#[test]
fn class_with_constructor_with_parameters() {

    let script = r#"
        class Dimension

            var height as Integer
            var length as Integer
            var magic_number = 2

            constructor(h, l)
                self.height = h
                self.length = l
            end

            -- error in function
            function area()
                return self.calc_area()
            end

            function calc_area() as Integer
                return self.height * self.length
            end

            function perimeter()
                return self.magic_number * (self.height + self.length)
            end

        end

        function main(x, y)
            var d = new Dimension(x, y)
            return d.area() + d.perimeter()
        end

    "#;

    test_success!(script, Some(vec![Variant::Integer(10), Variant::Integer(20)]), Variant::Integer(260))

}

#[test]
fn class_without_constructor() {

    let script = r#"
        class myservice

            var magic_number = 2 * 13  - 4

            function get_magic_number()
                return self.magic_number
            end

        end

        function main()
            var svc = new myservice()
            return svc.get_magic_number()
        end

    "#;

    test_success!(script, None, Variant::Integer(22))

}
