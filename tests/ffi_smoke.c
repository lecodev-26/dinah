#include "dinah.h"

int main(void) {
    if (dinah_abi_version() != 1u) return 1;
    if ((dinah_abi_features() & UINT64_C(1)) == 0) return 2;
    if (dinah_abi_platform_family() > 2u) return 3;
    return 0;
}
