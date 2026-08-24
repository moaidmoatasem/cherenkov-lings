# Hints: Drill 04 - Do Not Test the Mock

## Hint 1 (Concept)
A fake or stub is useful when the real dependency is unavailable. But when you fake everything and assert on the fake, you are only testing your own imagination. The Micro-Crucible exists so you always have a real -- but controllable -- target to test against.

## Hint 2 (Pattern)
Use the Crucible API for all integration tests. The base URL is http://localhost:8081. Use the requests library: response = requests.post("http://localhost:8081/checkout", json={"item_id": "item-1"})

## Hint 3 (Code Diff)
Remove: class FakePaymentGateway and gateway = FakePaymentGateway()
Add:    response = requests.post("http://localhost:8081/checkout", json={"item_id": "item-1"}, timeout=5)
        assert response.status_code == 200
