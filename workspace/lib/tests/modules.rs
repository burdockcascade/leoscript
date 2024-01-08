mod common;

#[test]
fn module_with_functions() {
    test_success!(r#"
        function main()
            return Calculator::add(10, 20) == 30 and Calculator::sub(20, 10) == 10
        end

        module Calculator

            function add(v1, v2)
                return v1 + v2
            end

            function sub(v1, v2)
                return v1 - v2
            end

        end
    "#);
}

#[test]
fn module_with_inner_module_and_functions() {
    test_success!(r#"
        function main()
            return MyApp::Calculator::add(10, 20) == 30 and MyApp::Calculator::sub(20, 10) == 10
        end

        module MyApp

            module Calculator

                function add(v1, v2)
                    return v1 + v2
                end

                function sub(v1, v2)
                    return v1 - v2
                end

            end

        end


    "#);
}

#[test]
fn module_with_class() {
    test_success!(r#"
        function main()

            -- vector fun
            var v1 =  Graphics::Vector2(10, 20)
            var v2 =  Graphics::Vector2(20, 30)
            var v3 = v1.add(v2)
            var length = v3.length()
            return length == v3.length()
        end

        module Graphics

            class Vector2

                attribute x
                attribute y

                constructor(x, y)
                    self.x = x
                    self.y = y
                end

                function add(other)
                    return  Graphics::Vector2(self.x + other.x, self.y + other.y)
                end

                function sub(other)
                    return  Graphics::Vector2(self.x - other.x, self.y - other.y)
                end

                function mul(other)
                    return  Graphics::Vector2(self.x * other.x, self.y * other.y)
                end

                function div(other)
                    return  Graphics::Vector2(self.x / other.x, self.y / other.y)
                end

                function dot(other)
                    return self.x * other.x + self.y * other.y
                end

                function cross(other)
                    return self.x * other.y - self.y * other.x
                end

                function length()
                    return Math::sqrt(self.x * self.x + self.y * self.y)
                end

                function normalize()
                    var l = self.length()
                    return  Graphics::Vector2(self.x / l, self.y / l)
                end

                function to_string()
                    return "Vector2(" + self.x + ", " + self.y + ")"
                end

            end

        end

    "#);
}
