import examples.scripts.graphics

function main()

    var r = new Graphics.Rectangle(3, 10, 100)
    var p = new Graphics.Point(0, 0)

    return r.contains(p)

end