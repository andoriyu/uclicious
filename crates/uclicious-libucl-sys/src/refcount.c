#include <stdint.h>
#include <ucl.h>

/* overflow must be non-NULL. No native layout assumptions escape this file. */
ucl_object_t *
uclicious_object_ref_checked(const ucl_object_t *object, bool *overflow)
{
    ucl_object_t *res = (ucl_object_t *)object;
    uint32_t count;

    *overflow = false;
    if (res == NULL) {
        return NULL;
    }
    if (res->flags & UCL_OBJECT_EPHEMERAL) {
        return ucl_object_ref(object);
    }

    count = __atomic_load_n(&res->ref, __ATOMIC_SEQ_CST);
    for (;;) {
        if (count == UINT32_MAX) {
            *overflow = true;
            return NULL;
        }
        if (__atomic_compare_exchange_n(&res->ref, &count, count + 1, true,
                                        __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST)) {
            return res;
        }
    }
}
