use crate::driver::tls::parse_buffer::ParseBuffer;
use crate::driver::tls::TlsError;
use core::fmt::{Debug, Formatter};
//use digest::generic_array::{ArrayLength, GenericArray};
use digest::Digest;
use generic_array::{ArrayLength, GenericArray};
use heapless::{consts::*, Vec};

pub struct Finished<N: ArrayLength<u8>> {
    pub verify: GenericArray<u8, N>,
    pub hash: Option<GenericArray<u8, N>>,
}

impl<N: ArrayLength<u8>> Debug for Finished<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Finished")
            .field("verify", &self.hash)
            .finish()
    }
}

impl<N: ArrayLength<u8>> Finished<N> {
    pub fn parse(buf: &mut ParseBuffer, len: u32) -> Result<Self, TlsError> {
        log::info!("finished len: {}", len);
        let mut verify = GenericArray::default();
        buf.fill(&mut verify);
        //let hash = GenericArray::from_slice()
        //let hash: Result<Vec<u8, _>, ()> = buf
        //.slice(len as usize)
        //.map_err(|_| TlsError::InvalidHandshake)?
        //.into();
        log::info!("hash {:?}", verify);
        //let hash = hash.map_err(|_| TlsError::InvalidHandshake)?;
        log::info!("hash ng {:?}", verify);
        Ok(Self { verify, hash: None })
    }

    pub(crate) fn encode<O: ArrayLength<u8>>(&self, buf: &mut Vec<u8, O>) -> Result<(), TlsError> {
        //let len = self.verify.len().to_be_bytes();
        //buf.extend_from_slice(&[len[1], len[2], len[3]]);
        buf.extend(self.verify.iter());
        Ok(())
    }
}