int
main(void) {
    int another(void);
    int yet_another(void);
    int a = 0;
    return yet_another();
}

int
another(void) {
    int b = 0;
    b = b ? b = 1 : 2;
    return b;
}

int
yet_another(void) {
    int c = 0;
    int d = 0;
    c = c ? 1 : d ? 2 : 3;
    return c;
}