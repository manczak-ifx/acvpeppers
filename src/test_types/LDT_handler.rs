pub fn large_message_handler(large_msg: &LargeMsg) -> Vec<u8> {
     println!(
        "Processing large message: content={}, fullLength={}",
        large_msg.content, large_msg.fullLength
    );
    let content_bytes = hex::decode(&large_msg.content).expect("Invalid hex in 'content'");
    let mut expanded_message = Vec::new();
    let repeat_count = (large_msg.fullLength as usize) / content_bytes.len();

    for _ in 0..repeat_count {
        expanded_message.extend(&content_bytes);
    }
    let remainder = (large_msg.fullLength as usize) % content_bytes.len();
    if remainder > 0 {
        expanded_message.extend(&content_bytes[..remainder]);
    }

    expanded_message
}
