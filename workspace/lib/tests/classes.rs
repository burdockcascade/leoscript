use leoscript::run_script;
use leoscript::runtime::ir::variant::Variant;

mod common;

#[test]
fn class_with_constructor_and_parameters() {

    let script = r#"
        class Book

            attribute name
            attribute pages = 0
            attribute author = "unknown"

            constructor(name, pages, author)
                self.name = name
                self.pages = pages
                self.author = author
            end

            function get_name()
                return self.name
            end

        end

        function main(x, y)
            var d = new Book("The Lord of the Rings", 1000, "J.R.R. Tolkien")
            return d.get_name() == "The Lord of the Rings"
        end

    "#;

    test_success!(script, Some(vec![Variant::Integer(10), Variant::Integer(20)]), Variant::Integer(230))

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
            var svc = new myservice()
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
            var svc = new myservice()
            do_something_with_service(svc)
            return svc.get_magic_number() == 24
        end
    "#);

}
