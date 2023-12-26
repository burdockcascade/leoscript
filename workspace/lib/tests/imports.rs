use leoscript::run_script;
use leoscript::runtime::ir::variant::Variant;

mod common;

#[test]
fn test_import_module() {

    test_success!(r#"
        import tests.scripts.graphics
        import tests.scripts.person

        function main()

            var vectors = new Dictionary()
            vectors.set("point1", new Graphics::Vector2(1,5))
            vectors.set("point2", new Graphics::Vector2(6,4))
            vectors.set("point3", new Graphics::Vector2(6,3))

            var p = new Person("John Doe", 30)

            var d = new Graphics::Dimension(10, 20)
            var a = d.area()

            return a == (10 * 20)

        end
    "#)

}