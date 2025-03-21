
def main():
    curr = 1
    last = 1

    count = 1

    fout = open("x.rs", "w")

    while curr < 2**64:
        nxt = curr + last
        last = curr
        curr = nxt
        count += 1
        print(f"fib({count}) -> {nxt}")
        fout.write(f"    {nxt}, // {count}\n")

    fout.write("\n")
    fout.close()


if __name__ == "__main__":
    main()
