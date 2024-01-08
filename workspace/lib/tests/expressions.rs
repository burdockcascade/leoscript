mod common;

#[test]
fn test_assign_to_variable() {

    test_success!(r#"
        function main()
            var a = 1
            a = 2
            return a == 2
        end
    "#);

}

#[test]
fn attribute_with_default_and_then_set_using_method() {

    test_success!(r#"
        class myservice

            attribute magic_number = 22

            function get_magic_number()
                return self.magic_number
            end

        end

        function main()
            var svc =  myservice()
            var x = svc.get_magic_number()
            return svc.get_magic_number() == 22
        end
    "#);

}

#[test]
fn attribute_with_default_and_then_set() {

    test_success!(r#"
        class myservice
            attribute magic_number = 22
        end

        function main()
            var svc =  myservice()
            svc.magic_number = 23
            return svc.magic_number == 23
        end
    "#);

}
