mod ocl_test_class;

use ocl_test_class::OclTestClass;
use std::time::Instant;
fn main() {
    let mut ocl_test_class = OclTestClass::new().unwrap();
    // let a: usize = 2_usize.pow(50);
    let mut start = Instant::now();
    let a: u32 = 600_000_000; // max 4 billion
    for i in 0..20 {
        let new_quant = a + i;
        match ocl_test_class.run(new_quant) {
            Ok(_) => {
                println!("Time taken: {:?}", start.elapsed());
                start = Instant::now();
            }
            Err(_) => (),
        };
    }
}
