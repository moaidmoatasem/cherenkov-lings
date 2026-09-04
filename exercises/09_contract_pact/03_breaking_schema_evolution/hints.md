## Hint 1 (Architectural Nudge)
Additive changes (new optional fields) keep old contracts passing; removing or renaming a field a consumer already depends on does not. A good regression test proves both halves of that claim.

## Hint 2 (API Pattern)
Verify two Pact contracts against the same live provider: one that only requires fields that still exist (`verifier.verify()` should succeed, check `verifier.results["result"]`), and one pinned to a field the provider no longer returns (`verifier.verify()` should raise `RuntimeError` -- inspect `verifier.results["errors"]` for the mismatch detail).

## Hint 3 (Code Diff)
```diff
- res = requests.get("http://localhost:8081/api/pact/orders")
- assert res.status_code == 200
+ safe_verifier = Verifier("OrdersService").add_transport(url=PROVIDER_URL).add_source(safe_pact_file)
+ safe_verifier.verify()
+ assert safe_verifier.results["result"] is True
+
+ breaking_verifier = Verifier("OrdersService").add_transport(url=PROVIDER_URL).add_source(breaking_pact_file)
+ with pytest.raises(RuntimeError):
+     breaking_verifier.verify()
```
