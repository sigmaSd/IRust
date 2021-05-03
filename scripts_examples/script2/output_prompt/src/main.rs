fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    let globals: irust_api::GlobalVariables = bincode::deserialize_from(handle).unwrap();

    bincode::serialize_into(
        std::io::stdout(),
        &format!("Out [{}]: ", globals.operation_number),
    )
    .unwrap();
}
