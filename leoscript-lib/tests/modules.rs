use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};

use leoscript_lib::common::variant::Variant;
use leoscript_lib::run_script;

mod common;

#[test]
fn simple_module() {

    //let _ = TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto);

    test_success!(r#"
        function main()

            -- vector fun
            var v1 = new Graphics.Vector2(10, 20)
            var v2 = new Graphics.Vector2(20, 30)
            var v3 = v1.add(v2)
            var length = v3.length()

            return Math.max(10, 20) == 20 and Math.min(10, 20) == 10
        end

        module Graphics

            class Vector2

                var x as Integer
                var y as Integer

                constructor(x, y)
                    self.x = x
                    self.y = y
                end

                function add(other)
                    return new Graphics.Vector2(self.x + other.x, self.y + other.y)
                end

                function sub(other)
                    return new Graphics.Vector2(self.x - other.x, self.y - other.y)
                end

                function mul(other)
                    return new Graphics.Vector2(self.x * other.x, self.y * other.y)
                end

                function div(other)
                    return new Graphics.Vector2(self.x / other.x, self.y / other.y)
                end

                function dot(other)
                    return self.x * other.x + self.y * other.y
                end

                function cross(other)
                    return self.x * other.y - self.y * other.x
                end

                function length()
                    return Math.sqrt(self.x * self.x + self.y * self.y)
                end

                function normalize()
                    var l = self.length()
                    return new Graphics.Vector2(self.x / l, self.y / l)
                end

                function to_string()
                    return "Vector2(" + self.x + ", " + self.y + ")"
                end

            end

        end

    "#);
}
