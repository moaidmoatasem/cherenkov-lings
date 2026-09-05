def read_heartbeat_payload(payload: bytes, claimed_length: int) -> bytes:
    """Echo back `claimed_length` bytes, never more than the payload actually has."""
    safe_length = min(claimed_length, len(payload))
    return payload[:safe_length]
