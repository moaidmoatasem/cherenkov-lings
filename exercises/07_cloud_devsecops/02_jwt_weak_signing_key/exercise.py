"""
PRODUCTION STORY:
The 'alg: none' bypass in JWT libraries (disclosed 2015)
Libraries across several languages let the token name the algorithm used to verify it. Setting the
header to {"alg": "none"} and dropping the signature produced a token those servers accepted as valid,
so any claim -- including {"role": "admin"} -- could be forged without knowing a key.
"""

import pytest

def test_jwt_config():
    # Anti-pattern: The JWT configuration accepts the 'none' algorithm
    # TODO: Modify the config to only accept secure algorithms like 'HS256' or 'RS256'
    jwt_config = {
        'secret': 'super_secret',
        'algorithms': ['HS256', 'none']
    }
    assert 'none' not in jwt_config.get('algorithms', [])
