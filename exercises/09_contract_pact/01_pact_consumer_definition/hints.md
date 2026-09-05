## Hint 1 (Architectural Nudge)
Consumer-Driven Contracts formalize the shape, types, and required fields that a consumer depends on -- and they do it against a *mock* of the provider, not the live service. The consumer under test should never make a real network call to `localhost:8081`.

## Hint 2 (API Pattern)
Build the interaction with `Pact(consumer, provider).upon_receiving(...).given(...).with_request(...).will_respond_with(...).with_body(...)`, using `pact.match.each_like` / `like` / `regex` for fields whose exact value shouldn't be pinned (only their type or pattern). Make the actual request inside `with pact.serve() as mock_server:`, against `mock_server.url` -- then call `pact.write_file(directory)` to emit the real Pact JSON contract.

## Hint 3 (Code Diff)
```diff
- res = requests.get("http://localhost:8081/api/pact/orders")
- assert res.status_code == 200
+ pact = Pact("OrdersWebClient", "OrdersService")
+ pact.upon_receiving("a request for the current orders").with_request(
+     "GET", "/api/pact/orders"
+ ).will_respond_with(200).with_body(
+     {"orders": match.each_like({"id": match.like("ORD-101"), "total": match.like(149.0)})}
+ )
+ with pact.serve() as mock_server:
+     res = requests.get(f"{mock_server.url}/api/pact/orders")
+     assert res.status_code == 200
+ pact.write_file(tmp_path, overwrite=True)
```
