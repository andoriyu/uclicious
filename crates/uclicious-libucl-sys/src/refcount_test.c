#include <stdint.h>
#include <ucl.h>

extern ucl_object_t *uclicious_object_ref_checked(const ucl_object_t *, bool *);

/* Return the failing line to Rust, which reports it as a test failure. The
 * fixture owns these objects exclusively; only C ever accesses their fields. */
#define CHECK(condition) do { if (!(condition)) return __LINE__; } while (0)

unsigned int
uclicious_refcount_test(unsigned int test_case)
{
    bool overflow = true;
    ucl_object_t *object;
    ucl_object_t *copy;

    if (test_case == 0) {
        CHECK(uclicious_object_ref_checked(NULL, &overflow) == NULL);
        CHECK(!overflow);
        object = ucl_object_fromint(42);
        CHECK(object != NULL);
        copy = uclicious_object_ref_checked(object, &overflow);
        CHECK(copy == object && !overflow && object->ref == 2);
        ucl_object_unref(object);
        CHECK(ucl_object_toint(copy) == 42);
        ucl_object_unref(copy);
    }
    else if (test_case == 1) {
        object = ucl_object_fromint(42);
        CHECK(object != NULL);
        object->ref = UINT32_MAX - 1;
        copy = uclicious_object_ref_checked(object, &overflow);
        CHECK(copy == object && !overflow && object->ref == UINT32_MAX);
        CHECK(uclicious_object_ref_checked(object, &overflow) == NULL);
        CHECK(overflow && object->ref == UINT32_MAX);
        CHECK(ucl_object_toint(object) == 42);
        /* Remove synthetic references, leaving the two real owners. */
        object->ref = 2;
        ucl_object_unref(object);
        CHECK(ucl_object_toint(copy) == 42);
        ucl_object_unref(copy);
    }
    else {
        object = ucl_object_fromint(42);
        CHECK(object != NULL);
        object->flags |= UCL_OBJECT_EPHEMERAL;
        object->ref = UINT32_MAX;
        copy = uclicious_object_ref_checked(object, &overflow);
        CHECK(copy != NULL && copy != object && !overflow);
        CHECK(object->ref == UINT32_MAX && ucl_object_toint(copy) == 42);
        object->ref = 1;
        ucl_object_unref(object);
        CHECK(ucl_object_toint(copy) == 42);
        ucl_object_unref(copy);
    }
    return 0;
}

/* Built only with the refcount-test-fixture feature, for Rust unit tests. */
uint32_t
uclicious_test_set_refcount(ucl_object_t *object, uint32_t count)
{
    uint32_t previous = object->ref;
    object->ref = count;
    return previous;
}
