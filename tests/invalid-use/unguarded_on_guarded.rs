use zc::Guarded;

#[derive(Guarded)]
#[zc(unguarded)]
pub struct StructWithDefault<'a>(&'a u8);

fn main() {}
