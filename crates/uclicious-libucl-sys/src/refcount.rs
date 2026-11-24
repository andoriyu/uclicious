use crate::ucl_object_t;

extern "C" {
    /// Acquire a native reference without overflowing its 32-bit count.
    ///
    /// NULL input returns NULL with `overflow` set to false. Ephemeral objects
    /// use libucl's deep-copy path. Other objects use an atomic compare-and-swap
    /// loop: at `UINT32_MAX`, return NULL and set `overflow` to true without
    /// changing the count; otherwise increment and return the original pointer.
    ///
    /// # Safety
    ///
    /// `object` must be NULL or a live libucl object whose lifetime covers this
    /// call. `overflow` must point to a writable bool. A successful non-NULL
    /// result must eventually be released with `ucl_object_unref`.
    pub fn uclicious_object_ref_checked(
        object: *const ucl_object_t,
        overflow: *mut bool,
    ) -> *mut ucl_object_t;
}

#[cfg(all(test, feature = "refcount-test-fixture"))]
mod tests {
    extern "C" {
        fn uclicious_refcount_test(case: u32) -> u32;
    }

    #[test]
    fn null_and_normal_acquisition() {
        assert_eq!(unsafe { uclicious_refcount_test(0) }, 0);
    }

    #[test]
    fn overflow_does_not_mutate_count() {
        assert_eq!(unsafe { uclicious_refcount_test(1) }, 0);
    }

    #[test]
    fn ephemeral_objects_are_copied() {
        assert_eq!(unsafe { uclicious_refcount_test(2) }, 0);
    }
}
