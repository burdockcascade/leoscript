import scripts.option

function main()

    -- nonsense code to test the calculator
    -- return Calculator.add(1, 2) + Calculator.sub(1, 2) + Calculator.mul(1, 2) + Calculator.div(1, 2)

    var hat = new Option(4)

    match hat.has_value()

        case true then
            return true
        end

        case false then
            return false
        end

    end

end