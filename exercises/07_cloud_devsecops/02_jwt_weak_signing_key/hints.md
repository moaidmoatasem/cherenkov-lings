# Hints: Drill 02 - JWT Weak Signing Algorithm

## Hint 1 (Architectural Nudge)
The 'none' algorithm in JWT allows an attacker to bypass signature verification entirely by just stripping the signature and setting alg='none' in the header.

## Hint 2 (Code Diff)
Remove 'none' from the allowed algorithms list.

## Hint 3 (Full Solution Diff)
```diff
  jwt_config = {
      'secret': 'super_secret',
-     'algorithms': ['HS256', 'none']
+     'algorithms': ['HS256']
  }
```
