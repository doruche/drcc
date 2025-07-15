int
main(void) {
    int x = 0;
    if (x == 0)
        x = 1;
    else
        x = 2;
    return x;
}

int
another(void) {
    int y = 0;
    if (y == 0)
        y = 1;
    else if (y == 1)
        y = 2;
    else if (y == 2)
        y = 3;
    else
        y = 4;
    return y;
}

int
yet_another(void) {
    int z = 0;
    if (z == 0)
        if (z == 1)
            z = 2;
    return z;
}
