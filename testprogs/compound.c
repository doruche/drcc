int
main(void) {
    int a = 0;
    {
        int b = 1;
        {
            int a = 2;
            b = a + 3;
        }
        b = a + b;
    }
    return a;
}

int
another(void) {
    int x = 1;
    if (x > 0) {
        x = 2;
        return x;
    } else {
        int x = 3;
        x = x + 1;
        return x;
    }
}