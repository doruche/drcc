int
main(int argc) {
    int foo(void);
    int foo(void);
    foo();
    {
        int foo = 3;
        foo = 4;
    }
    return 0;
}

int
another(int a, int b) {
    int foo(void);
    {
        int foo = 3;
    }
    return a + b;
}


int
yet_another(int a) {
    if (a > 0) {
        return a + yet_another(a - 1);
    } else {
        return 0;
    }
}

int
huge(int a, int b, int c, int d, int e, int f, int g, int h, long x, int y) {
    return a + b + c + d + e + f + g + h + x + y;
}

int
call_huge(void) {
    int val = huge(1, 2, 3, 4, 5, 6, 7, 8, 9L, 10);
    return val;
}