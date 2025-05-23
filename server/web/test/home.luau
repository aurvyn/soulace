--!strict

function echo(text: string): ()
    print(text)
end

function fibonacci(text: string): ()
    local n = tonumber(text)
    if n == nil then
        print("Invalid input: not a number")
        return
    end
    local result = fib(n)
    print(result)
end

function fib(n: number): number
    if n <= 1 then
        return n
    end
    return fib(n - 1) + fib(n - 2)
end