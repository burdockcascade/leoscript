function main()

    var vectors = new Dictionary()
    vectors.set("point1", new Graphics.Vector2(1,5))
    vectors.set("point2", new Graphics.Vector2(6,4))
    vectors.set("point3", new Graphics.Vector2(6,3))

    return vectors.length()

end

module Graphics

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

end