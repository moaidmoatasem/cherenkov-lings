## Hint 1 (Architectural Nudge)
Provider verification replays a consumer's recorded contract against the *real*, running provider -- the provider never gets to grade its own homework by asserting only the fields it happens to remember.

## Hint 2 (API Pattern)
Build a `Pact(...)` interaction like drill 01's, write it to a file with `pact.write_file(...)`, then feed that file into `Verifier(provider_name).add_transport(url=provider_url).add_source(pact_file)` and call `.verify()`. Check `verifier.results["result"]` to confirm every interaction passed.

## Hint 3 (Code Diff)
```diff
- res = requests.get("http://localhost:8081/api/pact/orders")
- assert res.status_code == 200
+ pact_file = _write_consumer_contract(tmp_path)
+ verifier = Verifier("OrdersService").add_transport(url=PROVIDER_URL).add_source(pact_file)
+ verifier.verify()
+ assert verifier.results["result"] is True
```
