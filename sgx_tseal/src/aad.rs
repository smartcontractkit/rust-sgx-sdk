// Copyright (c) 2017 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! 
//! Provides APIs to authenticate and verify the input data with AES-GMAC.
//!
use sgx_types::*;
use core::mem;
use core::ptr;
use core::slice;
use super::internal::*;
#[cfg(not(feature = "use_std"))]
use alloc::boxed::Box;

/// The structure about sealed data, for authenticate and verify.
pub struct SgxMacAadata<T: ?Sized> {
    inner: SgxInternalSealedData,
    marker: * const T,
}

impl<T> Default for SgxMacAadata<T> {
    fn default() -> SgxMacAadata<T> {
        SgxMacAadata {
            inner: SgxInternalSealedData::new(),
            marker: ptr::null(), 
        }
    }
}

impl<T> Default for SgxMacAadata<[T]> {
    fn default() -> SgxMacAadata<[T]> {
        let p = Box::<[T]>::default();
        SgxMacAadata {
            inner: SgxInternalSealedData::new(),
            marker: Box::into_raw(p),
        }
    }
}

impl<T: ?Sized> Clone for SgxMacAadata<T> {
    fn clone(&self) -> SgxMacAadata<T> {
        SgxMacAadata {
           inner: self.inner.clone(),
           marker: self.marker,
        }
    }
}

impl<T> SgxMacAadata<T> {
    ///
    /// Create a SgxMacAadata with default values.
    /// 
    pub fn new() -> Self {
        SgxMacAadata::default()
    }

    /// 
    /// Convert a pointer of sgx_sealed_data_t buffer to SgxMacAadata.
    ///
    /// # Requirements
    /// 
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///  
    /// # Parameters
    ///
    /// **p**
    ///
    /// The mutable pointer of sgx_sealed_data_t buffer.
    ///
    /// **len**
    ///
    /// The size of the parameter `p`.
    ///
    /// # Return value 
    ///
    /// **Some(SgxMacAadata)**
    ///
    /// Indicates the conversion is successfully. The return value is SgxMacAadata.
    ///
    /// **None**
    /// 
    /// Maybe the size of T is zero.
    /// 
    pub unsafe fn from_raw_sealed_data_t(p: * mut sgx_sealed_data_t, len: u32) -> Option<Self> {
        
        let size = mem::size_of::<T>();
        if size == 0 {
            return None;
        } 
        SgxInternalSealedData::from_raw_sealed_data_t(p, len).map(|x| SgxMacAadata{inner: x, marker: ptr::null()})
    }

    /// 
    /// This function is used to authenticate the input data with AES-GMAC.
    ///
    /// # Descryption
    /// 
    /// The mac_aadata function retrieves a key unique to the enclave and
    /// uses that key to generate the authentication tag based on the input data buffer. This function can be utilized to provide authentication assurance for additional data (of practically unlimited length per invocation) that is not
    /// encrypted. The data origin authentication can be demonstrated on future
    /// instantiations of the enclave using the MAC stored into the data blob.
    /// Use `calc_raw_sealed_data_size` to calculate the number of bytes to
    /// allocate for the SgxMacAadata structure. The input sealed data buffer
    /// must be allocated within the enclave
    /// 
    /// # Requirements
    /// 
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///  
    /// # Parameters
    ///
    /// **additional_text**
    ///
    /// Pointer to the plain text to provide authentication for.
    ///
    /// # Return value 
    ///
    /// The sealed data in SgxMacAadata.
    ///
    /// # Errors
    /// 
    /// **SGX_ERROR_INVALID_PARAMETER**
    /// 
    /// Indicates an error if the parameters do not meet any of the following conditions:
    /// 
    /// * additional_text buffer can be within or outside the enclave, but cannot cross the enclave boundary.
    /// * encrypt_text must be non-zero.
    /// * encrypt_text buffer must be within the enclave.
    /// 
    /// **SGX_ERROR_OUT_OF_MEMORY**
    /// 
    /// The enclave is out of memory.
    /// 
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    /// 
    pub fn mac_aadata(additional_text: &T) -> SgxResult<Self> {

        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        } 
        let aad_slice: &[u8] = unsafe{slice::from_raw_parts(additional_text as * const _ as * const u8, size)};      
        let result = SgxInternalSealedData::mac_aadata(aad_slice);
        result.map(|x| SgxMacAadata {inner: x, marker: ptr::null()})
    }

    /// 
    /// This function is used to authenticate the input data with AES-GMAC. This is
    /// the expert mode version of the function mac_aadata.
    ///
    /// # Descryption
    /// 
    /// The mac_aadata_ex is an extended version of mac_aadata. It
    /// provides parameters for you to identify how to derive the sealing key (key
    /// policy and attributes_mask). Typical callers of the seal library should be
    /// able to use mac_aadata and the default values provided for key_policy (MR_SIGNER) and an attribute mask which includes the RESERVED,
    /// INITED and DEBUG bits. Before you use this function, you should have a clear
    /// understanding of the impact of using a policy and/or attribute_mask that
    /// is different from that in mac_aadata.
    /// # Requirements
    /// 
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///  
    /// # Parameters
    /// 
    /// **key_policy**
    /// 
    /// Specifies the policy to use in the key derivation. Function sgx_seal_data uses the MRSIGNER policy. 
    ///
    /// Key policy name | Value | Description 
    /// ---|---|---
    /// KEYPOLICY_MRENCLAVE | 0x0001 | -Derive key using the enclave??s ENCLAVE measurement register  
    /// KEYPOLICY_MRSIGNER |0x0002 | -Derive key using the enclave??s SIGNER measurement register 
    /// 
    /// **attribute_mask**
    /// 
    /// Identifies which platform/enclave attributes to use in the key derivation. See  
    /// the definition of sgx_attributes_t to determine which attributes will be  
    /// checked.  Function sgx_seal_data uses flags=0xfffffffffffffff3,?xfrm=0. 
    /// 
    /// **misc_mask**
    ///
    /// The misc mask bits for the enclave. Reserved for future function extension.
    ///
    /// **additional_text**
    ///
    /// Pointer to the additional Message Authentication Code (MAC) data. 
    /// This additional data is optional and no data is necessary.
    ///
    /// # Return value 
    ///
    /// The sealed data in SgxSealedData.
    ///
    /// # Errors
    /// 
    /// **SGX_ERROR_INVALID_PARAMETER**
    /// 
    /// Indicates an error if the parameters do not meet any of the following conditions:
    /// 
    /// * additional_text buffer can be within or outside the enclave, but cannot cross the enclave boundary.
    /// * encrypt_text must be non-zero.
    /// * encrypt_text buffer must be within the enclave.
    /// 
    /// **SGX_ERROR_OUT_OF_MEMORY**
    /// 
    /// The enclave is out of memory.
    /// 
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    /// 
    pub fn mac_aadata_ex(key_policy: u16,
                         attribute_mask: sgx_attributes_t,
                         misc_mask: sgx_misc_select_t,
                         additional_text: &T) -> SgxResult<Self> {

        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        } 
        let aad_slice: &[u8] = unsafe{slice::from_raw_parts(additional_text as * const _ as * const u8, size)};
        let result = SgxInternalSealedData::mac_aadata_ex(key_policy, 
                                                          attribute_mask, 
                                                          misc_mask, 
                                                          aad_slice);
        result.map(|x| SgxMacAadata {inner: x, marker: ptr::null()})
    }

    ///
    /// This function is used to verify the authenticity of the input sealed data structure using AES-GMAC. This function verifies the MAC generated with sgx_mac_aadataorsgx_mac_aadata_ex.
    ///
    /// # Descryption
    /// 
    /// The sgx_unmac_aadata function verifies the tag with AES-GMAC. Use this
    /// function to demonstrate the authenticity of data that was preserved by an
    /// earlier instantiation of this enclave.
    ///
    /// # Requirements
    /// 
    /// Library: libsgx_tservice.a or libsgx_tservice_sim.a (simulation)
    ///
    /// # Return value 
    ///
    /// The pointer of the additional data.
    ///
    /// # Errors
    /// 
    /// **SGX_ERROR_INVALID_PARAMETER**
    /// 
    /// The size of T may be zero.
    /// 
    /// **SGX_ERROR_INVALID_CPUSVN**
    /// 
    /// The CPUSVN in the data blob is beyond the CPUSVN value of the platform.
    /// 
    /// **SGX_ERROR_INVALID_ISVSVN**
    /// 
    /// The ISVSVN in the data blob is greater than the ISVSVN value of the enclave.
    /// 
    /// **SGX_ERROR_MAC_MISMATCH**
    /// 
    /// The tag verification fails. The error may be caused by a platform update, software update, or corruption of the sealed_data_t structure.
    /// 
    /// **SGX_ERROR_OUT_OF_MEMORY**
    /// 
    /// The enclave is out of memory.
    /// 
    /// **SGX_ERROR_UNEXPECTED**
    ///
    /// Indicates a crypto library failure or the RDRAND instruction fails to generate a
    /// random number.
    /// 
    pub fn unmac_aadata(&self) -> SgxResult<Box<T>> {

        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let aad_len = self.get_add_mac_txt_len() as usize;
        if size != aad_len {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        self.inner.unmac_aadata().map(|x| {
            let ptr = Box::into_raw(x.additional);
            unsafe{Box::from_raw(ptr as * mut _ as * mut T)}
        })
    }
}

impl<T> SgxMacAadata<[T]> {
    ///
    /// Create a SgxMacAadata with default values.
    /// 
    pub fn new() -> Self {
        SgxMacAadata::default()
    }

    /// 
    /// Convert a pointer of sgx_sealed_data_t buffer to SgxMacAadata.
    ///
    pub unsafe fn from_raw_sealed_data_t(p: * mut sgx_sealed_data_t, len: u32) -> Option<Self> {
        
        let size = mem::size_of::<T>();
        if size == 0 {
            return None;
        } 
        let opt = SgxInternalSealedData::from_raw_sealed_data_t(p, len);
        opt.map(|x| {
            let p = Box::<[T]>::default();
            SgxMacAadata{inner: x, marker: Box::into_raw(p)}
        })
    }

    /// 
    /// This function is used to authenticate the input data with AES-GMAC.
    ///
    pub fn mac_aadata(additional_text: &[T]) -> SgxResult<Self> {

        let size = mem::size_of::<T>() * additional_text.len();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let aad_slice: &[u8] = unsafe{slice::from_raw_parts(additional_text.as_ptr() as * const _ as * const u8, size)};      
        let result = SgxInternalSealedData::mac_aadata(aad_slice);
        result.map(|x| {
            let p = Box::<[T]>::default();
            SgxMacAadata {
                inner: x,
                marker: Box::into_raw(p),
            }
        })
    }

    /// 
    /// This function is used to authenticate the input data with AES-GMAC. This is
    /// the expert mode version of the function mac_aadata.
    ///
    pub fn mac_aadata_ex(key_policy: u16,
                         attribute_mask: sgx_attributes_t,
                         misc_mask: sgx_misc_select_t,
                         additional_text: &[T]) -> SgxResult<Self> {

        let size = mem::size_of::<T>() * additional_text.len();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let aad_slice: &[u8] = unsafe{slice::from_raw_parts(additional_text.as_ptr() as * const _ as * const u8, size)};
        let result = SgxInternalSealedData::mac_aadata_ex(key_policy, 
                                                          attribute_mask, 
                                                          misc_mask, 
                                                          aad_slice);
        result.map(|x| {
            let p = Box::<[T]>::default();
            SgxMacAadata {
                inner: x,
                marker: Box::into_raw(p),
            }
        })
    }

    ///
    /// This function is used to verify the authenticity of the input sealed data structure using AES-GMAC. This function verifies the MAC generated with sgx_mac_aadataorsgx_mac_aadata_ex.
    ///
    pub fn unmac_aadata(&self) -> SgxResult<Box<[T]>> {

        let size = mem::size_of::<T>();
        if size == 0 {
            return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER);
        }
        let aad_len = self.get_add_mac_txt_len() as usize;
        if size > aad_len {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        if (aad_len % size) != 0 {
            return Err(sgx_status_t::SGX_ERROR_MAC_MISMATCH);
        }
        self.inner.unmac_aadata().map(|x| {
            let ptr = Box::into_raw(x.additional);
            unsafe{Box::from_raw(ptr as * mut _ as * mut [T])}
        })
    }
}

impl<T: ?Sized> SgxMacAadata<T> {
    ///
    /// Get the size of payload in SgxMacAadata.
    ///
    pub fn get_payload_size(&self) -> u32 {
        self.inner.get_payload_size()
    }
    ///
    /// Get a slice of payload in SgxMacAadata.
    ///
    pub fn get_payload_tag(&self) -> &[u8; SGX_SEAL_TAG_SIZE] {
        self.inner.get_payload_tag()
    }
    ///
    /// Get the pointer of sgx_key_request_t in SgxMacAadata.
    ///
    pub fn get_key_request(&self) -> &sgx_key_request_t {
        self.inner.get_key_request()
    }

    ///
    /// Get a slice of additional text in SgxMacAadata.
    ///
    pub fn get_additional_txt(&self) -> &[u8] {
        self.inner.get_additional_txt()
    }

    ///
    /// Calculate the size of the sealed data in SgxMacAadata.
    ///
    pub fn calc_raw_sealed_data_size(add_mac_txt_size: u32, encrypt_txt_size: u32) -> u32  {
        SgxInternalSealedData::calc_raw_sealed_data_size(add_mac_txt_size, encrypt_txt_size)
    }

    ///
    /// Get the size of the additional mactext in SgxMacAadata.
    ///
    pub fn get_add_mac_txt_len(&self) -> u32 {
        self.inner.get_add_mac_txt_len()
    } 

    ///
    /// Convert SgxMacAadata to the pointer of sgx_sealed_data_t.
    ///
    /// # Parameters
    ///
    /// **p**
    ///
    /// The pointer of sgx_sealed_data_t to save the data in SgxMacAadata.
    ///
    /// **len**
    ///
    /// The size of the pointer of sgx_sealed_data_t.
    ///
    /// # Error
    ///
    /// **Some(* mut sgx_sealed_data_t)**
    ///
    /// Indicates the conversion is successfully. The return value is the pointer of sgx_sealed_data_t.
    ///
    /// **None**
    ///
    /// May be the parameter p and len is not avaliable.
    ///
    pub unsafe fn to_raw_sealed_data_t(&self, p: * mut sgx_sealed_data_t, len: u32) -> Option<* mut sgx_sealed_data_t> {
        self.inner.to_raw_sealed_data_t(p, len)
    }
}