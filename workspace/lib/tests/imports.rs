mod common;

#[test]
fn test_import_module() {

    test_success!(r#"
        import tests.scripts.graphics

        function main()

            var vectors = Dictionary()
            vectors.set("point1", Graphics::Vector2(1,5))
            vectors.set("point2", Graphics::Vector2(6,4))
            vectors.set("point3", Graphics::Vector2(6,3))

            var d = Graphics::Dimension(10, 20)
            var a = d.area()

            return a == (10 * 20)

        end
    "#)

}