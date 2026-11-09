//! Uzunlik prefiksli freymlar.

use serde::{Serialize, de::DeserializeOwned};
use std::io::{self, Read, Write};

/// Bitta freym chegarasi. Katta ma’lumot freymda emas, vaqtinchalik faylda uzatiladi.
pub const MAX_FRAME_BYTES: u32 = 16 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("freym hajmi {0} bayt chegaradan katta")]
    TooLarge(u64),
    #[error("noto‘g‘ri freym mazmuni: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("oqim yopildi")]
    Closed,
}

pub fn write_frame<W: Write, T: Serialize>(out: &mut W, value: &T) -> Result<(), FrameError> {
    let body = serde_json::to_vec(value)?;
    let len = u32::try_from(body.len())
        .ok()
        .filter(|len| *len <= MAX_FRAME_BYTES)
        .ok_or(FrameError::TooLarge(body.len() as u64))?;
    out.write_all(&len.to_le_bytes())?;
    out.write_all(&body)?;
    out.flush()?;
    Ok(())
}

pub fn read_frame<R: Read, T: DeserializeOwned>(input: &mut R) -> Result<T, FrameError> {
    let mut len = [0_u8; 4];
    match input.read_exact(&mut len) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Err(FrameError::Closed),
        Err(e) => return Err(e.into()),
    }
    let len = u32::from_le_bytes(len);
    if len > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge(len.into()));
    }
    let mut body = vec![0_u8; len as usize];
    input.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_multiple_frames() {
        let mut buf = Vec::new();
        write_frame(&mut buf, &"salom o‘zbek").expect("write");
        write_frame(&mut buf, &42_u32).expect("write");
        let mut cur = Cursor::new(buf);
        assert_eq!(
            read_frame::<_, String>(&mut cur).expect("read"),
            "salom o‘zbek"
        );
        assert_eq!(read_frame::<_, u32>(&mut cur).expect("read"), 42);
        assert!(matches!(
            read_frame::<_, u32>(&mut cur),
            Err(FrameError::Closed)
        ));
    }

    #[test]
    fn oversized_length_is_rejected_before_allocation() {
        let mut cur = Cursor::new((MAX_FRAME_BYTES + 1).to_le_bytes().to_vec());
        assert!(matches!(
            read_frame::<_, u32>(&mut cur),
            Err(FrameError::TooLarge(_))
        ));
    }

    #[test]
    fn truncated_body_is_an_error() {
        let mut bytes = 10_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(b"{\"a\"");
        let mut cur = Cursor::new(bytes);
        assert!(read_frame::<_, serde_json::Value>(&mut cur).is_err());
    }
}
