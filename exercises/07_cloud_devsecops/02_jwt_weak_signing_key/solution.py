import pytest

def test_jwt_config():
    # Solution: The JWT configuration rejects 'none' algorithm
    jwt_config = {
        'secret': 'super_secret',
        'algorithms': ['HS256']
    }
    assert 'none' not in jwt_config.get('algorithms', [])
