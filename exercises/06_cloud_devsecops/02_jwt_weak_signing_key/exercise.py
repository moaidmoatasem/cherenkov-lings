"""
PRODUCTION STORY:
Capital One AWS S3 Cloud Security Breach (2019)
Misconfigured token verification routines accepting unsigned or weakly-signed tokens ('none' algorithm)
allowed adversaries to forge administrative identities and exfiltrate over 100 million customer records.
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
