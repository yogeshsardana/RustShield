use vstd::prelude::*;

verus! {

fn test_simple_addition() {
    let x: u64 = 5;
    let y: u64 = 7;
    let z = x + y;
    assert(z == 12);
}

fn test_boolean_logic() {
    assert(true);
    assert(!false);
    assert(true && true);
    assert(true || false);
}

fn test_ensures(x: u64) -> (y: u64)
    ensures
        y == x + 1,
{
    x + 1
}

} // verus!

fn main() {}
