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