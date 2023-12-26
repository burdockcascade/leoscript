use leoscript::run_script;
use leoscript::runtime::ir::variant::Variant;

mod common;

#[test]
fn for_loop() {
    test_success!(r#"
         function main()

            -- set counter to 0
            var counter = 0

            -- set start to 1
            var start_at as Integer = 1

            -- set target to 3
            var target as Integer = 20

            var stepi = 1

            -- loop
            for t = start_at to target step stepi do

                -- increment counter
                counter = target + stepi + t

            end

            return counter == 41

        end
    "#);
}

#[test]
fn for_loop_with_continue() {
   test_success!(r#"
         function main()

            var counter = 0

            for t = 1 to 10 step 1 do

                if t == 5 then
                    continue
                end

                counter = counter + 1

            end

            return counter == 9

        end
    "#);
}

#[test]
fn for_loop_with_break() {
    test_success!(r#"

         function main()

            var counter = 0

            for t = 1 to 10 step 2 do

                if t >= 10 then
                    break
                end

                counter = counter + 1

            end

            return counter == 5

        end

    "#);
}

#[test]
fn for_in_array() {
    test_success!(r#"
         function main()

            -- set a counter
            var counter = 0

            -- set an array of numbers
            var v1 = [2, 3, 4, 5, 6, 7, 8, 9]

            -- for in loop
            for x in v1 do

                -- increment counter
                counter = counter + 1

            end

            return counter == 8

        end
    "#);
}

#[test]
fn for_in_dict() {
    test_success!(r#"
         function main()

            var counter = 0

            var numbers = new Dictionary()
            numbers.set("one", 1)
            numbers.set("two", 2)
            numbers.set("three", 3)
            numbers.set("four", 4)
            numbers.set("five", 5)

            for num in numbers.keys() do
                counter = counter + 1
            end

            return counter == 5

        end
    "#);
}

#[test]
fn for_in_dict_with_continue() {
    test_success!(r#"
         function main()

            -- set a counter
            var counter = 0

            -- set an array of numbers
            var numbers = new Dictionary()
            numbers.set("one", 1)
            numbers.set("two", 2)
            numbers.set("three", 3)
            numbers.set("four", 4)
            numbers.set("five", 5)
            numbers.set("six", 6)

            -- for in loop
            for x in numbers.keys() do

                if x == "three" then
                    continue
                end

                -- increment the counter
                counter = counter + 1

            end

            return counter == 5

        end
    "#);
}


#[test]
fn while_loop() {
    test_success!(r#"
         function main()

            var b as Integer = 0

            while b < 10 do

                b = b + 1

            end

            return b == 10

        end
    "#);
}

#[test]
fn while_loop_with_inner_loop() {
    test_success!(r#"
         function main()

            -- loop in a loop
            var d = 0
            var x = 0
            while d < 4 do

                d = d + 1

                var e = 0
                while e < 4 do

                    e = e + 1

                    x = x + 1

                end

            end

            return x == 16

        end
    "#);
}

#[test]
fn while_loop_with_continue() {
    test_success!(r#"
         function main()

            -- test while continue
            var a = 0
            var b = 0
            while a < 4 do

                a = a + 1

                if a == 2 then
                    continue
                end

                b = b + 1

            end

            return b == 3

        end
    "#);
}

#[test]
fn while_loop_with_break() {
    test_success!(r#"
         function main()

            -- test while break
            var c = 0
            var b = 0
            while c < 4 do

                c = c + 1

                if c == 2 then
                    break
                end

                b = b + 1

            end

            return b == 1

        end
    "#);
}