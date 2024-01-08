use leoscript::runtime::ir::variant::Variant;

mod common;

#[test]
fn class_with_no_constructor() {
    let script = r#"
        function main(x)
            var d = Book()
            d.pages = x
            return d.pages == x
        end

        class Book
            attribute pages = 0
        end

    "#;

    test_success!(script, Some(vec![Variant::Integer(10)]))
}

#[test]
fn class_with_fields_but_without_constructor() {
    let script = r#"
        class myservice

            attribute magic_number = 22

            function get_magic_number()
                return self.magic_number
            end

        end

        function main()
            var svc = myservice()
            return svc.get_magic_number()
        end

    "#;

    test_success!(script, None, Variant::Integer(22))
}

#[test]
fn pass_object_as_parameter() {
    test_success!(r#"
        class myservice

            attribute magic_number = 23

            function get_magic_number()
                return self.magic_number
            end

        end

        function do_something_with_service(svc)
            svc.magic_number = svc.get_magic_number() + 1
        end

        function main()
            var svc = myservice()
            do_something_with_service(svc)
            return svc.get_magic_number() == 24
        end
    "#);
}
