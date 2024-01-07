mod common;

#[test]
pub fn dict_new() {

    test_success!(r#"
        function main()
            var d =  Dictionary({a: 1, b: 2, c: 3})
            return d.length() == 3 and d.get("a") == 1 and d.get("b") == 2 and d.get("c") == 3
        end
    "#);
}

#[test]
pub fn dict_set() {

    test_success!(r#"
        function main()
            var d =  Dictionary()
            d.set("a", 1)
            d.set("b", 2)
            d.set("c", 3)
            return d.get("a") == 1 and d.get("b") == 2 and d.get("c") == 3
        end
    "#);
}

#[test]
pub fn dict_length() {

    test_success!(r#"
        function main()
            var d =  Dictionary()
            d.set("a", 1)
            d.set("b", 2)
            d.set("c", 3)
            return d.length() == 3
        end
    "#);
}

#[test]
pub fn dict_remove() {

    test_success!(r#"
        function main()
            var d =  Dictionary()
            d.set("a", 1)
            d.set("b", 2)
            d.set("c", 3)
            d.remove("b")
            return d.length() == 2 and d.get("a") == 1 and d.get("c") == 3
        end
    "#);
}

#[test]
pub fn dict_clear() {

    test_success!(r#"
        function main()
            var d =  Dictionary()
            d.set("a", 1)
            d.set("b", 2)
            d.set("c", 3)
            d.clear()
            return d.length() == 0
        end
    "#);
}

#[test]
pub fn dicts_2() {

    test_success!(r#"
        function main()
            var d1 =  Dictionary()
            var d2 =  Dictionary()
            d1.set("a", 1)
            d1.set("b", 2)
            d1.set("c", 3)
            d2.set("a", 1)
            d2.set("b", 2)
            d2.set("c", 3)
            return d1.get("a") == d2.get("a") and d1.get("b") == d2.get("b") and d1.get("c") == d2.get("c")
        end
    "#);
}