# devmem-rs

Small Rust library for accessing the physical address space using /dev/mem

## Example
```rust
use devmem::Mapping

let mut mapping = unsafe {
    Mapping::new(0x1000_0000, 8).unwrap()
};
let data_to_write: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
mapping.copy_from_slice(&data_to_write);
    
let mapping = unsafe {
    Mapping::new(0x1000_0004, 4).unwrap()
};
let mut data_read: Vec<u8> = vec![0x00; 4];
mapping.copy_into_slice(&mut data_read);

assert_eq!(data_read, data_to_write[4..8]);
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
