/* automatically generated + manually expanded FFI bindings to FAISS C API */

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub type faiss_idx_t = i64;
pub type idx_t = faiss_idx_t;
pub type FaissMetricType = ::std::os::raw::c_int;

pub const FaissMetricType_METRIC_INNER_PRODUCT: FaissMetricType = 0;
pub const FaissMetricType_METRIC_L2: FaissMetricType = 1;
pub const FaissMetricType_METRIC_L1: FaissMetricType = 2;
pub const FaissMetricType_METRIC_Linf: FaissMetricType = 3;
pub const FaissMetricType_METRIC_Lp: FaissMetricType = 4;
pub const FaissMetricType_METRIC_Canberra: FaissMetricType = 20;
pub const FaissMetricType_METRIC_BrayCurtis: FaissMetricType = 21;
pub const FaissMetricType_METRIC_JensenShannon: FaissMetricType = 22;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FaissIndex_H {
    _unused: [u8; 0],
}
pub type FaissIndex = FaissIndex_H;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FaissSearchParameters_H {
    _unused: [u8; 0],
}
pub type FaissSearchParameters = FaissSearchParameters_H;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FaissIDSelector_H {
    _unused: [u8; 0],
}
pub type FaissIDSelector = FaissIDSelector_H;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FaissRangeSearchResult_H {
    _unused: [u8; 0],
}
pub type FaissRangeSearchResult = FaissRangeSearchResult_H;

extern "C" {
    pub fn faiss_IndexFlatL2_new(p_index: *mut *mut FaissIndex, d: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
    pub fn faiss_Index_free(index: *mut FaissIndex);
    pub fn faiss_Index_d(index: *const FaissIndex) -> ::std::os::raw::c_int;
    pub fn faiss_Index_is_trained(index: *const FaissIndex) -> ::std::os::raw::c_int;
    pub fn faiss_Index_ntotal(index: *const FaissIndex) -> idx_t;
    pub fn faiss_Index_train(index: *mut FaissIndex, n: idx_t, x: *const f32) -> ::std::os::raw::c_int;
    pub fn faiss_Index_add(index: *mut FaissIndex, n: idx_t, x: *const f32) -> ::std::os::raw::c_int;
    pub fn faiss_Index_add_with_ids(index: *mut FaissIndex, n: idx_t, x: *const f32, ids: *const idx_t) -> ::std::os::raw::c_int;
    pub fn faiss_Index_search(index: *const FaissIndex, n: idx_t, x: *const f32, k: idx_t, distances: *mut f32, labels: *mut idx_t) -> ::std::os::raw::c_int;
    pub fn faiss_Index_reset(index: *mut FaissIndex) -> ::std::os::raw::c_int;

    // Optional: With selector / search parameters
    pub fn faiss_Index_search_with_params(
        index: *const FaissIndex,
        n: idx_t,
        x: *const f32,
        k: idx_t,
        params: *const FaissSearchParameters,
        distances: *mut f32,
        labels: *mut idx_t,
    ) -> ::std::os::raw::c_int;

    pub fn faiss_Index_range_search(
        index: *const FaissIndex,
        n: idx_t,
        x: *const f32,
        radius: f32,
        result: *mut FaissRangeSearchResult,
    ) -> ::std::os::raw::c_int;

    // From faiss/c_api/index_io_c.h
    pub fn faiss_read_index(
        fname: *const ::std::os::raw::c_char,
        io_flags: ::std::os::raw::c_int,
        p_out: *mut *mut FaissIndex,
    ) -> ::std::os::raw::c_int;

    pub fn faiss_write_index(
        index: *const FaissIndex,
        fname: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;

    // Optional: ID selector helpers
    pub fn faiss_IDSelectorBatch_new(
        p_sel: *mut *mut FaissIDSelector,
        n: idx_t,
        ids: *const idx_t,
    ) -> ::std::os::raw::c_int;

    pub fn faiss_IDSelector_free(sel: *mut FaissIDSelector);

    // Optional: Parameter builder
    pub fn faiss_SearchParameters_new(
        p_sp: *mut *mut FaissSearchParameters,
        sel: *mut FaissIDSelector,
    ) -> ::std::os::raw::c_int;

    pub fn faiss_SearchParameters_free(obj: *mut FaissSearchParameters);
}
