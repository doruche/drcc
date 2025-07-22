int
foo(int x, int y) {
    return 10 - (3 * y + x);
}

int
bar(void) {
    int a = 5;
    int b = ~5;
    int c = !5;
    int d = 5 + 3;
    int e = 5l - 3;
    int f = 5 * 3;
    int g = 5 / 3;
    int h = 5 % 3;
    int i = 5 == 3;
    int j = 5 != 3;
    int k = 5 < 3;
    int l = 5 > 3;
    return 0;
}