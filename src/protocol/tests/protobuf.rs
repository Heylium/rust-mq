#[cfg(test)]
mod test {

    #[test]
    fn build_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/")
            .compile(
                &[
                    "src/common.proto",
                    "src/kv.proto",
                    "src/placement.proto",
                    "src/openraft.proto"
                ],
                &["src/"]
            ).unwrap();
    }
}