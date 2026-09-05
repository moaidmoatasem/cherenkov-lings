def read_heartbeat_payload(payload: bytes, claimed_length: int) -> bytes:
    """Echo back `claimed_length` bytes of the payload, as the client asked."""
    return payload[:claimed_length]
