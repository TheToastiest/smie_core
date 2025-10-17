use crate::error::SmieError;
use crate::ffi::*;
use std::ffi::CString;
use std::ptr;

pub struct Index {
    ptr: *mut FaissIndex,
}

impl Index {
    /// Load a FAISS index from a file path
    pub fn from_file(path: &str) -> Result<Self, SmieError> {
        let c_path = CString::new(path)
            .map_err(|e| SmieError::Other(format!("Failed to convert path to CString: {e}")))?;
        
        let mut index_ptr: *mut FaissIndex = ptr::null_mut();
        let result = unsafe { faiss_read_index(c_path.as_ptr(), 0, &mut index_ptr) };

        if result != 0 || index_ptr.is_null() {
            return Err(SmieError::Other("Failed to load FAISS index".into()));
        }

        Ok(Self { ptr: index_ptr })
    }

    /// Create a new FlatL2 index with given dimensionality
    pub fn new_flat_l2(dim: i32) -> Result<Self, SmieError> {
        let mut index_ptr: *mut FaissIndex = ptr::null_mut();
        let result = unsafe { faiss_IndexFlatL2_new(&mut index_ptr, dim) };

        if result != 0 || index_ptr.is_null() {
            return Err(SmieError::Other(format!(
                "Failed to create FlatL2 index with dim {}",
                dim
            )));
        }

        Ok(Self { ptr: index_ptr })
    }

        pub fn save(&self, path: &str) -> Result<(), SmieError> {
        let c_path = CString::new(path)
            .map_err(|e| SmieError::Other(format!("Invalid path for CString: {e}")))?;
        
        let result = unsafe { faiss_write_index(self.ptr, c_path.as_ptr()) };

        if result != 0 {
            return Err(SmieError::Other("Failed to write FAISS index to file".into()));
        }

        Ok(())
    }

        pub fn safe_search(&self, embedding: &[f32], k: usize) -> Result<Vec<i64>, SmieError> {
        let (_distances, labels) = self.search(embedding, k)?;
        Ok(labels)
    }

    /// Returns the dimensionality of vectors in the index
    pub fn dim(&self) -> Result<i32, SmieError> {
        Ok(unsafe { faiss_Index_d(self.ptr) })
    }

    /// Returns the number of vectors stored in the index
    pub fn total_entries(&self) -> Result<i64, SmieError> {
        Ok(unsafe { faiss_Index_ntotal(self.ptr) })
    }

    /// Search the index for top-k closest vectors to the given embedding
    pub fn search(&self, embedding: &[f32], k: usize) -> Result<(Vec<f32>, Vec<i64>), SmieError> {
        let expected_dim = self.dim()? as usize;

        if embedding.len() != expected_dim {
            return Err(SmieError::Other(format!(
                "Embedding dim mismatch: expected {}, got {}",
                expected_dim,
                embedding.len()
            )));
        }

        let mut distances = vec![0.0_f32; k];
        let mut labels = vec![-1_i64; k];

        println!(
            "[Debug] FAISS dim = {}, embedding len = {}, k = {}",
            expected_dim, embedding.len(), k
        );
        println!(
            "[Debug] Index ptr = {:?}, distances ptr = {:?}, labels ptr = {:?}",
            self.ptr,
            distances.as_mut_ptr(),
            labels.as_mut_ptr()
        );

        let result = unsafe {
            faiss_Index_search(
                self.ptr,
                1,                         // n = 1 query vector
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
        unsafe { faiss_Index_free(self.ptr) };
    }
}
