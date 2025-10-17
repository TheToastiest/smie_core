use crate::ffi::*;
use std::ptr;
use std::ffi::CString;
use crate::error::SmieError;

/// Safe wrapper around `FaissIndex`
pub struct Index {
    inner: *mut FaissIndex,
}

impl Index {
    /// Creates a FlatL2 index with the given dimension
    /// Creates an Index from a raw FAISS pointer
pub fn from_ptr(ptr: *mut FaissIndex) -> Self {
    Self { inner: ptr }
}
    pub fn new_flat_l2(d: i32) -> Result<Self, i32> {
        let mut index: *mut FaissIndex = ptr::null_mut();
        let err = unsafe { faiss_IndexFlatL2_new(&mut index, d) };
        if err == 0 {
            Ok(Self { inner: index })
        } else {
            Err(err)
        }
    }

    pub fn d(&self) -> i32 {
        unsafe { faiss_Index_d(self.inner) }
    }

    pub fn add(&mut self, n: i64, x: &[f32]) -> Result<(), i32> {
        let err = unsafe { faiss_Index_add(self.inner, n, x.as_ptr()) };
        if err == 0 { Ok(()) } else { Err(err) }
    }

    pub fn search(&self, n: i64, x: &[f32], k: i64) -> Result<(Vec<f32>, Vec<i64>), i32> {
        let mut distances = vec![0.0f32; (n * k) as usize];
        let mut labels = vec![-1i64; (n * k) as usize];
        let err = unsafe {
            faiss_Index_search(
                self.inner,
                n,
                x.as_ptr(),
                k,
                distances.as_mut_ptr(),
                labels.as_mut_ptr(),
            )
        };
        if err == 0 {
            Ok((distances, labels))
        } else {
            Err(err)
        }
    }
    
    pub fn from_file(path: &str) -> Result<Self, SmieError> {
        let c_path = CString::new(path).map_err(|e| SmieError::Other(format!("Invalid path CString: {e}")))?;
        let mut index: *mut FaissIndex = ptr::null_mut();
        let err = unsafe { faiss_read_index(c_path.as_ptr(), 0, &mut index) };

        if err == 0 && !index.is_null() {
            Ok(Self { inner: index })
        } else {
        Err(SmieError::Other("Failed to load FAISS index".into()))
        }
    }


    pub fn save(&self, path: &str) -> Result<(), i32> {
        let c_path = CString::new(path).unwrap();
        let err = unsafe { faiss_write_index(self.inner, c_path.as_ptr()) };
        if err == 0 {
            Ok(())
        } else {
            Err(err)
        }
    }

    pub fn safe_search(&self, embedding: &[f32], k: usize) -> Result<(Vec<f32>, Vec<i64>), SmieError> {
        let expected_dim = self.d() as usize;

        if embedding.len() != expected_dim {
            return Err(SmieError::Other(format!(
                "Embedding dim mismatch: expected {}, got {}",
                expected_dim,
                embedding.len()
            )));
        }

        let mut distances = vec![0.0_f32; k];
        let mut labels = vec![-1_i64; k];

        let result = unsafe {
            faiss_Index_search(
                self.inner,
                1,
                embedding.as_ptr(),
                k as i64,
                distances.as_mut_ptr(),
                labels.as_mut_ptr(),
            )
        };

        if result != 0 {
            return Err(SmieError::Other("FAISS search failed".into()));
        }

        Ok((distances, labels))
    }

}

impl Drop for Index {
    fn drop(&mut self) {
        unsafe { faiss_Index_free(self.inner) };
    }
}


