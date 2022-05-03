#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

const int some_constant = 29;

const char* instructions = "\x48\x31\xff\xB8\x3C\x00\x00\x00\x0F\x05";

int main() {
    printf("main @ %p\n", main);
    printf("instructions @ %p\n", instructions);

    // Get the memory for instructions
    size_t region = (size_t) instructions;
    // Mask it to the lower page boundary
    region = region & (~0xFFF);

    printf("Page start: %p\n", (void*) region);

    printf("making instructions executable...\n");
    int ret = mprotect(
            (void *) region,
            0x1000,
            PROT_READ | PROT_EXEC
    );

    if (ret != 0) {
        printf("mprotect failed: error %s\n", strerror(errno));
        return 1;
    }

    void (*f)(void) = (void*) instructions;
    printf("jumping...\n");
    f();
    printf("after jump\n");
}
