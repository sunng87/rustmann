fn main() {
    prost_build::compile_protos(&["protos/riemann.proto"], &["protos/"]).unwrap();
}
