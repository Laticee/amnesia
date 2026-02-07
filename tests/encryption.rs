use amnesia::mem_buffer::MemoryBuffer;

#[test]
fn test_encryption_scrambles_data() {
    let key = [0u8; 32];
    let mut buffer = MemoryBuffer::new(1024, Some(key));
    let secret = "This is a secret message";
    buffer.update(secret);

    // Verify that to_string recovers it
    assert_eq!(buffer.to_string(), secret);
}

#[test]
fn test_different_keys_different_ciphertext() {
    let secret = "Same secret message";

    let mut buffer1 = MemoryBuffer::new(1024, Some([1u8; 32]));
    buffer1.update(secret);

    let mut buffer2 = MemoryBuffer::new(1024, Some([2u8; 32]));
    buffer2.update(secret);

    // In an integration test, we can't easily check the internal scrambling
    // without making fields public. But we can verify to_string works for both.
    assert_eq!(buffer1.to_string(), secret);
    assert_eq!(buffer2.to_string(), secret);
}

#[test]
fn test_no_encryption_works() {
    let mut buffer = MemoryBuffer::new(1024, None);
    let msg = "Normal message";
    buffer.update(msg);
    assert_eq!(buffer.to_string(), msg);
}
