-- example module with multiple classes
module Graphics

    class Point
        var x = 0
        var y = 0

        constructor(x, y)
            self.x = x
            self.y = y
        end

    end

    class Vector2
        var x = 0
        var y = 0

        constructor(x, y)
            self.x = x
            self.y = y
        end

    end

    class Vector3
        var x = 0
        var y = 0
        var z = 0

        constructor(x, y, z)
            self.x = x
            self.y = y
            self.z = z
        end

    end

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

end