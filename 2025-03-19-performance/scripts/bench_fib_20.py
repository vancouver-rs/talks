
def fib(n):
    if n < 2:
        return 1
    return fib(n-1) + fib(n-2)

from timeit import timeit

number = 5000
t = timeit("fib(20)", globals=globals(), number=number)
print(f"fib(20) took {t/number*1_000_000:.1f} microseconds")
