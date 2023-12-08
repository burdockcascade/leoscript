function main()

    var a = 1
    var b = 2
    var c = Math.max(a, b)

    var v2 = new Graphics.Vector2(3, 4)
    var v3 = new Graphics.Vector3(5, 6, 7)

    return c
end

module Graphics

    class Vector2
        var x = 0
        var y = 0
    end

    class Vector3
        var x = 0
        var y = 0
        var z = 0
    end

end