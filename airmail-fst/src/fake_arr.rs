use std::ops::{Range, RangeFrom, RangeFull, RangeToInclusive};
use std::{
    fmt::Debug,
    ops::{Bound, RangeBounds},
};

pub type Ulen = u64; // maybe changeable? shouldn't be Ulen since then we couldn't use an index > 2GB in webassembly
pub type FakeArrRef<'a> = FakeArr<'a>;

#[derive(Debug, Clone)]
pub enum FakeArr<'a> {
    FakeLocal(Vec<u8>),
    Buffer(Buffer),
    Remote(String),
    Slice(&'a FakeArr<'a>, Ulen, Ulen),
}

impl<'a> FakeArr<'a> {
    pub fn len(&self) -> Ulen {
        todo!();
    }
    pub async fn read_into(&self, offset: Ulen, buf: &mut [u8]) -> std::io::Result<()> {
        todo!();
    }
    pub fn get_ofs_len(&self, start: Bound<Ulen>, end: Bound<Ulen>) -> (Ulen, Ulen) {
        use Bound::*;
        let start = match start {
            Unbounded => 0,
            Included(i) => i,
            Excluded(i) => panic!(),
        };
        let end = match end {
            Unbounded => self.len(),
            Included(i) => i + 1,
            Excluded(i) => i,
        };
        return (start, end - start);
    }
    /*fn slice_w_range(&self, e: SomRang) -> FakeArrPart<'_> {

    }*/
    pub fn slice(&'a self, bounds: ShRange<Ulen>) -> FakeArr<'a> {
        let (offset, len) = self.get_ofs_len(bounds.0, bounds.1);
        FakeArr::Slice(&self, offset, len)
    }
    pub fn full_slice(&self) -> FakeArr<'_> {
        self.slice((..).into())
    }
    pub async fn get_byte(&self, offset: Ulen) -> u8 {
        self.slice((offset..offset + 1).into())
            .actually_read_it()
            .await[0]
    }
    pub async fn actually_read_it(&self) -> Vec<u8> {
        let mut v = vec![0; self.len() as usize];
        self.read_into(0, &mut v).await.unwrap();
        v
    }
    pub async fn to_vec(&self) -> Vec<u8> {
        self.actually_read_it().await
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[macro_export]
macro_rules! slic {
    ($($e:ident).+ [$x:tt..]) => (($($e).*).slice((($x)..).into()));
    ($($e:ident).+ [$x:tt..$y:tt]) => (($($e).*).slice((($x)..($y)).into()));
    ($($e:ident).+ [..=$y:tt]) => (($($e).*).slice((..=($y)).into()));
    ($($e:ident).+ [..]) =>(($($e).*).slice((..).into()));
}
// #[macro_export]
// macro_rules! slic2 {
//     ($($e:ident).+ [$x:tt..]) => (($($e).*).slice2(($x..).into()));
//     ($($e:ident).+ [$x:tt..$y:tt]) => (($($e).*).slice2(($x..$y).into()));
//     ($($e:ident).+ [..=$y:tt]) => (($($e).*).slice2((..=$y).into()));
//     ($($e:ident).+ [..]) =>(($($e).*).slice2((..).into()));
//     ($($e:ident).+ [$x:tt]) => (($($e).*).get_byte($x));
// }
// todo: is there any better way?
pub struct ShRange<T>(Bound<T>, Bound<T>);

fn bound_cloned<T: Clone>(b: Bound<&T>) -> Bound<T> {
    match b {
        Bound::Unbounded => Bound::Unbounded,
        Bound::Included(x) => Bound::Included(x.clone()),
        Bound::Excluded(x) => Bound::Excluded(x.clone()),
    }
}
impl<T: Clone> From<Range<T>> for ShRange<T> {
    fn from(r: Range<T>) -> Self {
        ShRange(bound_cloned(r.start_bound()), bound_cloned(r.end_bound()))
    }
}
impl<T: Clone> From<RangeFull> for ShRange<T> {
    fn from(r: RangeFull) -> Self {
        ShRange(bound_cloned(r.start_bound()), bound_cloned(r.end_bound()))
    }
}
impl<T: Clone> From<RangeFrom<T>> for ShRange<T> {
    fn from(r: RangeFrom<T>) -> Self {
        ShRange(bound_cloned(r.start_bound()), bound_cloned(r.end_bound()))
    }
}
impl<T: Clone> From<RangeToInclusive<T>> for ShRange<T> {
    fn from(r: RangeToInclusive<T>) -> Self {
        ShRange(bound_cloned(r.start_bound()), bound_cloned(r.end_bound()))
    }
}

const EMPTY1: Vec<u8> = vec![];

pub fn empty() -> FakeArr<'static> {
    let x = FakeArr::FakeLocal(EMPTY1);
    x
}

pub fn local_slice_to_fake_arr<'a>(slice: &'a [u8]) -> FakeArr<'a> {
    FakeArr::FakeLocal(slice.to_vec())
}
