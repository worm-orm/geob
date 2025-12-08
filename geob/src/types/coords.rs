use core::fmt;

use udled::{
    AsSlice, Input, TokenizerExt,
    bytes::{Endian, FromBytes, FromBytesExt},
};

use crate::util::{read_f64, read_u32};

#[derive(Clone, Copy)]
pub struct CoordRef<'a> {
    data: &'a [u8],
    endian: Endian,
}

impl<'a> CoordRef<'a> {
    #[inline]
    pub fn x(&self) -> f64 {
        read_f64(self.data, self.endian)
    }

    #[inline]
    pub fn y(&self) -> f64 {
        read_f64(&self.data[8..], self.endian)
    }
}

impl<'a> PartialEq for CoordRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() && self.y() == other.y()
    }
}

impl<'a> fmt::Debug for CoordRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoordRef")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for CoordRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        endian: Endian,
    ) -> udled::Result<Self> {
        let x = reader.parse((f64::byteorder(endian), f64::byteorder(endian)).slice())?;
        Ok(CoordRef {
            data: x.value,
            endian,
        })
    }
}

// Bytecode: len(u32) 0..len(f64, f64)
#[derive(Clone, Copy)]
pub struct CoordSeqRef<'a> {
    data: &'a [u8],
    endian: Endian,
}

impl<'a> CoordSeqRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(self.data, self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<CoordRef<'a>> {
        if idx >= self.len() {
            return None;
        }

        let byte_idx = size_of::<u32>() + idx * 16;
        Input::new(&self.data[byte_idx..])
            .parse(CoordRef::byteorder(self.endian))
            .ok()
            .map(|m| m.value)
    }

    pub fn iter(&self) -> CoordSeqIter<'a> {
        CoordSeqIter {
            seg: *self,
            len: self.len(),
            idx: 0,
        }
    }
}

impl<'a> PartialEq for CoordSeqRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (left, right) in self.iter().zip(other.iter()) {
            if left != right {
                return false;
            }
        }

        true
    }
}

impl<'a> fmt::Debug for CoordSeqRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut writer = f.debug_list();

        writer.entries(self.iter());

        writer.finish()
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for CoordSeqRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let len = reader.parse(u32::byteorder(byteorder))?;

        let coords = reader.parse(
            CoordRef::byteorder(byteorder)
                .repeat(len.value as _)
                .spanned(),
        )?;

        let span = len.span + coords;

        Ok(CoordSeqRef {
            data: reader.buffer().sliced(span).unwrap(),
            endian: byteorder,
        })
    }
}

pub struct CoordSeqIter<'a> {
    seg: CoordSeqRef<'a>,
    len: usize,
    idx: usize,
}

impl<'a> Iterator for CoordSeqIter<'a> {
    type Item = CoordRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            return None;
        }

        let next = self.seg.get(self.idx);

        self.idx += 1;

        next
    }
}

#[derive(Clone, Copy)]
pub struct MultiCoordSeqRef<'a> {
    data: &'a [u8],
    endian: Endian,
}

impl<'a> MultiCoordSeqRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(self.data, self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<CoordSeqRef<'a>> {
        if idx >= self.len() {
            return None;
        }

        let mut input = Input::new(&self.data[4..]);

        if idx == 0 {
            input
                .parse(CoordSeqRef::byteorder(self.endian))
                .map(|m| m.value)
                .ok()
        } else {
            for i in 0..=idx {
                if i == idx {
                    return input
                        .parse(CoordSeqRef::byteorder(self.endian))
                        .map(|m| m.value)
                        .ok();
                } else {
                    input.eat(CoordSeqRef::byteorder(self.endian)).ok();
                }
            }

            None
        }
    }

    pub fn iter(&self) -> MultiCoordSeqIter<'a> {
        MultiCoordSeqIter {
            seg: *self,
            len: self.len(),
            idx: 0,
        }
    }
}

impl<'a> MultiCoordSeqRef<'a> {
    pub(crate) const fn new(data: &'a [u8], endian: Endian) -> MultiCoordSeqRef<'a> {
        MultiCoordSeqRef { data, endian }
    }
}

impl<'a> fmt::Debug for MultiCoordSeqRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut writer = f.debug_list();

        writer.entries(self.iter());

        writer.finish()
    }
}

impl<'a> PartialEq for MultiCoordSeqRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (left, right) in self.iter().zip(other.iter()) {
            if left != right {
                return false;
            }
        }

        true
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for MultiCoordSeqRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let len = reader.parse(u32::byteorder(byteorder))?;

        let coords = reader.parse(
            CoordSeqRef::byteorder(byteorder)
                .repeat(len.value as _)
                .spanned(),
        )?;

        let span = len.span + coords;

        Ok(MultiCoordSeqRef {
            data: reader.buffer().sliced(span).unwrap(),
            endian: byteorder,
        })
    }
}

pub struct MultiCoordSeqIter<'a> {
    seg: MultiCoordSeqRef<'a>,
    len: usize,
    idx: usize,
}

impl<'a> Iterator for MultiCoordSeqIter<'a> {
    type Item = CoordSeqRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            return None;
        }

        let next = self.seg.get(self.idx);

        self.idx += 1;

        next
    }
}

#[derive(Clone, Copy)]
pub struct CoordSegSegSegRef<'a> {
    data: &'a [u8],
    endian: Endian,
}

impl<'a> CoordSegSegSegRef<'a> {
    pub fn len(&self) -> usize {
        read_u32(self.data, self.endian) as _
    }

    pub fn get(&self, idx: usize) -> Option<MultiCoordSeqRef<'a>> {
        if idx >= self.len() {
            return None;
        }

        let mut input = Input::new(&self.data[4..]);

        if idx == 0 {
            input
                .parse(MultiCoordSeqRef::byteorder(self.endian))
                .map(|m| m.value)
                .ok()
        } else {
            for i in 0..=idx {
                if i == idx {
                    return input
                        .parse(MultiCoordSeqRef::byteorder(self.endian))
                        .map(|m| m.value)
                        .ok();
                } else {
                    input.parse(MultiCoordSeqRef::byteorder(self.endian)).ok();
                }
            }

            None
        }
    }

    pub fn iter(&self) -> CoordSegSegSegIter<'a> {
        CoordSegSegSegIter {
            seg: *self,
            len: self.len(),
            idx: 0,
        }
    }
}

impl<'a> fmt::Debug for CoordSegSegSegRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut writer = f.debug_list();

        writer.entries(self.iter());

        writer.finish()
    }
}

impl<'a> PartialEq for CoordSegSegSegRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (left, right) in self.iter().zip(other.iter()) {
            if left != right {
                return false;
            }
        }

        true
    }
}

impl<'input> FromBytes<'input, &'input [u8]> for CoordSegSegSegRef<'input> {
    fn parse(
        reader: &mut udled::Reader<'_, 'input, &'input [u8]>,
        byteorder: Endian,
    ) -> udled::Result<Self> {
        let len = reader.parse(u32::byteorder(byteorder))?;
        let coords = reader.parse(
            MultiCoordSeqRef::byteorder(byteorder)
                .repeat(len.value as _)
                .spanned(),
        )?;

        let span = len.span + coords;

        Ok(CoordSegSegSegRef {
            data: reader.buffer().sliced(span).unwrap(),
            endian: byteorder,
        })
    }
}

pub struct CoordSegSegSegIter<'a> {
    seg: CoordSegSegSegRef<'a>,
    len: usize,
    idx: usize,
}

impl<'a> Iterator for CoordSegSegSegIter<'a> {
    type Item = MultiCoordSeqRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            return None;
        }

        let next = self.seg.get(self.idx);

        self.idx += 1;

        next
    }
}
