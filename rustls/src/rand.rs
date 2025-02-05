use crate::msgs::codec;
/// The single place where we generate random material
/// for our own use.  These functions never fail,
/// they panic on error.
use ring::rand::{SecureRandom, SystemRandom};

/// Fill the whole slice with random material.
pub fn fill_random(bytes: &mut [u8]) -> Result<(), GetRandomFailed> {
    SystemRandom::new().fill(bytes).map_err(|_| GetRandomFailed)
}

/// Make a Vec<u8> of the given size
/// containing random material.
pub fn random_vec(len: usize) -> Result<Vec<u8>, GetRandomFailed> {
    let mut v = vec![0; len];
    fill_random(&mut v)?;
    Ok(v)
}

/// Return a uniformly random u32.
pub fn random_u32() -> Result<u32, GetRandomFailed> {
    let mut buf = [0u8; 4];
    fill_random(&mut buf)?;
    codec::decode_u32(&buf).ok_or(GetRandomFailed)
}

#[derive(Debug)]
pub struct GetRandomFailed;
