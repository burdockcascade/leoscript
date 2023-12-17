class Dictionary

    var value = {}

    constructor(value)
        self.value = value
    end

    function get(key)
        return std_dictionary_get(self.value, key)
    end

    function set(key, value)
        self.value = std_dictionary_set(self.value, key, value)
    end

    function length()
        return std_dictionary_length(self.value)
    end

    function keys()
        return std_dictionary_keys(self.value)
    end

    function values()
        return std_dictionary_values(self.value)
    end

    function remove(key)
        self.value = std_dictionary_remove(self.value, key)
    end

    function clear()
        self.value = std_dictionary_clear(self.value)
    end

end