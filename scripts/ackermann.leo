
function ackermann(m, n)
  if m == 0 then
    return n + 1
  else if m == 1 then
    return 2 ^ n
  else if n == 0 then
    return ackermann(m - 1, 1)
  else
    return ackermann(m, ackermann(m - 1, n))
  end
end