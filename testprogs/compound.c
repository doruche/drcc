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