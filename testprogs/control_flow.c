int
main(void) {
    int i = 0;
    while (i < 5) {
        int j;
        for (j = 0; j < 5; j = j + 1) {
            if (j == 2) {
                continue;
            } else if (j == 4) {
                break;
            }
        }
        i = i + 1;
        if (i == 3) {
            break;
        }
    }
    return i;
}