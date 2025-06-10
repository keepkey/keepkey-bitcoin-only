# KeepKey Transport Quick Reference

## 🚨 CRITICAL: Format Differences

| Transport | Magic Bytes | Header Size | First Byte |
|-----------|-------------|-------------|------------|
| **v5 Protocol** | `## (0x23 0x23)` | 8 bytes | - |
| **HID (v4)** | `0x3F 0x23 0x23` | 10 bytes | 0x00 (Report ID) |
| **USB** | Uses v5 format | 8 bytes | 0x3F (?) |
| **WebUSB** | Uses v5 format | 8 bytes | 0x3F (?) |

## Packet Structure

### HID First Packet (64 bytes)
```
[0x00][0x3F][0x23][0x23][TypeHi][TypeLo][Len0][Len1][Len2][Len3][Data...54 bytes]
```

### USB/WebUSB First Packet (64 bytes)
```
[0x3F][0x23][0x23][TypeHi][TypeLo][Len0][Len1][Len2][Len3][Data...55 bytes]
```

### Continuation Packets (All transports)
```
[0x3F][Data...63 bytes]
```

## Code Snippets

### HID Write (v5 → v4 translation)
```rust
// Input has v5 format, convert to v4
packet[0] = 0x00;  // Report ID
packet[1] = 0x3F;  // Magic
packet[2] = 0x23;  
packet[3] = 0x23;
packet[4..6].copy_from_slice(&msg[2..4]);   // Type
packet[6..10].copy_from_slice(&msg[4..8]);  // Length
packet[10..].copy_from_slice(&msg[8..]);    // Data
```

### HID Read (v4 → v5 translation)
```rust
// Convert v4 response to v5 format
buf.push(0x23);  // #
buf.push(0x23);  // #
buf.extend_from_slice(&packet[3..9]);   // Type + Length
buf.extend_from_slice(&packet[9..]);    // Data
```

### USB Write (no translation needed)
```rust
packet[0] = 0x3F;  // ? marker
packet[1..].copy_from_slice(&msg);  // Already v5 format
```

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| "invalid tag value: 0" | Wrong magic bytes | Check HID uses v4 format |
| "buffer too short" | Missing continuation packets | Read until msg_length bytes |
| Device timeout | Wrong Report ID | HID must use 0x00 |
| No response | Double encoding | Don't add headers twice |

## Message Types

- **0**: Initialize → Features (17)
- **1**: Ping → Success (2)  
- **2**: Success (response)
- **3**: Failure (error)
- **17**: Features (device info)

## Remember

1. **HID is special** - Uses old v4 format!
2. **Message::encode()** already adds `##` header
3. **Always read continuation packets**
4. **Report ID 0x00** for HID only
5. **Test with Initialize message first** 