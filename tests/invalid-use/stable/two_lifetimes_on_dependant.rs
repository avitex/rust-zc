use zc::Dependant;

#[derive(Dependant)]
pub struct MyStruct<'a, 'b>(&'a (), &'b ());

fn main() {}
