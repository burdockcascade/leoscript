class Option

    var value

    constructor(value)
        self.value = value
    end

    function has_value()
        return self.value != null
    end

end