-- example module with multiple classes for testing purposes
-- code is nonsense, but it's just for testing
module Graphics

    class Point
        var x = 0
        var y = 0

        constructor(x, y)
            self.x = x
            self.y = y
        end

    end

    class Rectangle

        var width = 0
        var height = 0
        var origin = new Graphics.Point(0, 0)

        constructor(origin, width, height)
            self.origin = origin
            self.width = width
            self.height = height
        end

        function area()
            return self.width * self.height
        end

        function perimeter()
            return 2 * (self.width + self.height)
        end

        function contains(point)
            return point.x >= self.origin.x and point.x < (self.origin.x + self.width) and
                   point.y >= self.origin.y and point.y < (self.origin.y + self.height)
        end

    end

    class Vector2
        var x = 0
        var y = 0

        constructor(x, y)
            self.x = x
            self.y = y
        end

        function add(other)
            return new Graphics.Vector2(self.x + other.x, self.y + other.y)
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