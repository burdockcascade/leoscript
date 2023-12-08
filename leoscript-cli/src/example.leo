function main()

    var vectors = new Dictionary()
    vectors.insert("point1", new Graphics.Vector2(1,5))
    vectors.insert("point2", new Graphics.Vector2(6,4))
    vectors.insert("point3", new Graphics.Vector2(6,3))

end

module Graphics

    class Vector2
        var x = 0
        var y = 0

        constructor(x, y)
            this.x = x
            this.y = y
        end

    end

    class Vector3
        var x = 0
        var y = 0
        var z = 0

        constructor(x, y, z)
            this.x = x
            this.y = y
            this.z = z
        end

    end

end