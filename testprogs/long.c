int
main(void) {
    long a = 1;
    int long b = 2l;
    long int c = 2147483648;
    long d = 2147483647;

    return 0;
}

long int
another(void) {
    long int d = 9223372036854775807;
    long e = 9223372036854775807l;
    return d + e;
}

long
yet_another(void) {
    long f = 123;
    int g = (int long)f;
    return f + g;
}