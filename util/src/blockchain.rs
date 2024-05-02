pub fn address_brief(address: String) -> String {
    let mut result = String::new();
    let len = address.len();
    if len <= 10 {
        result.push_str(&address);
    } else {
        result.push_str(&address[..4]);
        result.push_str("...");
        result.push_str(&address[len - 4..]);
    }
    result
}
