int
main(void) {
    int i = 0;
    while (i < 5) {
        i = i + 1;
    }
    return i;
}

int
do_while(void) {
    int j = 0;
    do {
        j = j + 1;
    } while (j < 5);
    return j;
}

int
for_loop(void) {
    int k = 0;
    for (int l = 0; l < 5; l = l + 1) {
        k = k + 1;
    }

    for (k = 0; k < 5; k = k + 1) {
        ;
    }

    for (;;) {
        return k;
    }

    return k;
}